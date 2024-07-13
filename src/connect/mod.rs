use std::error::Error;

use headless_chrome::{Browser, LaunchOptionsBuilder};

static USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0";

pub struct Connect {
    token: String,
}

impl Connect {
    pub fn new() -> Connect {
        Connect {
            token: String::new(),
        }
    }
}

impl Connect {
    pub fn get_apple_token(&self, url: &str, username: &str, password: &str) {
        println!("Getting apple token");
    }
    pub fn get_google_token(&self, url: &str, username: &str, password: &str) {
        println!("Getting google token");
    }
    #[warn(unused_must_use)]
    pub fn get_legacy_token(
        &self,
        url: &str,
        username: &str,
        password: &str,
    ) -> Result<String, Box<dyn Error>> {
        let browser = Browser::default()?;
        let tab = browser.new_tab()?;

        tab.navigate_to(url)?;
        tab.wait_until_navigated()?;
        tab.wait_for_element("input#login-name")?
            .type_into(username)?;
        tab.wait_for_element("input#password")?
            .type_into(password)?;
        tab.wait_for_element("input.btn")?.click()?;
        tab.wait_until_navigated()?;

        let body = tab.wait_for_element("body")?.get_inner_text()?;
        let parsed = json::parse(&body)?;

        Ok(parsed["token"].to_string())
    }
}

pub fn get_oauth_links() -> Result<Vec<String>, ureq::Error> {
    let body = ureq::post("https://login.growtopiagame.com/player/login/dashboard")
        .set("User-Agent", USER_AGENT)
        .send_string("requestedName%7C%0Af%7C1%0Aprotocol%7C208%0Agame_version%7C4.61%0A")?
        .into_string()?;

    let pattern = regex::Regex::new("https:\\/\\/login\\.growtopiagame\\.com\\/(apple|google|player\\/growid)\\/(login|redirect)\\?token=[^\"]+");
    let links = pattern
        .unwrap()
        .find_iter(&body)
        .map(|m| m.as_str().to_owned())
        .collect::<Vec<String>>();

    Ok(links)
}
