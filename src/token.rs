use headless_chrome::{Browser, LaunchOptions};
use serde_json::Value;

pub fn fetch(_bot_name: String, url: String) -> String {
    let launch_options = LaunchOptions {
        headless: false,
        ..Default::default()
    };
    
    let browser = Browser::new(launch_options).unwrap();
    let tab = browser.new_tab().unwrap();
    tab.enable_stealth_mode().unwrap();
    tab.navigate_to(&url).unwrap();

    while !tab.get_url().contains("validate") {}
    
    if tab.get_url().contains("validate") {
        if let Ok(body_element) = tab.find_element("body") {
            if let Ok(body_text) = body_element.get_inner_text() {
                match serde_json::from_str::<Value>(&body_text) {
                    Ok(json) => {
                        if let Some(token_value) = json.get("token") {
                            if let Some(token) = token_value.as_str() {
                                println!("token: {}", token);
                                return token.to_string();
                            }
                        }
                    }
                    Err(e) => {
                        println!("Failed to parse JSON: {}", e);
                    }
                }
            }
        }
    }
    
    return "".to_string();
}