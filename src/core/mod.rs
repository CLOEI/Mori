mod astar;
pub mod features;
mod inventory;
mod login;
mod packet_handler;
mod proxy;
mod variant_handler;

use astar::AStar;
use byteorder::{ByteOrder, LittleEndian};
use gtitem_r::structs::ItemDatabase;
use inventory::Inventory;
use mlua::prelude::*;
use rusty_enet as enet;
use socks::Socks5Datagram;
use std::collections::HashMap;
use std::mem::size_of;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::str::{self, FromStr};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, RwLock};
use std::{thread, time::Duration, vec};
use urlencoding::encode;

use crate::core::proxy::{SocketType, Socks5UdpSocket};
use crate::manager::proxy_manager::ProxyManager;
use crate::types::bot_info::{EStatus, TemporaryData, FTUE};
use crate::types::{etank_packet_type::ETankPacketType, player::Player, tank_packet::TankPacket};
use crate::utils::safe_check;
use crate::{
    lua_register, types,
    types::{
        bot_info::{Info, Server, State},
        elogin_method::ELoginMethod,
        epacket_type::EPacketType,
        login_info::LoginInfo,
        vector::Vector2,
    },
    utils,
    utils::{
        config, logging,
        proton::{self},
        random::{self},
    },
};

static USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0";

pub struct Bot {
    pub info: Mutex<Info>,
    pub state: Mutex<State>,
    pub server: Mutex<Server>,
    pub position: Mutex<Vector2>,
    pub temporary_data: RwLock<TemporaryData>,
    pub host: Mutex<enet::Host<SocketType>>,
    pub peer_id: Mutex<Option<enet::PeerID>>,
    pub world: RwLock<gtworld_r::World>,
    pub inventory: Mutex<Inventory>,
    pub players: Mutex<Vec<Player>>,
    pub astar: Mutex<AStar>,
    pub ftue: Mutex<FTUE>,
    pub item_database: Arc<RwLock<ItemDatabase>>,
    pub proxy_manager: Arc<RwLock<ProxyManager>>,
    pub logs: Arc<Mutex<Vec<String>>>,
    pub sender: Sender<String>,
    pub lua: Mutex<Lua>,
    pub agent: ureq::Agent,
}

