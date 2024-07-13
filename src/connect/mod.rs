use std::{collections::HashMap, error::Error};

use byteorder::{ByteOrder, LittleEndian};
use enet::*;
use headless_chrome::Browser;
use spdlog::{info, warn};

static USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0";

pub struct Connect {
    parsed_server_data: HashMap<String, String>,
}

impl Connect {
    pub fn new() -> Connect {
        Connect {
            parsed_server_data: HashMap::new(),
        }
    }
}

impl Connect {
    pub fn to_http(&mut self) {
        let req = ureq::post("https://www.growtopia1.com/growtopia/server_data.php").set(
            "User-Agent",
            "UbiServices_SDK_2022.Release.9_PC64_ansi_static",
        );

        let res = req.send_string("").unwrap();

        let body = res.into_string().unwrap();
        self.parse_server_data(body);
    }
    pub fn to_enet(&self) {
        let enet = Enet::new().expect("Failed to initialize ENet");

        let mut enet_host = Enet::create_host::<()>(
            &enet,
            None,
            1,
            ChannelLimit::Limited(1),
            BandwidthLimit::Unlimited,
            BandwidthLimit::Unlimited,
            true,
        )
        .expect("Failed to create ENet host");

        info!(
            "Connecting to {}:{}",
            self.parsed_server_data["server"], self.parsed_server_data["port"]
        );
        let server = self.parsed_server_data["server"]
            .parse()
            .expect("Failed to parse server address");
        let port = self.parsed_server_data["port"]
            .parse()
            .expect("Failed to parse server port");
        let mut enet_peer = enet_host
            .connect(&Address::new(server, port), 2, 0)
            .expect("Failed to connect to the server");

        loop {
            match enet_host.service(1000).expect("Service failed") {
                Some(Event::Connect(_)) => info!("Connected to the server"),
                Some(Event::Disconnect(..)) => info!("Disconnected from the server"),
                Some(Event::Receive { ref packet, .. }) => {
                    let data = packet.data();
                    let packet_id = LittleEndian::read_u32(&data[0..4]);
                    info!("Received packet with ID: {}", packet_id);
                }
                _ => (),
            }
        }
    }
    pub fn parse_server_data(&mut self, data: String) {
        self.parsed_server_data = data
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
