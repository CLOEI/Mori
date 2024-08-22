use chromiumoxide::{Browser, BrowserConfig, Page};
use futures::StreamExt;
use paris::info;
use regex::Regex;
use serde_json::Value;
use std::{io, process::Command};

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

#[tokio::main]
pub async fn get_google_token(
    email: &str,
    password: &str,
    url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let (mut browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .with_head()
            .args(vec![
                "--excludeSwitches=enable-automation",
                "--disable-blink-features=AutomationControlled",
                "--lang=en-EN",
                "--window-size=1920,1080",
                &format!("--user-agent={}", USER_AGENT),
            ])
            .build()?,
    )
    .await?;

    let handle = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() {
                break;
            }
        }
    });

    let page = browser.new_page(url).await?;
    page.enable_stealth_mode().await?;
    match page
        .find_xpath(format!("//li/div[@data-identifier='{}']", email))
        .await
    {
        Ok(elem) => {
            elem.click().await?;
        }
        Err(..) => match page.find_xpath("//*[@id=\"identifierId\"]").await {
            Ok(..) => {
                handle_google_login_form(email, password, &page).await?;
            }
            Err(..) => {
                page.find_xpath("//li/div[not(@data-identifier)]")
                    .await?
                    .click()
                    .await?;
                page.wait_for_navigation_response().await?;
                handle_google_login_form(email, password, &page).await?;
            }
        },
    };
    page.wait_for_navigation_response().await?;
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    let source = page
        .find_element("body")
        .await?
        .inner_text()
        .await?
        .unwrap();
    if source.contains("too many people") {
        return Err("Too many people trying to login".into());
    }
    let json: Value = serde_json::from_str(&source).unwrap();

    browser.close().await?;
    handle.await?;
    Ok(json["token"].to_string())
}

async fn handle_google_login_form(
    email: &str,
    password: &str,
    page: &Page,
) -> Result<(), Box<dyn std::error::Error>> {
    page.find_xpath("//*[@id=\"identifierId\"]")
        .await?
        .type_str(email)
        .await?;

    page.find_xpath("//*[@id=\"identifierNext\"]/div/button/span")
        .await?
        .click()
        .await?;
    page.wait_for_navigation_response().await?;

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    page.find_xpath("//*[@id=\"password\"]/div[1]/div/div[1]/input")
        .await?
        .type_str(password)
        .await?;
    page.find_xpath("//*[@id=\"passwordNext\"]/div/button/span")
        .await?
        .click()
        .await?;
    page.wait_for_navigation_response().await?;

    Ok(())
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