impl Bot {
    pub fn new(
        bot_config: types::config::BotConfig,
        item_database: Arc<RwLock<ItemDatabase>>,
        proxy_manager: Arc<RwLock<ProxyManager>>,
    ) -> Arc<Self> {
        let lua = Mutex::new(Lua::new());
        let logs = Arc::new(Mutex::new(Vec::new()));
        let (sender, receiver) = std::sync::mpsc::channel();
        let logs_clone = Arc::clone(&logs);
        thread::spawn(move || loop {
            match receiver.recv() {
                Ok(message) => {
                    let mut logs = logs_clone.lock().unwrap();
                    logs.push(message);
                }
                Err(_) => {
                    break;
                }
            }
        });

        let payload = utils::textparse::parse_and_store_as_vec(&bot_config.payload);
        let mut proxy_address: Option<SocketAddr> = None;
        let mut proxy_username = String::new();
        let mut proxy_password = String::new();
        let mut agent = ureq::AgentBuilder::new();

        if config::get_bot_use_proxy(payload[0].clone()) {
            let mut proxy_manager = proxy_manager.write().unwrap();
            if let Some(proxy_index) = proxy_manager
                .proxies
                .iter()
                .position(|proxy| proxy.whos_using.len() < 3)
            {
                if let Some(proxy_data) = proxy_manager.get_mut(proxy_index) {
                    proxy_data.whos_using.push(payload[0].clone());
                    proxy_address = Some(
                        SocketAddr::from_str(&format!(
                            "{}:{}",
                            proxy_data.proxy.ip, proxy_data.proxy.port
                        ))
                        .unwrap(),
                    );
                    proxy_username = proxy_data.proxy.username.clone();
                    proxy_password = proxy_data.proxy.password.clone();
                    logging::info(
                        &format!(
                            "Using proxy: {}:{}:{}:{}",
                            proxy_data.proxy.ip,
                            proxy_data.proxy.port,
                            proxy_username,
                            proxy_password
                        ),
                        &sender,
                    );
                    agent = agent.proxy(
                        ureq::Proxy::new(format!(
                            "socks5://{}:{}@{}:{}",
                            proxy_username,
                            proxy_password,
                            proxy_data.proxy.ip,
                            proxy_data.proxy.port
                        ))
                        .unwrap(),
                    );
                }
            }
        }

        let socket: SocketType = if let Some(proxy) = proxy_address {
            if proxy_username.is_empty() || proxy_password.is_empty() {
                logging::error("Proxy username or password is empty", &sender);
            }
            let udp_datagram = Socks5Datagram::bind_with_password(
                proxy,
                SocketAddr::from_str("0.0.0.0:0").unwrap(),
                &proxy_username,
                &proxy_password,
            )
            .expect("Failed to bind SOCKS5 datagram");
            logging::info("Bound to proxy", &sender);
            SocketType::Socks5(Socks5UdpSocket::new(udp_datagram))
        } else {
            let udp_socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))
                .expect("Failed to bind UDP socket");
            SocketType::Udp(udp_socket)
        };

        let host = enet::Host::<SocketType>::new(
            socket,
            enet::HostSettings {
                peer_limit: 1,
                channel_limit: 2,
                compressor: Some(Box::new(enet::RangeCoder::new())),
                checksum: Some(Box::new(enet::crc32)),
                using_new_packet: true,
                ..Default::default()
            },
        )
        .expect("Failed to create host");

        Arc::new(Self {
            info: Mutex::new(Info {
                payload,
                recovery_code: bot_config.recovery_code,
                login_method: bot_config.login_method,
                token: bot_config.token,
                login_info: LoginInfo::new(),
                ..Default::default()
            }),
            state: Mutex::new(State::default()),
            server: Mutex::new(Server::default()),
            position: Mutex::new(Vector2::default()),
            temporary_data: RwLock::new(TemporaryData::default()),
            host: Mutex::new(host),
            peer_id: Mutex::new(None),
            world: RwLock::new(gtworld_r::World::new(item_database.clone())),
            inventory: Mutex::new(Inventory::new()),
            players: Mutex::new(Vec::new()),
            astar: Mutex::new(AStar::new(item_database.clone())),
            ftue: Mutex::new(FTUE::default()),
            item_database,
            proxy_manager,
            logs,
            sender,
            lua,
            agent: agent.build().clone(),
        })
    }

    pub fn log_info(&self, message: &str) {
        logging::info(message, &self.sender);
    }

    pub fn log_warn(&self, message: &str) {
        logging::warn(message, &self.sender);
    }

    pub fn log_error(&self, message: &str) {
        logging::error(message, &self.sender);
    }

    pub fn logon(self: Arc<Self>, data: String) {
        {
            let lua = self.lua.lock().expect("Failed to lock Lua");
            let _ = lua_register::register(&lua, &self);
        }
        self.set_status(EStatus::Connecting);
        if data.is_empty() {
            self.spoof();
        } else {
            self.update_login_info(data);
        }
        {
            let mut state = self.state.lock().expect("Failed to lock state");
            state.is_running = true;
        }
        poll(Arc::clone(&self));
        self.process_events();
    }

    pub fn set_status(&self, status: EStatus) {
        let mut info = self.info.lock().expect("Failed to lock info");
        info.status = status
    }

    pub fn reconnect(&self) -> bool {
        self.set_status(EStatus::Reconnecting);
        self.to_http();

        let (meta, login_method, oauth_links_empty) = {
            let info = self.info.lock().unwrap();
            (
                info.server_data.get("meta").cloned(),
                info.login_method.clone(),
                info.oauth_links.is_empty(),
            )
        };

        if let Some(meta) = meta {
            let mut info = self.info.lock().unwrap();
            info.login_info.meta = meta;
        }

        if login_method != ELoginMethod::STEAM && oauth_links_empty {
            match self.get_oauth_links() {
                Ok(links) => {
                    let mut info = self.info.lock().unwrap();
                    info.oauth_links = links;
                    self.log_info("Successfully got OAuth links for: apple, google and legacy");
                }
                Err(err) => {
                    self.log_info(&format!("Failed to get OAuth links: {}", err));
                    return false;
                }
            }
        }

        self.get_token();

        {
            let state = self.state.lock().unwrap();
            if !state.is_running {
                return false;
            }
        }

        let (server, port) = {
            let info = self.info.lock().unwrap();
            (
                info.server_data
                    .get("server")
                    .cloned()
                    .expect("Failed to get server"),
                info.server_data
                    .get("port")
                    .cloned()
                    .expect("Failed to get port"),
            )
        };

        self.connect_to_server(&server, &port);
        true
    }

    pub fn connect_bot(self: Arc<Self>) {
        let mut state = self.state.lock().expect("Failed to lock state");
        if state.is_running == false {
            self.log_info("Connecting bot");
            self.set_status(EStatus::Connecting);
            state.is_running = true;
            drop(state);
            self.process_events();
        } else {
            self.log_info("Bot is already connected");
        }
    }

    pub fn disconnect_bot(self: Arc<Self>) {
        let mut state = self.state.lock().expect("Failed to lock state");
        if state.is_running == true {
            self.log_info("Disconnecting bot");
            state.is_running = false;
            state.is_redirecting = false;
            self.set_status(EStatus::Disconnected);
            self.disconnect_now();
            self.log_info("Bot disconnected");
        } else {
            self.log_info("Bot is already disconnected");
        }
    }

    pub fn reconnect_bot(self: Arc<Self>) {
        self.log_info("Reconnecting bot");
        {
            let mut state = self.state.lock().expect("Failed to lock state");
            state.is_running = false;
            state.is_redirecting = false;
        }
        self.set_status(EStatus::Reconnecting);
        self.disconnect_now();
        {
            let mut state = self.state.lock().expect("Failed to lock state");
            state.is_running = true;
        }
        self.process_events();
    }

    fn update_login_info(&self, data: String) {
        self.set_status(EStatus::UpdatingData);
        let mut info = self.info.lock().expect("Failed to lock info");
        let parsed_data = utils::textparse::parse_and_store_as_map(&data);
        for (key, value) in parsed_data {
            match key.as_str() {
                "UUIDToken" => info.login_info.uuid = value.clone(),
                "protocol" => info.login_info.protocol = value.clone(),
                "fhash" => info.login_info.fhash = value.clone(),
                "mac" => info.login_info.mac = value.clone(),
                "requestedName" => info.login_info.requested_name = value.clone(),
                "hash2" => info.login_info.hash2 = value.clone(),
                "fz" => info.login_info.fz = value.clone(),
                "f" => info.login_info.f = value.clone(),
                "player_age" => info.login_info.player_age = value.clone(),
                "game_version" => info.login_info.game_version = value.clone(),
                "lmode" => info.login_info.lmode = value.clone(),
                "cbits" => info.login_info.cbits = value.clone(),
                "rid" => info.login_info.rid = value.clone(),
                "GDPR" => info.login_info.gdpr = value.clone(),
                "hash" => info.login_info.hash = value.clone(),
                "category" => info.login_info.category = value.clone(),
                "token" => info.login_info.token = value.clone(),
                "total_playtime" => info.login_info.total_playtime = value.clone(),
                "door_id" => info.login_info.door_id = value.clone(),
                "klv" => info.login_info.klv = value.clone(),
                "meta" => info.login_info.meta = value.clone(),
                "platformID" => info.login_info.platform_id = value.clone(),
                "deviceVersion" => info.login_info.device_version = value.clone(),
                "zf" => info.login_info.zf = value.clone(),
                "country" => info.login_info.country = value.clone(),
                "user" => info.login_info.user = value.clone(),
                "wk" => info.login_info.wk = value.clone(),
                "tankIDName" => info.login_info.tank_id_name = value.clone(),
                "tankIDPass" => info.login_info.tank_id_pass = value.clone(),
                _ => {}
            }
        }
    }

    fn token_still_valid(&self) -> bool {
        self.log_info("Checking if token is still valid");
        self.set_status(EStatus::CheckingToken);

        let (token, login_info) = {
            let info = self.info.lock().unwrap();
            if info.token.is_empty() {
                return false;
            }
            (info.token.clone(), info.login_info.to_string())
        };

        loop {
            let response = self.agent.post("https://login.growtopiagame.com/player/growid/checktoken?valKey=40db4045f2d8c572efe8c4a060605726")
                .set("User-Agent", "UbiServices_SDK_2022.Release.9_PC64_ansi_static")
                .send_form(&[
                    ("refreshToken", token.as_str()),
                    ("clientData", login_info.as_str()),
                ]);

            match response {
                Ok(res) => {
                    if res.status() != 200 {
                        self.log_error("Failed to refresh token, retrying...");
                        thread::sleep(Duration::from_secs(1));
                        continue;
                    }

                    let response_text = res.into_string().unwrap_or_default();
                    let json_response: serde_json::Value =
                        serde_json::from_str(&response_text).unwrap();

                    if json_response["status"] == "success" {
                        let new_token = json_response["token"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string();
                        self.log_info(&format!("Token is still valid | new token: {}", new_token));

                        let mut info = self.info.lock().unwrap();
                        info.token = new_token;

                        return true;
                    } else {
                        self.log_error("Token is invalid");
                        return false;
                    }
                }
                Err(err) => {
                    self.log_error(&format!("Request error: {}, retrying...", err));
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
            }
        }
    }

    pub fn sleep(&self) {
        loop {
            {
                let mut temp = self.temporary_data.write().unwrap();
                if temp.timeout == 0 {
                    break;
                }
                temp.timeout -= 1;
            }
            thread::sleep(Duration::from_secs(1));
        }
    }

    pub fn get_token(&self) {
        if self.token_still_valid() {
            return;
        }

        self.log_info("Getting token for bot");
        self.set_status(EStatus::GettingToken);
        let (payload, recovery_code, method, oauth_links) = {
            let info = self.info.lock().unwrap();
            (
                info.payload.clone(),
                info.recovery_code.clone(),
                info.login_method.clone(),
                info.oauth_links.clone(),
            )
        };

        let token_result = match method {
            ELoginMethod::GOOGLE => match login::get_google_token(
                oauth_links.get(1).unwrap_or(&"".to_string()),
                &payload[0],
                &payload[1],
            ) {
                Ok(res) => res,
                Err(err) => {
                    if err.to_string().contains("too many people") {
                        self.log_error("Too many people trying to login");
                    } else {
                        self.log_error(&format!("Failed to get Google token: {}", err));
                    }
                    return;
                }
            },
            ELoginMethod::LEGACY => match login::get_legacy_token(
                oauth_links.get(2).unwrap_or(&"".to_string()),
                &payload[0],
                &payload[1],
            ) {
                Ok(res) => res,
                Err(err) => {
                    self.log_error(&format!("Failed to get legacy token: {}", err));
                    return;
                }
            },
            ELoginMethod::STEAM => {
                {
                    let mut info = self.info.lock().unwrap();
                    info.login_info.platform_id = "15,1,0".to_string();
                }
                match login::get_ubisoft_token(
                    self,
                    &recovery_code,
                    &payload[0],
                    &payload[1],
                    &payload[2],
                    &payload[3],
                ) {
                    Ok(res) => res,
                    Err(err) => {
                        self.log_error(&format!("Failed to get Ubisoft token: {}", err));
                        return;
                    }
                }
            }
            _ => {
                self.log_warn("Invalid login method");
                return;
            }
        };

        if !token_result.is_empty() {
            let mut info = self.info.lock().expect("Failed to lock info");
            info.token = token_result;
            self.log_info(&format!("Received the token: {}", info.token));
        }
    }

    pub fn get_oauth_links(&self) -> Result<Vec<String>, ureq::Error> {
        self.log_info("Getting OAuth links");
        self.set_status(EStatus::GettingOAuth);

        let login_info = {
            let info = self.info.lock().unwrap().login_info.to_string();
            let encoded_info = encode(&info).to_string();
            encoded_info
        };

        loop {
            let res = ureq::post("https://login.growtopiagame.com/player/login/dashboard")
                .set("User-Agent", USER_AGENT)
                .send_string(&login_info);

            match res {
                Ok(res) => {
                    if res.status() != 200 {
                        self.log_warn("Failed to get OAuth links");
                        self.sleep();
                    } else {
                        let body = res.into_string()?;
                        let pattern =
                            regex::Regex::new("https://login\\.growtopiagame\\.com/(apple|google|player/growid)/(login|redirect)\\?token=[^\"]+");
                        let links = match pattern {
                            Ok(regex) => regex
                                .find_iter(&body)
                                .map(|m| m.as_str().to_owned())
                                .collect::<Vec<String>>(),
                            Err(_) => Vec::new(),
                        };
                        return Ok(links);
                    }
                }
                Err(err) => {
                    self.log_error(&format!("Request error: {}, retrying...", err));
                    self.sleep();
                }
            }
        }
    }

    pub fn spoof(&self) {
        self.log_info("Spoofing core data");
        self.set_status(EStatus::SpoofingData);
        let mut info = self.info.lock().unwrap();
        info.login_info.klv = proton::generate_klv(
            &info.login_info.protocol,
            &info.login_info.game_version,
            &info.login_info.rid,
        );
        info.login_info.hash =
            proton::hash_string(&format!("{}RT", info.login_info.mac)).to_string();
        info.login_info.hash2 =
            proton::hash_string(&format!("{}RT", random::hex(16, true))).to_string();
    }

    pub fn to_http(&self) {
        self.log_info("Fetching server data");
        let server = if config::get_use_alternate_server() {
            "https://www.growtopia2.com/growtopia/server_data.php"
        } else {
            "https://www.growtopia1.com/growtopia/server_data.php"
        };
        self.set_status(EStatus::FetchingServer);
        loop {
            let req = self
                .agent
                .post(server)
                .set(
                    "User-Agent",
                    "UbiServices_SDK_2022.Release.9_PC64_ansi_static",
                )
                .set("Content-Type", "application/x-www-form-urlencoded")
                .send_string("version=5.23&platform=0&protocol=216");

            let res = match req {
                Ok(res) => res,
                Err(err) => {
                    self.log_error(&format!("Request error: {}, retrying...", err));
                    self.sleep();
                    continue;
                }
            };

            if res.status() != 200 {
                self.log_warn("Failed to fetch server data");
                self.sleep();
            } else {
                let body = res.into_string().unwrap_or_default();
                self.parse_server_data(body);
                break;
            }
        }
    }

    pub fn parse_server_data(&self, data: String) {
        self.log_info("Parsing server data");
        self.set_status(EStatus::ParsingServer);
        let mut info = self.info.lock().unwrap();
        info.server_data = data
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

    fn connect_to_server(&self, ip: &str, port: &str) {
        self.log_info(&format!("Connecting to the server {}:{}", ip, port));
        self.set_status(EStatus::Connecting);

        let socket_address = SocketAddr::from_str(&format!("{}:{}", ip, port)).unwrap();

        let mut host = self.host.lock().unwrap();
        match host.connect(socket_address, 2, 0) {
            Ok(peer) => {
                peer.set_ping_interval(100);
            }
            Err(err) => {
                self.log_error(&format!("Failed to connect to the server: {}", err));
            }
        }
    }

    pub fn set_ping(&self) {
        if let Ok(mut host) = self.host.try_lock() {
            if let Ok(peer_id) = self.peer_id.try_lock() {
                if let Some(peer_id) = *peer_id {
                    let peer = host.peer_mut(peer_id);
                    if let Ok(mut temp) = self.temporary_data.try_write() {
                        temp.ping = peer.round_trip_time().as_millis() as u32;
                    }
                }
            }
        }
    }

    fn process_events(self: Arc<Self>) {
        loop {
            let (is_running, is_redirecting, ip, port) = {
                let state = self.state.lock().unwrap();
                let server = self.server.lock().unwrap();

                (
                    state.is_running,
                    state.is_redirecting,
                    server.ip.clone(),
                    server.port.to_string(),
                )
            };

            if !is_running {
                break;
            }

            if is_redirecting {
                self.log_info(&format!("Redirecting to server {}:{}", ip, port));
                self.connect_to_server(&ip, &port);
            } else {
                if !self.reconnect() {
                    return;
                }
            }

            loop {
                let event = {
                    let mut host = self.host.lock().unwrap();
                    host.service().ok().flatten().map(|e| e.no_ref())
                };

                if let Some(event) = event {
                    match event {
                        enet::EventNoRef::Connect { peer, .. } => {
                            self.log_info("Connected to the server");
                            self.set_status(EStatus::Connected);
                            let mut peer_id = self.peer_id.lock().unwrap();
                            *peer_id = Some(peer);
                        }
                        enet::EventNoRef::Disconnect { .. } => {
                            self.log_warn("Disconnected from the server");
                            self.set_status(EStatus::Disconnected);
                            let mut world = self.world.write().unwrap();
                            let mut position = self.position.lock().unwrap();
                            let mut temp = self.temporary_data.write().unwrap();
                            let mut state = self.state.lock().unwrap();
                            state.is_ingame = false;
                            state.is_allowed_to_warp = false;
                            self.players.lock().unwrap().clear();
                            world.reset();
                            position.reset();
                            temp.entered_world = false;
                            break;
                        }
                        enet::EventNoRef::Receive { packet, .. } => {
                            let data = packet.data();
                            if data.len() < 4 {
                                continue;
                            }
                            let packet_id = LittleEndian::read_u32(&data[0..4]);
                            let packet_type = EPacketType::from(packet_id);
                            let bot_clone = Arc::clone(&self);
                            packet_handler::handle(bot_clone, packet_type, &data[4..]);
                        }
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
    }

    pub fn disconnect(&self) {
        let peer_id = self.peer_id.lock().unwrap().clone();
        if let Some(peer_id) = peer_id {
            if let Ok(mut host) = self.host.try_lock() {
                let peer = host.peer_mut(peer_id);
                peer.disconnect(0);
            }
        }
    }

    pub fn disconnect_now(&self) {
        let peer_id = self.peer_id.lock().unwrap().clone();
        if let Some(peer_id) = peer_id {
            if let Ok(mut host) = self.host.try_lock() {
                let peer = host.peer_mut(peer_id);
                peer.disconnect_now(0);
            }
        }
    }

    pub fn send_packet(&self, packet_type: EPacketType, message: String) {
        let mut packet_data = Vec::new();
        packet_data.extend_from_slice(&(packet_type as u32).to_le_bytes());
        packet_data.extend_from_slice(message.as_bytes());
        let pkt = enet::Packet::reliable(packet_data.as_slice());

        if let Ok(peer_id) = self.peer_id.lock() {
            if let Some(peer_id) = *peer_id {
                if let Ok(mut host) = self.host.try_lock() {
                    let peer = host.peer_mut(peer_id);
                    if let Err(err) = peer.send(0, &pkt) {
                        self.log_error(&format!("Failed to send packet: {}", err));
                    }
                }
            }
        }
    }

    pub fn send_packet_raw(&self, packet: &TankPacket) {
        let packet_size = size_of::<EPacketType>()
            + size_of::<TankPacket>()
            + packet.extended_data_length as usize;
        let mut enet_packet_data = vec![0u8; packet_size];

        let packet_type = EPacketType::NetMessageGamePacket as u32;
        enet_packet_data[..size_of::<u32>()].copy_from_slice(&packet_type.to_le_bytes());

        let tank_packet_bytes = bincode::serialize(packet).expect("Failed to serialize TankPacket");
        enet_packet_data[size_of::<u32>()..size_of::<u32>() + tank_packet_bytes.len()]
            .copy_from_slice(&tank_packet_bytes);

        let enet_packet = enet::Packet::reliable(enet_packet_data.as_slice());

        if let Ok(peer_id) = self.peer_id.lock() {
            if let Some(peer_id) = *peer_id {
                if let Ok(mut host) = self.host.try_lock() {
                    let peer = host.peer_mut(peer_id);
                    if let Err(err) = peer.send(0, &enet_packet) {
                        self.log_error(&format!("Failed to send packet: {}", err));
                    }
                }
            }
        }
    }

    pub fn is_inworld(&self) -> bool {
        self.world.read().unwrap().name != "EXIT"
    }

    pub fn collect(&self) {
        if !self.is_inworld() {
            return;
        }

        let (bot_x, bot_y) = {
            let position = self.position.lock().expect("Failed to lock position");
            (position.x, position.y)
        };

        let items = {
            let world = self.world.read().expect("Failed to lock world");
            world.dropped.items.clone()
        };

        for obj in items {
            let dx = (bot_x - obj.x).abs() / 32.0;
            let dy = (bot_y - obj.y).abs() / 32.0;
            let distance = (dx.powi(2) + dy.powi(2)).sqrt();
            if distance <= 5.0 {
                let can_collect = {
                    let inventory = self.inventory.lock().expect("Failed to lock inventory");
                    let inventory_size = inventory.size;

                    if inventory.items.get(&obj.id).is_none()
                        && inventory_size > inventory.item_count as u32
                    {
                        true
                    } else {
                        if let Some(item) = inventory.items.get(&obj.id) {
                            item.amount < 200
                        } else {
                            false
                        }
                    }
                };

                if can_collect {
                    let mut pkt = TankPacket::default();
                    pkt._type = ETankPacketType::NetGamePacketItemActivateObjectRequest;
                    pkt.vector_x = obj.x;
                    pkt.vector_y = obj.y;
                    pkt.value = obj.uid;
                    self.send_packet_raw(&pkt);
                    self.log_info("Collect packet sent");
                }
            }
        }
    }

    pub fn place(&self, offset_x: i32, offset_y: i32, item_id: u32) {
        let mut pkt = TankPacket::default();
        pkt._type = ETankPacketType::NetGamePacketTileChangeRequest;
        let (base_x, base_y) = {
            let position = self.position.lock().expect("Failed to lock position");
            pkt.vector_x = position.x;
            pkt.vector_y = position.y;
            pkt.int_x = (position.x / 32.0).floor() as i32 + offset_x;
            pkt.int_y = (position.y / 32.0).floor() as i32 + offset_y;
            pkt.value = item_id;

            (
                (position.x / 32.0).floor() as i32,
                (position.y / 32.0).floor() as i32,
            )
        };

        if pkt.int_x <= base_x + 4
            && pkt.int_x >= base_x - 4
            && pkt.int_y <= base_y + 4
            && pkt.int_y >= base_y - 4
        {
            self.send_packet_raw(&pkt);
            /*
            00000000 00000000 00001010 00110000
            the fifth bit from the end is set if facing left
             */
            pkt.flags = if offset_x > 0 { 2592 } else { 2608 };
            pkt._type = ETankPacketType::NetGamePacketState;
            self.send_packet_raw(&pkt);
        }
    }

    pub fn punch(&self, offset_x: i32, offset_y: i32) {
        self.place(offset_x, offset_y, 18);
    }

    pub fn wrench(&self, offset_x: i32, offset_y: i32) {
        self.place(offset_x, offset_y, 32);
    }

    pub fn wear(&self, item_id: u32) {
        let packet = TankPacket {
            _type: ETankPacketType::NetGamePacketItemActivateRequest,
            value: item_id,
            ..Default::default()
        };

        self.send_packet_raw(&packet);
    }

    pub fn warp(&self, world_name: String) {
        self.log_info(&format!("Warping to world: {}", world_name));
        self.state.lock().unwrap().is_allowed_to_warp = false;
        self.send_packet(
            EPacketType::NetMessageGameMessage,
            format!("action|join_request\nname|{}\ninvitedWorld|0\n", world_name),
        );
    }

    pub fn talk(&self, message: String) {
        self.send_packet(
            EPacketType::NetMessageGenericText,
            format!("action|input\n|text|{}\n", message),
        );
    }

    pub fn leave(&self) {
        if self.is_inworld() {
            self.send_packet(
                EPacketType::NetMessageGameMessage,
                "action|quit_to_exit\n".to_string(),
            );
        }
    }

    pub fn walk(&self, x: i32, y: i32, ap: bool) {
        if !ap {
            let mut position = self.position.lock().expect("Failed to lock position");
            position.x += (x * 32) as f32;
            position.y += (y * 32) as f32;
        }

        let mut pkt = TankPacket::default();
        {
            let position = self.position.lock().expect("Failed to lock position");
            pkt._type = ETankPacketType::NetGamePacketState;
            pkt.vector_x = position.x;
            pkt.vector_y = position.y;
            pkt.int_x = -1;
            pkt.int_y = -1;
            pkt.flags |= (1 << 1) | (1 << 5);
        }

        if safe_check::is_connected(self) && self.is_inworld() {
            self.send_packet_raw(&pkt);
        }
    }

    pub fn find_path(&self, x: u32, y: u32) {
        let position = {
            let position = self.position.lock().expect("Failed to lock position");
            position.clone()
        };

        let paths = {
            let astar = self.astar.lock().expect("Failed to lock astar");
            astar.find_path((position.x as u32) / 32, (position.y as u32) / 32, x, y)
        };

        let delay = config::get_findpath_delay();
        if let Some(paths) = paths {
            for node in paths {
                let pos_y = get_coordinate_to_touch_ground(node.y as f32 * 32.0);
                {
                    let mut position = self.position.lock().expect("Failed to lock position");
                    position.x = node.x as f32 * 32.0;
                    position.y = pos_y;
                }
                self.walk(node.x as i32, node.y as i32, true);
                thread::sleep(Duration::from_millis(delay as u64));
            }
        }
    }

    pub fn drop_item(&self, item_id: u32, amount: u32) {
        self.send_packet(
            EPacketType::NetMessageGenericText,
            format!("action|drop\n|itemID|{}\n", item_id),
        );
        thread::sleep(Duration::from_millis(100));
        let mut temp_data = self.temporary_data.write().unwrap();
        temp_data.drop = (item_id, amount);
    }

    pub fn trash_item(&self, item_id: u32, amount: u32) {
        self.send_packet(
            EPacketType::NetMessageGenericText,
            format!("action|trash\n|itemID|{}\n", item_id),
        );
        thread::sleep(Duration::from_millis(100));
        let mut temp_data = self.temporary_data.write().unwrap();
        temp_data.trash = (item_id, amount);
    }

    pub fn accept_access(&self) {
        let net_id = {
            let state = self.state.lock().unwrap();
            state.net_id
        };
        self.wrench_player(net_id);
        let mut temp_data = self.temporary_data.write().unwrap();

        temp_data.dialog_callback = Some(|bot: &Arc<Bot>| {
            let net_id = {
                let state = bot.state.lock().unwrap();
                state.net_id
            };
            bot.send_packet(EPacketType::NetMessageGenericText, format!("action|dialog_return\ndialog_name|popup\nnetID|{}|\nbuttonClicked|acceptlock\n", net_id));
            let mut temp_data = bot.temporary_data.write().unwrap();
            temp_data.dialog_callback = Some(|bot: &Arc<Bot>| {
                bot.send_packet(
                    EPacketType::NetMessageGenericText,
                    "action|dialog_return\ndialog_name|acceptaccess\n".to_string(),
                );
                let mut temp_data = bot.temporary_data.write().unwrap();
                temp_data.dialog_callback = None;
            });
        });
    }

    pub fn wrench_player(&self, net_id: u32) {
        self.send_packet(
            EPacketType::NetMessageGenericText,
            format!("action|wrench\n|netid|{}\n", net_id),
        );
    }

    pub fn enter_door(&self) {
        self.punch(0, 0);
    }
}

fn poll(bot: Arc<Bot>) {
    let bot_clone = Arc::clone(&bot);
    thread::spawn(move || loop {
        if !safe_check::is_connected(&bot_clone) {
            break;
        }
        if config::get_auto_collect() {
            bot_clone.collect();
        }
        bot_clone.set_ping();
        thread::sleep(Duration::from_millis(100));
    });
}

pub fn get_coordinate_to_touch_ground(y: f32) -> f32 {
    let colrect_bottom_center_y = y + 30.0;
    let block_y = ((colrect_bottom_center_y / 32.0).floor() + 1.0) * 32.0;

    (block_y - 30.0).floor()
}
