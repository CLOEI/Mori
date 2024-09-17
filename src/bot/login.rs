use paris::{error, info, warn};
use regex::Regex;
use serde_json::Value;
use std::{env, io, process::Command, time::Duration};
use std::process::Stdio;
use base64::Engine;
use base64::engine::general_purpose;
use egui::TextBuffer;
use ureq::Agent;
use urlencoding::encode;
use crate::utils::error;
use wait_timeout::ChildExt;
use crate::utils;

static USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

pub fn post_ubisoft_rememberme(agent: &Agent, ticket: &str) -> Result<String, ureq::Error> {
    let body = agent
        .post("https://public-ubiservices.ubi.com/v3/profiles/sessions")
        .set("User-Agent", USER_AGENT)
        .set("Ubi-AppId", "f2f8f582-6b7b-4d87-9a19-c72f07fccf99")
        .set("Ubi-RequestedPlatformType", "uplay")
        .set("Authorization", &format!("rm_v1 t={}", ticket))
        .set("Content-Type", "application/json")
        .send_json(
            ureq::json!({
                "rememberMe": true,
            }),
        )?;

    let json: Value = body.into_json()?;
    Ok(json["ticket"].to_string())
}

pub fn post_ubisoft_2fa_ticket(
    agent: &Agent,
    ticket: &str,
    token: &str,
) -> Result<String, ureq::Error> {
    println!("{} {}", ticket, token);
    let body = agent
        .post("https://public-ubiservices.ubi.com/v3/profiles/sessions")
        .set("User-Agent", USER_AGENT)
        .set("Ubi-AppId", "f2f8f582-6b7b-4d87-9a19-c72f07fccf99")
        .set("Ubi-RequestedPlatformType", "uplay")
        .set("Ubi-2faCode", &format!("{}", token))
        .set("Authorization", &format!("ubi_2fa_v1 t={}", ticket))
        .set("Content-Type", "application/json")
        .send_json(
            ureq::json!({
                "rememberMe": true,
            }),
        )?;

    let json: Value = body.into_json()?;
    let ticket = &json["ticket"].to_string();
    if json.get("rememberMeTicket").is_some() {
        let remember_me_ticket = json["rememberMeTicket"].as_str().unwrap().to_string();
        let ticket = post_ubisoft_rememberme(&agent, &remember_me_ticket)?;
        return Ok(ticket.to_string());
    }
    Ok(ticket.to_string())
}

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
    recovery_code: &str,
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
    if json.get("twoFactorAuthenticationTicket").is_some()
    {
        let ticket = &json["twoFactorAuthenticationTicket"].as_str().unwrap().to_string();
        let token = rust_otp::make_totp(&recovery_code.to_ascii_uppercase(), 30, 0).unwrap();
        let session_ticket = post_ubisoft_2fa_ticket(&agent, ticket, &token.to_string())?;
        Ok(session_ticket)
    } else {
        let game_token = get_ubisoft_game_token(agent, json["ticket"].as_str().unwrap().to_string().as_str())?;
        Ok(game_token)
    }
}

pub fn get_ubisoft_token(bot_info: &str, recovery_code: &str, email: &str, password: &str, steamuser: &str, steampassword: &str) -> Result<String, error::CustomError> {
    let agent = ureq::AgentBuilder::new().redirects(5).build();
    let session = match get_ubisoft_session(&agent, email, password, recovery_code) {
        Ok(res) => res,
        Err(err) => {
            return Err(error::CustomError::Other(format!("Failed to get ubisoft session: {}", err)));
        }
    };

    let current_dir = env::current_dir().expect("Failed to get current directory");
    let executable_path = current_dir.join("steamtoken.exe");
    let timeout = Duration::from_secs(5);

    loop {
        let mut child = Command::new(&executable_path)
            .arg("866020")
            .arg(steamuser)
            .arg(steampassword)
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to execute steamtoken");

        match child.wait_timeout(timeout).unwrap() {
            Some(status) => {
                if status.success() {
                    let output = child.wait_with_output().expect("Failed to read output");
                    let steam_token = utils::textparse::format_string_as_steam_token(String::from_utf8_lossy(&output.stdout).as_str());

                    let formated = encode(format!("UbiTicket|{}{}\n", session, bot_info).as_str()).to_string();
                    let body = agent
                        .post("https://login.growtopiagame.com/player/login/dashboard?valKey=40db4045f2d8c572efe8c4a060605726")
                        .set("user-agent", USER_AGENT)
                        .send_string(format!("{}steamToken%7C{}.240", formated, steam_token.trim_end()).as_str())?;

                    let json: Value = match body.into_json() {
                        Ok(json) => json,
                        Err(err) => {
                            return Err(error::CustomError::Other(format!("Failed to parse json: {}", err)));
                        }
                    };
                    return Ok(json["token"].to_string())
                }
            }
            None => {
                child.kill().expect("Failed to kill process");
                warn!("Process timed out, retrying...");
            }
        }
    }
}

pub fn get_apple_token(url: &str) -> Result<String, io::Error> {
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

    let body = req.into_string()?;
    let json: Value = serde_json::from_str(&body).unwrap();
    Ok(json["token"].as_str().unwrap().to_string())
}

pub fn extract_token_from_html(body: &str) -> Option<String> {
    let regex = Regex::new(r#"name="_token"\s+type="hidden"\s+value="([^"]*)""#).unwrap();
    regex
        .captures(body)
        .and_then(|cap| cap.get(1).map(|match_| match_.as_str().to_string()))
}
