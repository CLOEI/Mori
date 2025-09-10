use anyhow::Result;
use regex::Regex;
use serde_json::Value;
use ureq::config::Config;

pub fn get_legacy_token(url: &str, username: &str, password: &str) -> Result<String> {
    get_legacy_token_with_proxy(url, username, password, None)
}

pub fn get_legacy_token_with_proxy(url: &str, username: &str, password: &str, proxy: Option<&str>) -> Result<String> {
    let agent = if let Some(proxy_url) = proxy {
        let proxy = ureq::Proxy::new(proxy_url)?;
        ureq::Agent::new_with_config(
            Config::builder()
                .proxy(Some(proxy))
                .build()
        )
    } else {
        ureq::Agent::new_with_defaults()
    };

    let body = agent
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36 Edg/139.0.0.0")
        .call()?
        .body_mut()
        .read_to_string()?;

    let token = match extract_token_from_html(&body) {
        Some(token) => token,
        None => return Err(anyhow::anyhow!("Failed to extract token from HTML")),
    };

    let mut req = agent
        .post("https://login.growtopiagame.com/player/growid/login/validate")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send_form([
            ("_token", &token),
            ("growId", &username.to_string()),
            ("password", &password.to_string()),
        ])?;

    if req.status() == 200 {
        let body = req.body_mut().read_to_string()?;
        let json: Value = serde_json::from_str(&body)?;
        Ok(json["token"].as_str().unwrap().to_string())
    } else {
        Err(anyhow::anyhow!(
            "Login failed with status: {}",
            req.status()
        ))
    }
}

fn extract_token_from_html(body: &str) -> Option<String> {
    let regex = Regex::new(r#"name="_token"\s+type="hidden"\s+value="([^"]*)""#).unwrap();
    regex
        .captures(body)
        .and_then(|cap| cap.get(1).map(|match_| match_.as_str().to_string()))
}
