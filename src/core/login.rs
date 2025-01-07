use crate::core::Bot;
use crate::utils;
use crate::utils::error;
use base64::engine::general_purpose;
use base64::Engine;
use egui::TextBuffer;
use paris::{error, info, warn};
use regex::Regex;
use serde_json::Value;
use std::process::Stdio;
use std::sync::Arc;
use std::{env, io, process::Command, time::Duration};
use ureq::Agent;
use urlencoding::encode;
use wait_timeout::ChildExt;
use crate::types::bot_info::EStatus;

static USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

pub fn post_ubisoft_rememberme(
    agent: &Agent,
    ticket: &str,
) -> Result<(String, String), ureq::Error> {
    let body = agent
        .post("https://public-ubiservices.ubi.com/v3/profiles/sessions")
        .set("User-Agent", USER_AGENT)
        .set("Ubi-AppId", "f2f8f582-6b7b-4d87-9a19-c72f07fccf99")
        .set("Ubi-RequestedPlatformType", "uplay")
        .set("Authorization", &format!("rm_v1 t={}", ticket))
        .set("Content-Type", "application/json")
        .send_json(ureq::json!({
            "rememberMe": true,
        }))?;

    let json: Value = body.into_json()?;
    Ok((
        json["ticket"].as_str().unwrap().to_string(),
        json["profileId"].as_str().unwrap().to_string(),
    ))
}

pub fn post_ubisoft_2fa_ticket(
    agent: &Agent,
    ticket: &str,
    token: &str,
) -> Result<(String, String), ureq::Error> {
    let body = agent
        .post("https://public-ubiservices.ubi.com/v3/profiles/sessions")
        .set("User-Agent", USER_AGENT)
        .set("Ubi-AppId", "f2f8f582-6b7b-4d87-9a19-c72f07fccf99")
        .set("Ubi-RequestedPlatformType", "uplay")
        .set("Ubi-2faCode", &format!("{}", token))
        .set("Authorization", &format!("ubi_2fa_v1 t={}", ticket))
        .set("Content-Type", "application/json")
        .send_json(ureq::json!({
            "rememberMe": true,
        }))?;

    let json: Value = body.into_json()?;
    let ticket = &json["ticket"];
    if json.get("rememberMeTicket").is_some() {
        let remember_me_ticket = json["rememberMeTicket"].as_str().unwrap().to_string();
        let ticket = post_ubisoft_rememberme(&agent, &remember_me_ticket)?;
        return Ok(ticket);
    }
    Ok((
        ticket.as_str().unwrap().to_string(),
        json["profileId"].as_str().unwrap().to_string(),
    ))
}

pub fn get_ubisoft_game_token(agent: &Agent, token: &str) -> Result<(String, String), ureq::Error> {
    let body = agent
        .post("https://public-ubiservices.ubi.com/v3/profiles/sessions")
        .set("User-Agent", USER_AGENT)
        .set("Authorization", &format!("Ubi_v1 t={}", token))
        .set("Ubi-AppId", "f2f8f582-6b7b-4d87-9a19-c72f07fccf99")
        .set("Ubi-RequestedPlatformType", "uplay")
        .set("Content-Type", "application/json")
        .send_json(ureq::json!({
            "rememberMe": true,
        }))?;

    let json: Value = body.into_json()?;
    Ok((
        json["ticket"].as_str().unwrap().to_string(),
        json["profileId"].as_str().unwrap().to_string(),
    ))
}

pub fn get_ubisoft_session(
    agent: &Agent,
    bot: &Bot,
    email: &str,
    password: &str,
    recovery_code: &str,
) -> Result<(String, String), ureq::Error> {
    let encoded = general_purpose::STANDARD.encode(format!("{}:{}", email, password));
    let body = agent
        .post("https://public-ubiservices.ubi.com/v3/profiles/sessions")
        .set("User-Agent", USER_AGENT)
        .set("Authorization", &format!("Basic {}", encoded))
        .set("Ubi-AppId", "afb4b43c-f1f7-41b7-bcef-a635d8c83822")
        .set("Ubi-RequestedPlatformType", "uplay")
        .set("Content-Type", "application/json")
        .send_json(ureq::json!({
            "rememberMe": true,
        }))?;

    let json: Value = body.into_json()?;
    if json.get("twoFactorAuthenticationTicket").is_some() {
        let ticket = &json["twoFactorAuthenticationTicket"]
            .as_str()
            .unwrap()
            .to_string();
        let token = rust_otp::make_totp(&recovery_code.to_ascii_uppercase(), 30, 0).unwrap();
        match post_ubisoft_2fa_ticket(&agent, ticket, &token.to_string()) {
            Ok(res) => Ok(res),
            Err(err) => {
                {
                    let mut state = bot.state.lock().expect("Failed to lock state");
                    let mut info = bot.info.lock().expect("Failed to lock info");
                    state.is_running = false;
                    info.status = EStatus::TwoFAFailed;
                }
                Err(err)
            }
        }
    } else {
        let game_token =
            get_ubisoft_game_token(agent, json["ticket"].as_str().unwrap().to_string().as_str())?;
        Ok(game_token)
    }
}

