use std::time::Duration;
use scraper::{Html, Selector};
use crate::constants::FHASH;
use crate::crypto::{compute_klv, hash_string, generate_rid};
use crate::server_data::LoginInfo;
use serde_json;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;


#[derive(Debug)]
pub struct DashboardLinks {
    pub apple: Option<String>,
    pub google: Option<String>,
    pub growtopia: Option<String>,
}

pub fn get_dashboard(login_url: &str, login_info: &LoginInfo, meta: &str) -> Result<DashboardLinks> {
    get_dashboard_proxied(login_url, login_info, meta, None)
}

pub fn get_dashboard_proxied(
    login_url: &str,
    login_info: &LoginInfo,
    meta: &str,
    proxy_url: Option<&str>,
) -> Result<DashboardLinks> {
    let rid = generate_rid();
    let hash = hash_string("RT");
    let klv = compute_klv(
        &login_info.game_version,
        &login_info.protocol.to_string(),
        &rid,
        hash,
    );

    let body = build_pipe_body(&[
        ("tankIDName",    ""),
        ("tankIDPass",    ""),
        ("requestedName", ""),
        ("f",             "1"),
        ("protocol",      &login_info.protocol.to_string()),
        ("game_version",  &login_info.game_version),
        ("cbits",         "0"),
        ("player_age",    "25"),
        ("GDPR",          "1"),
        ("FCMToken",      ""),
        ("category",      "_-5100"),
        ("totalPlaytime", "0"),
        ("klv",           &klv),
        ("meta",          meta),
        ("fhash",         &FHASH.to_string()),
        ("rid",           &rid),
        ("platformID",    "2"),
        ("deviceVersion", "0"),
        ("country",       "us"),
        ("hash",          &hash.to_string()),
        ("mac",           "02:00:00:00:00:00"),
        ("wk",            "NONE0"),
    ]);

    let agent = if let Some(p) = proxy_url {
        let proxy = ureq::Proxy::new(p)?;
        ureq::Agent::new_with_config(ureq::config::Config::builder().proxy(Some(proxy)).timeout_global(Some(Duration::from_secs(20))).build())
    } else {
        ureq::Agent::new_with_config(ureq::config::Config::builder().timeout_global(Some(Duration::from_secs(20))).build())
    };

    let html = agent
        .post(format!(
            "https://{}/player/login/dashboard?valKey=40db4045f2d8c572efe8c4a060605726",
            login_url
        ))
        .header(
            "User-Agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko)",
        )
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send(body)?
        .body_mut()
        .read_to_string()?;

    if html.trim_start().starts_with('{') {
        let msg = serde_json::from_str::<serde_json::Value>(&html)
            .ok()
            .and_then(|v| v["message"].as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| html.clone());
        return Err(format!("Dashboard returned error: {msg}").into());
    }

    let document = Html::parse_document(&html);
    let link_selector = Selector::parse("a")
        .map_err(|e| format!("Failed to parse selector: {e}"))?;

    let mut apple_href = None;
    let mut google_href = None;
    let mut growtopia_href = None;

    for element in document.select(&link_selector) {
        if let Some(onclick) = element.value().attr("onclick") {
            if onclick.contains("optionChose('Apple')") {
                apple_href = element.value().attr("href").map(Into::into);
            } else if onclick.contains("optionChose('Google')") {
                google_href = element.value().attr("href").map(Into::into);
            } else if onclick.contains("optionChose('Grow')") {
                growtopia_href = element.value().attr("href").map(Into::into);
            }
        }
    }

    Ok(DashboardLinks {
        apple: apple_href,
        google: google_href,
        growtopia: growtopia_href,
    })
}

fn build_pipe_body(fields: &[(&str, &str)]) -> String {
    fields.iter().map(|(k, v)| format!("{k}|{v}\n")).collect()
}
