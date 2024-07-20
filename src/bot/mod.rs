mod astar;
mod inventory;
pub mod login;
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
use std::sync::Arc;

use astar::AStar;
use byteorder::{ByteOrder, LittleEndian};
use enet::*;
use gtitem_r::structs::ItemDatabase;
use gtworld_r::World;
use inventory::Inventory;
use spdlog::info;

static USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0";

pub struct Server {
    ip: String,
    port: String,
}

pub struct Bot {
    pub display_name: String,
    pub username: String,
    pub password: String,
    pub code: String,
    method: ELoginMethod,
    oauth_links: Vec<String>,
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
    pub world: World,
    pub inventory: Inventory,
    pub astar: AStar,
}

impl Bot {
    pub fn new(
        username: String,
        password: String,
        code: String,
        method: ELoginMethod,
        item_database: Arc<ItemDatabase>,
    ) -> Bot {
        Bot {
            display_name: String::new(),
            username: username,
            password: password,
            code: code,
            method: method,
            oauth_links: Vec::new(),
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
            world: World::new(Arc::clone(&item_database)),
            inventory: Inventory::new(),
            astar: AStar::new(Arc::clone(&item_database)),
        }
    }
}

impl Bot {
    pub fn login(&mut self) {
        self.to_http();
        if self.method != ELoginMethod::UBISOFT {
            match self.get_oauth_links() {
                Ok(links) => {
                    self.oauth_links = links;
                    info!("Successfully got OAuth links for: apple, google and legacy");
                }
                Err(err) => {
                    info!("Failed to get OAuth links: {}", err);
                    return;
                }
            }
        }
        self.get_token();
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
        // TODO: Handle error, loop with delay until token is received
        info!("Getting token for {}", self.username);
        match self.method {
            ELoginMethod::UBISOFT => {
                let res =
                    login::get_ubisoft_token(&self.username, &self.password, &self.code).unwrap();
                self.token = res;
            }
            ELoginMethod::APPLE => {
                let res = login::get_apple_token(self.oauth_links[0].as_str()).unwrap();
                self.token = res;
            }
            ELoginMethod::GOOGLE => {
                let res = login::get_google_token(self.oauth_links[1].as_str()).unwrap();
                self.token = res;
            }
            ELoginMethod::LEGACY => {
                let res = login::get_legacy_token(
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

    pub fn to_http(&mut self) {
        let req = ureq::post("https://www.growtopia1.com/growtopia/server_data.php").set(
            "User-Agent",
            "UbiServices_SDK_2022.Release.9_PC64_ansi_static",
        );

        let res = req.send_string("").unwrap();

        let body = res.into_string().unwrap();
        self.parse_server_data(body);
    }

    pub fn find_path(&mut self, peer: &mut Peer<()>, x: u32, y: u32) {
        let paths =
            match self
                .astar
                .find_path((self.pos_x as u32) / 32, (self.pos_y as u32) / 32, x, y)
            {
                Some(path) => path,
                None => return,
            };

        for i in 0..paths.len() {
            let node = &paths[i];
            self.walk(peer, node.x as f32, node.y as f32, true);
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

    // ap = absolute path, should be self explanatory
    pub fn walk(&mut self, peer: &mut Peer<()>, x: f32, y: f32, ap: bool) {
        if ap {
            self.pos_x = x * 32.0;
            self.pos_y = y * 32.0;
        } else {
            self.pos_x += x * 32.0;
            self.pos_y += y * 32.0;
        }

        let mut pkt = TankPacketType::new();
        let mut flags: u32 = 0;
        flags |= 1 << 1; // unknown
        flags |= 1 << 5; // is on a solid block

        pkt.packet_type = ETankPacketType::NetGamePacketState;
        pkt.vector_x = self.pos_x;
        pkt.vector_y = self.pos_y;
        pkt.flags = flags;
        pkt.int_x = -1;
        pkt.int_y = -1;

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
        packet_data.extend_from_slice(&pkt.value.to_le_bytes());
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

    pub fn talk(&mut self, peer: &mut Peer<()>, message: &str) {
        self.send_packet(
            peer,
            EPacketType::NetMessageGenericText,
            format!("action|input\n|text|{}\n", message),
        );
    }

    pub fn place(&mut self, peer: &mut Peer<()>, offset_x: i32, offset_y: i32, block_id: u32) {
        let mut pkt = TankPacketType::new();

        pkt.packet_type = ETankPacketType::NetGamePacketTileChangeRequest;
        pkt.vector_x = self.pos_x;
        pkt.vector_y = self.pos_y;
        pkt.int_x = ((self.pos_x / 32.0).floor() as i32) + offset_x;
        pkt.int_y = ((self.pos_y / 32.0).floor() as i32) + offset_y;
        pkt.value = block_id;

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
        packet_data.extend_from_slice(&pkt.value.to_le_bytes());
        packet_data.extend_from_slice(&pkt.vector_x.to_le_bytes());
        packet_data.extend_from_slice(&pkt.vector_y.to_le_bytes());
        packet_data.extend_from_slice(&pkt.vector_x2.to_le_bytes());
        packet_data.extend_from_slice(&pkt.vector_y2.to_le_bytes());
        packet_data.extend_from_slice(&pkt.unk12.to_le_bytes());
        packet_data.extend_from_slice(&pkt.int_x.to_le_bytes());
        packet_data.extend_from_slice(&pkt.int_y.to_le_bytes());
        packet_data.extend_from_slice(&pkt.extended_data_length.to_le_bytes());

        if pkt.int_x <= (self.pos_x / 32.0).floor() as i32 + 4
            && pkt.int_x >= (self.pos_x / 32.0).floor() as i32 - 4
            && pkt.int_y <= (self.pos_y / 32.0).floor() as i32 + 4
            && pkt.int_y >= (self.pos_y / 32.0).floor() as i32 - 4
        {
            let pkt = Packet::new(&packet_data, PacketMode::ReliableSequenced).unwrap();
            peer.send_packet(pkt, 0).unwrap();
        }
    }

    pub fn punch(&mut self, peer: &mut Peer<()>, offset_x: i32, offset_y: i32) {
        self.place(peer, offset_x, offset_y, 18)
    }

    pub fn warp(&mut self, peer: &mut Peer<()>, world: &str) {
        self.send_packet(
            peer,
            EPacketType::NetMessageGameMessage,
            format!("action|join_request\nname|{}\ninvitedWorld|0\n", world),
        );
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

    pub fn get_oauth_links(&self) -> Result<Vec<String>, ureq::Error> {
        let body = ureq::post("https://login.growtopiagame.com/player/login/dashboard")
            .set("User-Agent", USER_AGENT)
            .send_string(format!("tankIDName|\ntankIDPass|\nrequestedName|BoardSickle\nf|1\nprotocol|209\ngame_version|4.62\nfz|41745432\nlmode|0\ncbits|1040\nplayer_age|20\nGDPR|3\ncategory|_-5100\ntotalPlaytime|0\nklv|b351d8dacd7a776848b31c74d3d550ec61dbb9b96c3ac67aea85034a84401a87\nhash2|841545814\nmeta|{}\nfhash|-716928004\nrid|01F9EBD204B52C940285667E15C00D62\nplatformID|0,1,1\ndeviceVersion|0\ncountry|us\nhash|-1829975549\nmac|b4:8c:9d:90:79:cf\nwk|66A6ABCD9753A066E39975DED77852A8\nzf|617169524\n", self.parsed_server_data["meta"]).as_str())?
            .into_string()?;

        let pattern = regex::Regex::new("https:\\/\\/login\\.growtopiagame\\.com\\/(apple|google|player\\/growid)\\/(login|redirect)\\?token=[^\"]+");
        let links = pattern
            .unwrap()
            .find_iter(&body)
            .map(|m| m.as_str().to_owned())
            .collect::<Vec<String>>();

        Ok(links)
    }
}