fn link_ubisoft_to_steam(
    agent: &Agent,
    session_ticket: &str,
    profile_id: &str,
    steam_ticket: &str,
) -> Result<(), ureq::Error> {
    agent
        .post(
            format!(
                "https://public-ubiservices.ubi.com/v2/users/{}/profiles",
                &profile_id
            )
            .to_string()
            .as_str(),
        )
        .set("User-Agent", USER_AGENT)
        .set(
            "Authorization",
            &format!("steam t={}", &steam_ticket.trim()),
        )
        .set("Ubi-AppId", "afb4b43c-f1f7-41b7-bcef-a635d8c83822")
        .set("Ubi-RequestedPlatformType", "uplay")
        .set("Content-Type", "application/json")
        .send_json(ureq::json!({
            "otherTicket": &session_ticket,
        }))?;

    Ok(())
}

pub fn get_ubisoft_token(
    bot: &Bot,
    recovery_code: &str,
    email: &str,
    password: &str,
    steamuser: &str,
    steampassword: &str,
) -> Result<String, error::CustomError> {
    let info = {
        let data = bot.info.lock().unwrap().login_info.to_string();
        data.clone()
    };
    let agent = ureq::AgentBuilder::new().redirects(5).build();
    let (session, profile_id) =
        match get_ubisoft_session(&agent, bot, email, password, recovery_code) {
            Ok(res) => res,
            Err(err) => {
                if err.to_string().contains("code 401") {
                    bot.state.lock().unwrap().is_running = false;
                    bot.info.lock().unwrap().status = EStatus::Unauthorized;
                }
                return Err(error::CustomError::Other(format!(
                    "Failed to get ubisoft session: {}",
                    err
                )));
            }
        };

    let current_dir = env::current_dir().expect("Failed to get current directory");
    let executable_path = if cfg!(windows) {
        current_dir.join("steamtoken.exe")
    } else {
        current_dir.join("steamtoken")
    };
    let timeout = Duration::from_secs(10);

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
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    let data = output_str.split("\n").collect::<Vec<&str>>();
                    let steam_token = utils::textparse::format_string_as_steam_token(&data[0]);

                    let formated =
                        encode(format!("UbiTicket|{}{}\n", &session, info).as_str()).to_string();
                    let body = agent
                        .post("https://login.growtopiagame.com/player/login/dashboard?valKey=40db4045f2d8c572efe8c4a060605726")
                        .set("user-agent", USER_AGENT)
                        .send_string(format!("{}steamToken%7C{}.240", formated, steam_token.trim_end()).as_str())?;

                    let body_str = body.into_string().unwrap();
                    let json: Value = match serde_json::from_str(&body_str) {
                        Ok(json) => json,
                        Err(err) => {
                            return Err(error::CustomError::Other(format!(
                                "Failed to parse json: {}",
                                err
                            )));
                        }
                    };

                    if json.get("message").is_some() {
                        let message = json["message"].as_str().unwrap().to_string();
                        if message.contains(
                            "Please try login with Steam account connected with Ubisoft Connect.",
                        ) {
                            warn!("Linking Ubisoft to Steam");
                            return match link_ubisoft_to_steam(
                                &agent,
                                &session,
                                &profile_id,
                                &data[1],
                            ) {
                                Ok(_token) => {
                                    info!("Successfully linked Ubisoft to Steam");
                                    Ok("".to_string())
                                }
                                Err(err) => Err(error::CustomError::Other(format!(
                                    "Failed to link Ubisoft to Steam: {}",
                                    err
                                ))),
                            };
                        }
                    }
                    return Ok(json["token"].to_string());
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
    let agent = ureq::AgentBuilder::new().redirects(0).build();
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

    if req.status() == 200 {
        let body = req.into_string()?;
        let json: Value = serde_json::from_str(&body).unwrap();
        Ok(json["token"].as_str().unwrap().to_string())
    } else {
        // 302 is possible invalid credentials redirection
        Err(ureq::Error::Status(
            403,
            ureq::Response::new(403, "forbidden", "invalid_credentials")?,
        ))
    }
}

pub fn extract_token_from_html(body: &str) -> Option<String> {
    let regex = Regex::new(r#"name="_token"\s+type="hidden"\s+value="([^"]*)""#).unwrap();
    regex
        .captures(body)
        .and_then(|cap| cap.get(1).map(|match_| match_.as_str().to_string()))
}
