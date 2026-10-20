use std::fs;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};
use std::time::Duration;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::constants::{DEFAULT_FHASH, DEFAULT_GAME_VER, DEFAULT_PROTOCOL};
use crate::items::proton_sdk_hash;

pub const LATEST_CACHE_API: &str = "https://mari-project.jad.li/api/v1/growtopia/cache/latest";
pub const ITEMS_ARCHIVE_LATEST_JSON: &str =
    "https://raw.githubusercontent.com/StileDevs/itemsdat-archive/main/latest.json";
pub const ITEMS_ARCHIVE_RAW_BASE: &str =
    "https://raw.githubusercontent.com/StileDevs/itemsdat-archive/main";

fn default_protocol() -> u32 {
    DEFAULT_PROTOCOL
}

fn default_fhash() -> i32 {
    DEFAULT_FHASH
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub game_version: String,
    #[serde(default = "default_protocol")]
    pub protocol: u32,
    #[serde(default = "default_fhash")]
    pub fhash: i32,
    #[serde(default)]
    pub items_content: String,
    #[serde(default)]
    pub items_hash: i32,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            game_version: DEFAULT_GAME_VER.to_string(),
            protocol: DEFAULT_PROTOCOL,
            fhash: DEFAULT_FHASH,
            items_content: String::new(),
            items_hash: 0,
        }
    }
}

#[derive(Deserialize)]
struct MariLatestResponse {
    version: String,
    #[serde(default)]
    #[allow(dead_code)]
    uri: Option<String>,
}

#[derive(Deserialize)]
pub struct ArchiveLatestItem {
    pub content: String,
    pub hash: i32,
}

#[derive(Deserialize)]
struct ArchiveLatestResponse {
    latest: ArchiveLatestItem,
}

static CONFIG: OnceLock<RwLock<CacheConfig>> = OnceLock::new();

pub fn cache_dir() -> PathBuf {
    PathBuf::from(".cache")
}

pub fn items_dat_path() -> PathBuf {
    cache_dir().join("items.dat")
}

pub fn config_path() -> PathBuf {
    cache_dir().join("config.json")
}

pub fn get_game_version() -> String {
    if let Some(lock) = CONFIG.get() {
        lock.read().unwrap().game_version.clone()
    } else if let Ok(config) = load_cached_config() {
        config.game_version
    } else {
        DEFAULT_GAME_VER.to_string()
    }
}

pub fn get_protocol() -> u32 {
    if let Some(lock) = CONFIG.get() {
        lock.read().unwrap().protocol
    } else if let Ok(config) = load_cached_config() {
        config.protocol
    } else {
        DEFAULT_PROTOCOL
    }
}

pub fn get_fhash() -> i32 {
    if let Some(lock) = CONFIG.get() {
        lock.read().unwrap().fhash
    } else if let Ok(config) = load_cached_config() {
        config.fhash
    } else {
        DEFAULT_FHASH
    }
}

fn load_cached_config() -> Result<CacheConfig> {
    let path = config_path();
    let data = fs::read_to_string(path)?;
    let cfg = serde_json::from_str(&data)?;
    Ok(cfg)
}

fn save_cached_config(cfg: &CacheConfig) -> Result<()> {
    fs::create_dir_all(cache_dir())?;
    let data = serde_json::to_string_pretty(cfg)?;
    fs::write(config_path(), data)?;
    Ok(())
}

fn create_agent() -> ureq::Agent {
    ureq::Agent::new_with_config(
        ureq::config::Config::builder()
            .timeout_global(Some(Duration::from_secs(15)))
            .build(),
    )
}

pub fn fetch_latest_game_version(agent: &ureq::Agent) -> Result<String> {
    let mut resp = agent
        .get(LATEST_CACHE_API)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Mori/2.0",
        )
        .call()?;
    let body = resp.body_mut().read_to_string()?;
    let data: MariLatestResponse = serde_json::from_str(&body)?;
    if data.version.trim().is_empty() {
        bail!("empty version returned by {}", LATEST_CACHE_API);
    }
    Ok(data.version)
}

pub fn fetch_latest_items_meta(agent: &ureq::Agent) -> Result<ArchiveLatestItem> {
    let mut resp = agent
        .get(ITEMS_ARCHIVE_LATEST_JSON)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Mori/2.0",
        )
        .call()?;
    let body = resp.body_mut().read_to_string()?;
    let data: ArchiveLatestResponse = serde_json::from_str(&body)?;
    Ok(data.latest)
}

pub fn download_items_dat(agent: &ureq::Agent, filename: &str) -> Result<Vec<u8>> {
    let url = format!("{}/{}", ITEMS_ARCHIVE_RAW_BASE, filename);
    let mut resp = agent
        .get(&url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Mori/2.0",
        )
        .call()?;
    let bytes = resp.body_mut().read_to_vec()?;
    Ok(bytes)
}

