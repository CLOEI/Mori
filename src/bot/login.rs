use paris::{error, info};
use regex::Regex;
use serde_json::Value;
use std::{io, process::Command, time::Duration};
use std::sync::Arc;
use base64::Engine;
use base64::engine::general_purpose;
use ureq::Agent;
use urlencoding::encode;
use crate::bot::Bot;

static USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

pub fn get_ubisoft_game_token(agent: &Agent, token: &str) -> Result<String, ureq::Error> {
    let body = agent
        .post("https://public-ubiservices.ubi.com/v3/profiles/sessions")
        .set("User-Agent", USER_AGENT)
        .set("Authorization", &format!("Ubi_v1 t={}", token))
        .set("Ubi-AppId", "f2f8f582-6b7b-4d87-9a19-c72f07fccf99")
        .set("Ubi-RequestedPlatformType", "uplay")
        .set("Content-Type", "application/json")
        .send_json(
            ureq::json!({
                "rememberMe": true,
            }),
        )?;

    let json: Value = body.into_json()?;
    Ok(json["ticket"].as_str().unwrap().to_string())
}

pub fn get_ubisoft_session(
    agent: &Agent,
    email: &str,
    password: &str,
) -> Result<String, ureq::Error> {
    let encoded = general_purpose::STANDARD.encode(format!("{}:{}", email, password));
    let body = agent
        .post("https://public-ubiservices.ubi.com/v3/profiles/sessions")
        .set("User-Agent", USER_AGENT)
        .set("Authorization", &format!("Basic {}", encoded))
        .set("Ubi-AppId", "afb4b43c-f1f7-41b7-bcef-a635d8c83822")
        .set("Ubi-RequestedPlatformType", "uplay")
        .set("Content-Type", "application/json")
        .send_json(
           ureq::json!({
               "rememberMe": true,
           }),
        )?;

    let json: Value = body.into_json()?;
    let game_token = get_ubisoft_game_token(agent, json["ticket"].as_str().unwrap().to_string().as_str())?;
    Ok(game_token)
}

pub fn get_ubisoft_token(bot_info: &str, email: &str, password: &str) -> Result<String, ureq::Error> {
    let agent = ureq::AgentBuilder::new().redirects(5).build();
    let session = match get_ubisoft_session(&agent, email, password) {
        Ok(res) => res,
        Err(err) => {
            return Err(err);
        }
    };

    let formated = encode(format!("UbiTicket|{}{}\nsteamToken|{}", session, bot_info, "").as_str()).to_string();
    let body = agent
        .post("https://login.growtopiagame.com/player/login/dashboard?valKey=40db4045f2d8c572efe8c4a060605726")
        .set("user-agent", USER_AGENT)
        .send_string(&formated)?;

    let json: Value = body.into_json()?;
    Ok(json["token"].to_string())
}

pub fn get_apple_token(url: &str) -> Result<String, std::io::Error> {
    println!("Getting apple token");
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(&["/c", "start", "", url])
            .spawn()
            .expect("Failed to open URL on Windows");
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(url)
            .spawn()
            .expect("Failed to open URL on Linux");
    }

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    Ok(buffer)
}

pub fn get_google_token(url: &str, username: &str, password: &str) -> Result<String, ureq::Error> {
    loop {
        let response = ureq::post("http://localhost:5123/token")
            .timeout(Duration::from_secs(60))
            .send_form(&[("url", url), ("email", username), ("password", password)]);

        match response {
            Ok(res) => {
                if res.status() == 200 {
                    return Ok(res.into_string()?);
                } else {
                    error!("Failed to get token, retrying...");
                }
            }
            Err(err) => {
                error!("Request error: {}, retrying...", err);
            }
        }

        std::thread::sleep(Duration::from_secs(1));
    }
}

pub fn get_legacy_token(url: &str, username: &str, password: &str) -> Result<String, ureq::Error> {
    let agent = ureq::AgentBuilder::new().build();
    let body = agent
        .get(url)
        .set("User-Agent", USER_AGENT)
        .call()?
        .into_string()?;

    let token = match extract_token_from_html(&body) {
        Some(token) => token,
        None => panic!("Failed to extract token"),
    };

    let req = agent
        .post("https://login.growtopiagame.com/player/growid/login/validate")
        .send_form(&[
            ("_token", &token),
            ("growId", &username),
            ("password", &password),
        ])?;

    let body = req.into_string().unwrap();
    let json: Value = serde_json::from_str(&body).unwrap();
    Ok(json["token"].as_str().unwrap().to_string())
}

pub fn extract_token_from_html(body: &str) -> Option<String> {
    let regex = Regex::new(r#"name="_token"\s+type="hidden"\s+value="([^"]*)""#).unwrap();
    regex
        .captures(body)
        .and_then(|cap| cap.get(1).map(|match_| match_.as_str().to_string()))
}
