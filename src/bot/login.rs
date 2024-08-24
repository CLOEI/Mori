use paris::{error, info};
use regex::Regex;
use serde_json::Value;
use std::{io, process::Command, time::Duration};

static USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

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