/// Runs the auto-update setup:
/// 1. Ensures the `.cache` directory exists.
/// 2. Fetches the latest game version from `mari-project`.
/// 3. Checks the latest `items.dat` archive from `StileDevs/itemsdat-archive`.
/// 4. Downloads and verifies `.cache/items.dat` using `proton_sdk_hash` if missing or outdated.
/// 5. Saves state to `.cache/config.json` and updates the active runtime config.
pub fn init() {
    println!("[Setup] Initializing auto-update check...");
    let _ = fs::create_dir_all(cache_dir());

    let mut current_config = load_cached_config().unwrap_or_default();
    let agent = create_agent();

    // 1. Check latest game version
    match fetch_latest_game_version(&agent) {
        Ok(v) => {
            println!("[Setup] Latest Growtopia version: {}", v);
            current_config.game_version = v;
        }
        Err(e) => {
            eprintln!(
                "[Setup] Warning: Failed to fetch latest game version: {e}. Using: {}",
                current_config.game_version
            );
        }
    }

    // 2. Check latest items.dat archive
    match fetch_latest_items_meta(&agent) {
        Ok(meta) => {
            println!(
                "[Setup] Latest items archive: {} (hash: {})",
                meta.content, meta.hash
            );

            let items_path = items_dat_path();
            let mut need_download = true;

            if items_path.exists() {
                match fs::read(&items_path) {
                    Ok(bytes) => {
                        let local_hash = proton_sdk_hash(&bytes);
                        if local_hash == meta.hash {
                            println!(
                                "[Setup] .cache/items.dat is up to date (hash: {local_hash})"
                            );
                            need_download = false;
                            current_config.items_hash = meta.hash;
                            current_config.items_content = meta.content.clone();
                        } else {
                            println!(
                                "[Setup] .cache/items.dat hash mismatch (local: {local_hash}, remote: {}), updating...",
                                meta.hash
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("[Setup] Failed reading existing .cache/items.dat: {e}");
                    }
                }
            } else {
                println!("[Setup] .cache/items.dat not found, downloading from archive...");
            }

            if need_download {
                match download_items_dat(&agent, &meta.content) {
                    Ok(bytes) => {
                        let hash = proton_sdk_hash(&bytes);
                        if hash == meta.hash {
                            if let Err(e) = fs::write(&items_path, &bytes) {
                                eprintln!("[Setup] Failed to save .cache/items.dat: {e}");
                            } else {
                                println!(
                                    "[Setup] Successfully downloaded and verified .cache/items.dat ({} bytes, hash: {hash})",
                                    bytes.len()
                                );
                                current_config.items_hash = meta.hash;
                                current_config.items_content = meta.content;
                            }
                        } else {
                            eprintln!(
                                "[Setup] Downloaded items.dat hash verification failed: got {hash}, expected {}",
                                meta.hash
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("[Setup] Failed to download {}: {e}", meta.content);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("[Setup] Warning: Failed to fetch items archive: {e}");
            let items_path = items_dat_path();
            if items_path.exists() {
                if let Ok(bytes) = fs::read(&items_path) {
                    let hash = proton_sdk_hash(&bytes);
                    current_config.items_hash = hash;
                    println!("[Setup] Using local .cache/items.dat (hash: {hash})");
                }
            }
        }
    }

    // Save updated config
    if let Err(e) = save_cached_config(&current_config) {
        eprintln!("[Setup] Failed to save .cache/config.json: {e}");
    }

    // Store in global static
    if let Some(lock) = CONFIG.get() {
        *lock.write().unwrap() = current_config;
    } else {
        let _ = CONFIG.set(RwLock::new(current_config));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_paths() {
        assert_eq!(cache_dir(), PathBuf::from(".cache"));
        assert_eq!(items_dat_path(), PathBuf::from(".cache/items.dat"));
        assert_eq!(config_path(), PathBuf::from(".cache/config.json"));
    }

    #[test]
    fn test_config_defaults() {
        let cfg = CacheConfig::default();
        assert_eq!(cfg.protocol, DEFAULT_PROTOCOL);
        assert_eq!(cfg.fhash, DEFAULT_FHASH);
    }

    #[test]
    fn test_fetch_latest_version() {
        let agent = create_agent();
        let res = fetch_latest_game_version(&agent);
        assert!(res.is_ok(), "Failed to fetch version: {:?}", res.err());
        let version = res.unwrap();
        assert!(!version.is_empty());
        println!("Fetched game version: {version}");
    }

    #[test]
    fn test_fetch_latest_items_meta() {
        let agent = create_agent();
        let res = fetch_latest_items_meta(&agent);
        assert!(res.is_ok(), "Failed to fetch items meta: {:?}", res.err());
        let meta = res.unwrap();
        assert!(meta.content.starts_with("items-"));
        assert_ne!(meta.hash, 0);
        println!(
            "Fetched latest archive: {} (hash: {})",
            meta.content, meta.hash
        );
    }
}
