mod packet_handler;
mod variant_handler;

use crate::types::e_login_method::ELoginMethod;
use crate::types::e_tank_packet_type::ETankPacketType;
use crate::types::login_info::LoginInfo;
use crate::types::tank_packet_type::TankPacketType;
use crate::utils::proton::hash_string;
use crate::utils::random::random_hex;
use crate::{types::e_packet_type::EPacketType, utils::proton::generate_klv};

use std::collections::HashMap;
use std::io;
use std::process::Command;

use byteorder::{ByteOrder, LittleEndian};
use enet::*;
use regex::Regex;
use spdlog::info;

static USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0";

pub struct Server {
    ip: String,
    port: String,
}

pub struct Bot {
    pub username: String,
    pub password: String,
    method: ELoginMethod,
    oauth_links: Vec<String>,
    pub current_world: String,
    pub net_id: u32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub gems: i32,
    pub parsed_server_data: HashMap<String, String>,
    pub token: String,
    pub is_redirect: bool,
    pub is_running: bool,
    pub is_banned: bool,
    pub is_ingame: bool,
    pub server: Server,
    pub login_info: LoginInfo,
}

impl Bot {
    pub fn new(
        username: String,
        password: String,
        method: ELoginMethod,
        oauth_links: Vec<String>,
    ) -> Bot {
        Bot {
            username: username,
            password: password,
            method: method,
            oauth_links: oauth_links,
            current_world: String::new(),
            net_id: 0,
            pos_x: 0.0,
            pos_y: 0.0,
            gems: 0,
            parsed_server_data: HashMap::new(),
            token: String::new(),
            is_redirect: false,
            is_running: false,
            is_banned: false,
            is_ingame: false,
            server: Server {
                ip: String::new(),
                port: String::new(),
            },
            login_info: LoginInfo::new(),
        }
    }
}

impl Bot {
    pub fn login(&mut self) {
        self.get_token();
        self.to_http();
        self.is_running = true;
        self.login_info.meta = self.parsed_server_data["meta"].clone();

        self.login_info.klv = generate_klv(
            &self.login_info.protocol,
            &self.login_info.game_version,
            &self.login_info.rid,
        );
        self.login_info.hash =
            hash_string(format!("{}RT", self.login_info.mac).as_str()).to_string();
        self.login_info.hash2 =
            hash_string(format!("{}RT", random_hex(16, true)).as_str()).to_string();

        let enet = Enet::new().expect("Failed to initialize ENet");

        while self.is_running {
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
            if self.is_redirect {
                info!("Redirecting to {}:{}...", self.server.ip, self.server.port);
                enet_host
                    .connect(
                        &Address::new(
                            self.server.ip.parse().unwrap(),
                            self.server.port.parse().unwrap(),
                        ),
                        2,
                        0,
                    )
                    .expect("Failed to connect to the server");
            } else {
                if self.is_ingame {
                    self.get_token();
                }
                self.to_http();
                info!(
                    "Connecting to {}:{}",
                    self.parsed_server_data["server"], self.parsed_server_data["port"]
                );
                enet_host
                    .connect(
                        &Address::new(
                            self.parsed_server_data["server"].parse().unwrap(),
                            self.parsed_server_data["port"].parse().unwrap(),
                        ),
                        2,
                        0,
                    )
                    .expect("Failed to connect to the server");
            }
            self.is_ingame = false;
            loop {
                match enet_host.service(1000).expect("Service failed") {
                    Some(Event::Connect(..)) => {
                        info!("Connected to the server");
                    }
                    Some(Event::Disconnect(..)) => {
                        info!("Disconnected from the server");
                        break;
                    }
                    Some(Event::Receive {
                        ref packet,
                        ref mut sender,
                        ..
                    }) => {
                        let data = packet.data();
                        if data.len() < 4 {
                            continue;
                        }
                        let packet_id = LittleEndian::read_u32(&data[0..4]);
                        let packet_type = EPacketType::from(packet_id);
                        packet_handler::handle(self, sender, packet_type, &data[4..]);
                    }
                    _ => (),
                }
            }
        }
    }

