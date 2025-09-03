use crate::types::login_info::LoginInfo;
use crate::types::server_data::ServerData;
use urlencoding::encode;
use anyhow::Result;
use scraper::{Html, Selector};
use serde_json::Value;

#[derive(Debug)]
#[derive(Clone)]
pub struct DashboardLinks {
    pub apple: Option<String>,
    pub google: Option<String>,
    pub growtopia: Option<String>,
}

pub fn check_token(token: &str, login_info: &str) -> Result<String> {
    if token.is_empty() {
        return Err(anyhow::anyhow!("Token is empty"));
    }

    let req = ureq::post("https://login.growtopiagame.com/player/growid/checktoken?valKey=40db4045f2d8c572efe8c4a060605726")
        .header("User-Agent", "UbiServices_SDK_2022.Release.9_PC64_ansi_static")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send_form([
            ("refreshToken", token),
            ("clientData", login_info),
        ])?
    .body_mut()
    .read_json::<Value>();

    match req {
        Ok(response) => {
            if response["status"] == "success" {
                let new_token = response["token"].as_str().unwrap_or_default().to_string();
                Ok(new_token)
            } else {
                Err(anyhow::anyhow!("Token validation failed: {}", response))
            }
        }
        Err(e) => {
            Err(anyhow::anyhow!("Failed to validate token: {}", e))
        }
    }
}

pub fn get_server_data(alternate: bool, login_info: &LoginInfo) -> Result<ServerData> {
    let url = if alternate {
        "https://www.growtopia1.com/growtopia/server_data.php"
    } else {
        "https://www.growtopia2.com/growtopia/server_data.php"
    };

    let req = ureq::post(url)
        .header("User-Agent", "UbiServices_SDK_2022.Release.9_PC64_ansi_static")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send(&format!("platform=0&protocol={}&version={}", login_info.protocol, login_info.game_version))?
        .body_mut()
        .read_to_string();

    match req {
        Ok(body) => ServerData::parse_from_response(&body),
        Err(e) => Err(anyhow::anyhow!("Failed to get server data: {}", e)),
    }
}

pub fn get_dashboard(login_url: &str, login_info: &LoginInfo) -> Result<DashboardLinks> {
    let data = encode(&login_info.to_string()).to_string();
    let req = ureq::post(format!("https://{}/player/login/dashboard?valKey=40db4045f2d8c572efe8c4a060605726", login_url))
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36 Edg/139.0.0.0")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send(data)
        .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?
        .body_mut()
        .read_to_string()
        .map_err(|e| anyhow::anyhow!("Failed to read response: {}", e))?;

    let document = Html::parse_document(&req);
    let link_selector = Selector::parse("a").map_err(|e| anyhow::anyhow!("Failed to parse selector: {}", e))?;

    let mut apple_href = None;
    let mut google_href = None;
    let mut growtopia_href = None;

    for element in document.select(&link_selector) {
        if let Some(onclick) = element.value().attr("onclick") {
            if onclick.contains("optionChose('Apple')") {
                apple_href = element.value().attr("href").map(|s| s.to_string());
            } else if onclick.contains("optionChose('Google')") {
                google_href = element.value().attr("href").map(|s| s.to_string());
            } else if onclick.contains("optionChose('Grow')") {
                growtopia_href = element.value().attr("href").map(|s| s.to_string());
            }
        }
    }

    Ok(DashboardLinks {
        apple: apple_href,
        google: google_href,
        growtopia: growtopia_href,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_dashboard() {
        let mut login_info = LoginInfo::new();
        let server_data = get_server_data(false, &login_info).unwrap();
        login_info.meta = server_data.meta;
        let res = get_dashboard(&server_data.loginurl, &login_info);
        assert!(res.is_ok());
    }
}