use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};
use uuid::Uuid;

// ── Persistent credential store ───────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
struct UserRecord {
    /// Argon2id PHC string
    password_hash: String,
}

fn user_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("data")
        .join("user.json")
}

fn load_user() -> Option<UserRecord> {
    let path = user_path();
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

fn save_user(record: &UserRecord) -> anyhow::Result<()> {
    let path = user_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(record)?;
    std::fs::write(path, data)?;
    Ok(())
}

// ── Auth state (shared across requests) ──────────────────────────────────────

#[derive(Clone)]
pub struct AuthState {
    inner: Arc<RwLock<AuthInner>>,
}

struct AuthInner {
    /// None means not yet set up.
    password_hash: Option<String>,
    /// Active session token (single-user, single-session).
    session_token: Option<String>,
}

impl AuthState {
    pub fn new() -> Self {
        let password_hash = load_user().map(|u| u.password_hash);
        Self {
            inner: Arc::new(RwLock::new(AuthInner {
                password_hash,
                session_token: None,
            })),
        }
    }

    pub fn is_registered(&self) -> bool {
        self.inner.read().unwrap().password_hash.is_some()
    }

    pub fn register(&self, password: &str) -> anyhow::Result<()> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("hash error: {}", e))?
            .to_string();

        save_user(&UserRecord {
            password_hash: hash.clone(),
        })?;

        let mut inner = self.inner.write().unwrap();
        inner.password_hash = Some(hash);
        inner.session_token = None;
        Ok(())
    }

    pub fn login(&self, password: &str) -> Option<String> {
        let inner = self.inner.read().unwrap();
        let hash_str = inner.password_hash.as_deref()?;

        let parsed = PasswordHash::new(hash_str).ok()?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .ok()?;

        drop(inner);

        let token = Uuid::new_v4().to_string();
        self.inner.write().unwrap().session_token = Some(token.clone());
        Some(token)
    }

    /// Returns true when the given token matches the active session.
    pub fn validate_token(&self, token: &str) -> bool {
        self.inner
            .read()
            .unwrap()
            .session_token
            .as_deref()
            .map(|t| t == token)
            .unwrap_or(false)
    }

    pub fn logout(&self) {
        self.inner.write().unwrap().session_token = None;
    }
}
