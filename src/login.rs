use scraper::{Html, Selector};
use serde_json::Value;
use std::fmt;

#[derive(Debug)]
pub enum LoginError {
    Exhausted,
    WrongCredentials,
    Other(String),
}

impl fmt::Display for LoginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoginError::Exhausted        => write!(f, "Login attempts exhausted. Please try again after 24 hours."),
            LoginError::WrongCredentials => write!(f, "Account credentials mismatched."),
            LoginError::Other(msg)       => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for LoginError {}

type Result<T> = std::result::Result<T, LoginError>;

pub fn get_legacy_token(url: &str, username: &str, password: &str) -> Result<String> {
    get_legacy_token_proxied(url, username, password, None)
}

pub fn get_legacy_token_proxied(
    url: &str,
    username: &str,
    password: &str,
    proxy_url: Option<&str>,
) -> Result<String> {
    let agent = if let Some(p) = proxy_url {
        let proxy = ureq::Proxy::new(p).map_err(|e| LoginError::Other(e.to_string()))?;
        ureq::Agent::new_with_config(ureq::config::Config::builder().proxy(Some(proxy)).build())
    } else {
        ureq::Agent::new_with_defaults()
    };

    let html = agent
        .get(url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko)",
        )
        .call().map_err(|e| LoginError::Other(e.to_string()))?
        .body_mut()
        .read_to_string().map_err(|e| LoginError::Other(e.to_string()))?;

    let csrf_token = extract_csrf_token(&html)
        .ok_or_else(|| LoginError::Other("Failed to extract CSRF token from login page".into()))?;

    let response = agent
        .post("https://login.growtopiagame.com/player/growid/login/validate")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko)",
        )
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Origin", "https://login.growtopiagame.com")
        .header("Referer", url)
        .send_form([
            ("_token", csrf_token.as_str()),
            ("growId", username),
            ("password", password),
        ]).map_err(|e| LoginError::Other(e.to_string()))?;

    if response.status() != 200 {
        return Err(LoginError::Other(format!("Login failed with status: {}", response.status())));
    }

    let body = response.into_body().read_to_string()
        .map_err(|e| LoginError::Other(e.to_string()))?;

    if let Ok(json) = serde_json::from_str::<Value>(&body) {
        return json["token"]
            .as_str()
            .map(Into::into)
            .ok_or_else(|| LoginError::Other("Missing 'token' field in login response".into()));
    }

    // Server redirected to HTML page — check for error message
    let document = Html::parse_document(&body);
    if let Ok(selector) = Selector::parse(".text-danger.text-danger-wrapper") {
        if let Some(el) = document.select(&selector).next() {
            let msg = el.text().collect::<String>();
            let msg = msg.trim();
            if !msg.is_empty() {
                let error = if msg.to_lowercase().contains("exhausted") {
                    LoginError::Exhausted
                } else if msg.to_lowercase().contains("mismatched") {
                    LoginError::WrongCredentials
                } else {
                    LoginError::Other(msg.to_string())
                };
                return Err(error);
            }
        }
    }

    Err(LoginError::Other("Login failed: unexpected response from server".into()))
}

pub fn check_token(
    token: &str,
    login_info: &str,
    proxy_url: Option<&str>,
) -> anyhow::Result<String> {
    if token.is_empty() {
        return Err(anyhow::anyhow!("Token is empty"));
    }

    let agent = if let Some(p) = proxy_url {
        let proxy = ureq::Proxy::new(p)?;
        ureq::Agent::new_with_config(ureq::config::Config::builder().proxy(Some(proxy)).build())
    } else {
        ureq::Agent::new_with_defaults()
    };

    let body = agent
        .post("https://login.growtopiagame.com/player/growid/checktoken?valKey=40db4045f2d8c572efe8c4a060605726")
        .header("User-Agent", "UbiServices_SDK_2022.Release.9_PC64_ansi_static")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send_form([
            ("refreshToken", token),
            ("clientData", login_info),
        ])?
        .body_mut()
        .read_to_string()?;

    let response: Value = serde_json::from_str(&body)?;

    if response["status"] == "success" {
        let new_token = response["token"].as_str().unwrap_or_default().to_string();
        Ok(new_token)
    } else {
        Err(anyhow::anyhow!("Token validation failed: {}", response))
    }
}

fn extract_csrf_token(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("input[name='_token']").ok()?;
    document
        .select(&selector)
        .next()?
        .value()
        .attr("value")
        .map(Into::into)
}
