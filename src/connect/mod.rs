use fantoccini::{ClientBuilder, Locator};

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
    #[tokio::main]
    pub async fn get_legacy_token(
        &self,
        url: &str,
        username: &str,
        password: &str,
    ) -> Result<String, fantoccini::error::CmdError> {
        let c = ClientBuilder::native()
            .connect("http://localhost:4444")
            .await
            .expect("failed to connect to WebDriver");
        c.set_ua(USER_AGENT).await?;
        println!("{}", url);
        c.goto(url).await?;
        c.find(Locator::Css("input#login-name"))
            .await?
            .send_keys(username)
            .await?;
        c.find(Locator::Css("input#password"))
            .await?
            .send_keys(password)
            .await?;
        c.find(Locator::Css("input.btn")).await?.click().await?;

        let html = c.source().await?;

        println!("{}", html);
        c.close().await?;
        Ok(html)
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