    pub fn get_token(&mut self) {
        info!("Getting token for {}", self.username);
        match self.method {
            ELoginMethod::APPLE => {
                let res = self.get_apple_token(self.oauth_links[0].as_str()).unwrap();
                self.token = res;
            }
            ELoginMethod::GOOGLE => {
                let res = self.get_google_token(self.oauth_links[1].as_str()).unwrap();
                self.token = res;
            }
            ELoginMethod::LEGACY => {
                let res = self
                    .get_legacy_token(
                        self.oauth_links[2].as_str(),
                        self.username.as_str(),
                        self.password.as_str(),
                    )
                    .unwrap();
                self.token = res;
            }
        }
        info!("Received the token: {}", self.token);
    }

    pub fn get_apple_token(&self, url: &str) -> Result<String, std::io::Error> {
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

    pub fn get_google_token(&self, url: &str) -> Result<String, std::io::Error> {
        println!("Getting google token");
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
        let data = json::parse(&buffer).unwrap();
        Ok(data["token"].to_string())
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

    pub fn walk(&mut self, peer: &mut Peer<()>, x: f32, y: f32) {
        self.pos_x += x * 32.0;
        self.pos_y += y * 32.0;

        let mut pkt = TankPacketType::new();
        let mut flags: Vec<u8> = Vec::with_capacity(4);
        flags[1] = true as u8; // unknown
        flags[5] = true as u8; // is on a solid block

        pkt.packet_type = ETankPacketType::NetGamePacketState;
        pkt.vector_x = self.pos_x;
        pkt.vector_y = self.pos_y;
        pkt.flags = LittleEndian::read_u32(&flags);
        pkt.int_x = u32::MAX;
        pkt.int_y = u32::MAX;

        let mut packet_data = Vec::new();
        packet_data.extend_from_slice(&(EPacketType::NetMessageGamePacket as u32).to_le_bytes());
        packet_data.extend_from_slice(&(pkt.packet_type as u8).to_le_bytes());
        packet_data.extend_from_slice(&pkt.unk1.to_le_bytes());
        packet_data.extend_from_slice(&pkt.unk2.to_le_bytes());
        packet_data.extend_from_slice(&pkt.unk3.to_le_bytes());
        packet_data.extend_from_slice(&pkt.net_id.to_le_bytes());
        packet_data.extend_from_slice(&pkt.unk4.to_le_bytes());
        packet_data.extend_from_slice(&pkt.flags.to_le_bytes());
        packet_data.extend_from_slice(&pkt.unk6.to_le_bytes());
        packet_data.extend_from_slice(&pkt.unk7.to_le_bytes());
        packet_data.extend_from_slice(&pkt.vector_x.to_le_bytes());
        packet_data.extend_from_slice(&pkt.vector_y.to_le_bytes());
        packet_data.extend_from_slice(&pkt.vector_x2.to_le_bytes());
        packet_data.extend_from_slice(&pkt.vector_y2.to_le_bytes());
        packet_data.extend_from_slice(&pkt.unk12.to_le_bytes());
        packet_data.extend_from_slice(&pkt.int_x.to_le_bytes());
        packet_data.extend_from_slice(&pkt.int_y.to_le_bytes());
        packet_data.extend_from_slice(&pkt.extended_data_length.to_le_bytes());

        let pkt = Packet::new(&packet_data, PacketMode::ReliableSequenced).unwrap();
        peer.send_packet(pkt, 0).unwrap();
    }

    pub fn send_packet(&self, peer: &mut Peer<()>, packet_type: EPacketType, message: String) {
        if packet_type == EPacketType::NetMessageGamePacket {
            // TODO: Implement this
        } else {
            let mut packet_data = Vec::new();
            packet_data.extend_from_slice(&(packet_type as u32).to_le_bytes());
            packet_data.extend_from_slice(&message.as_bytes());
            let pkt = Packet::new(&packet_data, PacketMode::ReliableSequenced).unwrap();
            peer.send_packet(pkt, 0).unwrap();
        }
    }

    pub fn disconnect(&self, peer: &mut Peer<()>) {
        peer.disconnect(0);
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
