use headless_chrome::{Browser, LaunchOptionsBuilder};
use regex::Regex;
use std::{collections::HashMap, error::Error};

use spdlog::info;

use crate::bot::Bot;
use crate::types::e_login_method::ELoginMethod;

static USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0";

pub struct Connect {
    username: String,
    password: String,
    method: ELoginMethod,
    oauth_links: Vec<String>,
    bot: Bot,
}

impl Connect {
    pub fn new(
        username: String,
        password: String,
        method: ELoginMethod,
        oauth_links: Vec<String>,
    ) -> Connect {
        Connect {
            username,
            password,
            method,
            oauth_links,
            bot: Bot::new(),
        }
    }
}

impl Connect {
    pub fn start(&mut self) {
        info!("Getting token for {}", self.username);
        match self.method {
            ELoginMethod::APPLE => {
                self.get_apple_token(self.oauth_links[0].as_str());
            }
            ELoginMethod::GOOGLE => {
                self.get_google_token(self.oauth_links[1].as_str());
            }
            ELoginMethod::LEGACY => {
                let res = self
                    .get_legacy_token(
                        self.oauth_links[2].as_str(),
                        self.username.as_str(),
                        self.password.as_str(),
                    )
                    .unwrap();
                self.bot.token = res;
            }
        }
        info!("Received the token: {}", self.bot.token);
        self.to_http();
        self.bot.login();
    }

    pub fn to_http(&mut self) {
        let req = ureq::post("https://www.growtopia1.com/growtopia/server_data.php").set(
            "User-Agent",
            "UbiServices_SDK_2022.Release.9_PC64_ansi_static",
        );

        let res = req.send_string("").unwrap();

        let body = res.into_string().unwrap();
        self.parse_server_data(body);
    }

    pub fn parse_server_data(&mut self, data: String) {
        self.bot.parsed_server_data = data
            .lines()
            .filter_map(|line| {
                let mut parts = line.splitn(2, '|');
                match (parts.next(), parts.next()) {
                    (Some(key), Some(value)) => Some((key.to_string(), value.to_string())),
                    _ => None,
                }
            })
            .collect::<HashMap<String, String>>();
    }

    pub fn get_apple_token(&self, url: &str) {
        println!("Getting apple token");
    }

    pub fn get_google_token(&self, url: &str) {
        println!("Getting google token");
    }

    pub fn get_legacy_token(
        &self,
        url: &str,
        username: &str,
        password: &str,
    ) -> Result<String, ureq::Error> {
        let agent = ureq::AgentBuilder::new().build();
        let body = agent.get(url)
            .set("User-Agent", USER_AGENT)
            .set("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
            .set("Accept-Language", "en-US,en;q=0.5")
            .set("Accept-Encoding", "gzip, deflate, br, zstd")
            .set("DNT", "1")
            .set("Sec-GPC", "1")
            .set("Connection", "keep-alive")
            .set("Upgrade-Insecure-Requests", "1")
            .set("Sec-Fetch-Dest", "document")
            .set("Sec-Fetch-Mode", "navigate")
            .set("Sec-Fetch-Site", "none")
            .set("Sec-Fetch-User", "?1")
            .set("Sec-CH-UA-Platform", "Windows")
            .set("Sec-CH-UA", "\"Edge\";v=\"120\", \"Chromium\";v=\"120\", \"Not=A?Brand\";v=\"24\"")
            .set("Sec-CH-UA-Mobile", "?0")
            .set("Priority", "u=1")
            .set("TE", "trailers").call()?.into_string()?;

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
        let json = json::parse(&body).unwrap();
        Ok(json["token"].to_string())
    }
}

pub fn extract_token_from_html(body: &str) -> Option<String> {
    let regex = Regex::new(r#"name="_token"\s+type="hidden"\s+value="([^"]*)""#).unwrap();
    regex
        .captures(body)
        .and_then(|cap| cap.get(1).map(|match_| match_.as_str().to_string()))
}

pub fn get_oauth_links() -> Result<Vec<String>, ureq::Error> {
    let body = ureq::post("https://login.growtopiagame.com/player/login/dashboard")
        .set("User-Agent", USER_AGENT)
        .send_string("requestedName|\nprotocol|208\ngame_version|4.61\n")?
        .into_string()?;

    let pattern = regex::Regex::new("https:\\/\\/login\\.growtopiagame\\.com\\/(apple|google|player\\/growid)\\/(login|redirect)\\?token=[^\"]+");
    let links = pattern
        .unwrap()
        .find_iter(&body)
        .map(|m| m.as_str().to_owned())
        .collect::<Vec<String>>();

    Ok(links)
}
