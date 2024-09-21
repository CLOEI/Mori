mod astar;
pub mod features;
mod inventory;
mod login;
mod packet_handler;
mod variant_handler;
mod proxy;

use astar::AStar;
use byteorder::{ByteOrder, LittleEndian};
use rusty_enet as enet;
use gtitem_r::structs::ItemDatabase;
use inventory::Inventory;
use paris::{error, info, warn};
use std::sync::{Arc, Mutex, RwLock};
use std::{collections::HashMap, thread, time::Duration, vec};
use std::fmt::Debug;
use std::mem::size_of;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::str::{self, FromStr};
use urlencoding::encode;
use socks::{Socks5Datagram};

use crate::types::bot_info::{TemporaryData, FTUE};
use crate::types::{etank_packet_type::ETankPacketType, player::Player};
use crate::{
    types::{self, tank_packet::TankPacket},
    utils,
};
use crate::{
    types::{
        bot_info::{Info, Server, State},
        elogin_method::ELoginMethod,
        epacket_type::EPacketType,
        login_info::LoginInfo,
        vector::Vector2,
    },
    utils::{
        proton::{self},
        random::{self},
    },
};
use crate::bot::proxy::{SocketType, Socks5UdpSocket};
use crate::utils::config;

static USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0";

pub struct Bot {
    pub info: Arc<RwLock<Info>>,
    pub state: Arc<RwLock<State>>,
    pub server: Arc<RwLock<Server>>,
    pub position: Arc<RwLock<Vector2>>,
    pub temporary_data: Arc<RwLock<TemporaryData>>,
    pub host: Arc<Mutex<enet::Host<SocketType>>>,
    pub peer_id: Arc<RwLock<Option<enet::PeerID>>>,
    pub world: Arc<RwLock<gtworld_r::World>>,
    pub inventory: Arc<RwLock<Inventory>>,
    pub players: Arc<RwLock<Vec<Player>>>,
    pub astar: Arc<RwLock<AStar>>,
    pub ftue: Arc<RwLock<FTUE>>,
    pub item_database: Arc<ItemDatabase>,
}

impl Bot {
    pub fn new(
        bot_config: types::config::BotConfig,
        item_database: Arc<ItemDatabase>,
    ) -> Self {
        let proxy_address: Option<String> = None;
        let socket: SocketType = if let Some(proxy) = proxy_address {
            let proxy_addr = SocketAddr::from_str("").expect("Invalid proxy address");
            let username = "".to_string();
            let password = "".to_string();

            let udp_datagram = Socks5Datagram::bind_with_password(proxy_addr, SocketAddr::from_str("0.0.0.0:0").unwrap(), &username, &password)
                .expect("Failed to bind SOCKS5 datagram");
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

        Self {
            info: Arc::new(RwLock::new(Info {
                payload: utils::textparse::parse_and_store_as_vec(&bot_config.payload),
                recovery_code: bot_config.recovery_code,
                login_method: bot_config.login_method,
                token: bot_config.token,
                login_info: LoginInfo::new(),
                timeout: 0,
                ..Default::default()
            })),
            state: Arc::new(RwLock::new(State::default())),
            server: Arc::new(RwLock::new(Server::default())),
            position: Arc::new(RwLock::new(Vector2::default())),
            temporary_data: Arc::new(RwLock::new(TemporaryData::default())),
            host: Arc::new(Mutex::new(host)),
            peer_id: Arc::new(RwLock::new(None)),
            world: Arc::new(RwLock::new(gtworld_r::World::new(item_database.clone()))),
            inventory: Arc::new(RwLock::new(Inventory::new())),
            players: Arc::new(RwLock::new(Vec::new())),
            astar: Arc::new(RwLock::new(AStar::new(item_database.clone()))),
            ftue: Arc::new(RwLock::new(FTUE::default())),
            item_database,
        }
    }
}

pub fn logon(bot: &Arc<Bot>, data: String) {
    set_status(bot, "Logging in...");
    if data.is_empty() {
        spoof(&bot);
    } else {
        update_login_info(&bot, data);
    }
    bot.state.write().unwrap().is_running = true;
    poll(bot);
    process_events(&bot);
}

pub fn set_status(bot: &Arc<Bot>, message: &str) {
    bot.info.write().unwrap().status = message.to_string();
}

pub fn reconnect(bot: &Arc<Bot>) -> bool {
    set_status(bot, "Reconnecting...");
    to_http(bot);

    let (meta, login_method, oauth_links_empty) = {
        let info = bot.info.read().unwrap();
        (
            info.server_data.get("meta").unwrap().clone(),
            info.login_method.clone(),
            info.oauth_links.is_empty(),
        )
    };

    {
        let mut info = bot.info.write().unwrap();
        info.login_info.meta = meta;
    }

    if login_method != ELoginMethod::STEAM && oauth_links_empty {
        match get_oauth_links(&bot) {
            Ok(links) => {
                let mut info = bot.info.write().unwrap();
                info.oauth_links = links;
                info!("Successfully got OAuth links for: apple, google and legacy");
            }
            Err(err) => {
                info!("Failed to get OAuth links: {}", err);
                return false;
            }
        }
    }

    get_token(bot);

    if {
        !bot.state.write().unwrap().is_running
    } {
        return false;
    }

    let (server, port) = {
        let info = bot.info.read().unwrap();
        (
            info.server_data["server"].clone(),
            info.server_data["port"].clone(),
        )
    };

    connect_to_server(bot, server, port);
    true
}

pub fn relog(bot: &Arc<Bot>) {
    info!("Relogging bot");
    {
        let mut state = bot.state.write().unwrap();
        state.is_running = false;
        state.is_redirecting = false;
    }
    set_status(&bot, "Relogging");
    disconnect(&bot);
    reconnect(&bot);
}

fn update_login_info(bot: &Arc<Bot>, data: String) {
    set_status(bot, "Updating login info");
    let mut info = bot.info.write().unwrap();
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

fn token_still_valid(bot: &Arc<Bot>) -> bool {
    info!("Checking if token is still valid");
    set_status(bot, "Checking refresh token");

    let (token, login_info) = {
        let info = bot.info.read().unwrap();
        if info.token.is_empty() {
            return false;
        }
        (info.token.clone(), info.login_info.to_string())
    };

    loop {
        // Make HTTP request to check token validity
        let response = ureq::post("https://login.growtopiagame.com/player/growid/checktoken?valKey=40db4045f2d8c572efe8c4a060605726")
            .set("User-Agent", "UbiServices_SDK_2022.Release.9_PC64_ansi_static")
            .send_form(&[("refreshToken", token.as_str()), ("clientData", login_info.as_str())]);

        match response {
            Ok(res) => {
                if res.status() != 200 {
                    error!("Failed to refresh token, retrying...");
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
                    info!("Token is still valid | new token: {}", new_token);

                    let mut info = bot.info.write().unwrap();
                    info.token = new_token;

                    return true;
                } else {
                    error!("Token is invalid");
                    return false;
                }
            }
            Err(err) => {
                error!("Request error: {}, retrying...", err);
                thread::sleep(Duration::from_secs(1));
                continue;
            }
        };
    }
}

pub fn poll(bot: &Arc<Bot>) {
    let bot_clone = bot.clone();
    thread::spawn(move || loop {
        {
            let state = bot_clone.state.read().unwrap();
            if !state.is_running {
                break;
            }
        }
        collect(&bot_clone);
        set_ping(&bot_clone);
        thread::sleep(Duration::from_millis(100));
    });
}

pub fn sleep(bot: &Arc<Bot>) {
    let mut info = bot.info.write().unwrap();
    info.timeout += utils::config::get_timeout();
    while info.timeout > 0 {
        info.timeout -= 1;
        drop(info);
        thread::sleep(Duration::from_secs(1));
        info = bot.info.write().unwrap();
    }
}

pub fn get_token(bot: &Arc<Bot>) {
    if token_still_valid(bot) {
        return;
    }

    info!("Getting token for bot");
    set_status(bot, "Getting token");
    let (payload, recovery_code, method, oauth_links) = {
        let info = bot.info.read().unwrap();
        (
            info.payload.clone(),
            info.recovery_code.clone(),
            info.login_method.clone(),
            info.oauth_links.clone(),
        )
    };

    let token_result = match method {
        ELoginMethod::GOOGLE => {
            match login::get_google_token(oauth_links[1].as_str(), &payload[0], &payload[1]) {
                Ok(res) => res,
                Err(err) => {
                    if err.to_string().contains("too many people") {
                        error!("Too many people trying to login");
                    } else {
                        error!("Failed to get Google token: {}", err);
                    }
                    return;
                }
            }
        }
        ELoginMethod::LEGACY => {
            match login::get_legacy_token(
                oauth_links[2].as_str(),
                &payload[0],
                &payload[1],
            ) {
                Ok(res) => res,
                Err(err) => {
                    error!("Failed to get legacy token: {}", err);
                    return;
                }
            }
        }
        ELoginMethod::STEAM => {
            {
                bot.info.write().unwrap().login_info.platform_id = "15,1,0".to_string();
            }
            let recovery_code = {
                let info = bot.info.read().unwrap();
                info.recovery_code.clone()
            };
            match login::get_ubisoft_token(&bot, &recovery_code, &payload[0], &payload[1], &payload[2], &payload[3]) {
                Ok(res) => res,
                Err(err) => {
                    error!("Failed to get Ubisoft token: {}", err);
                    return;
                }
            }
        }
        _ => {
            warn!("Invalid login method");
            return;
        }
    };

    if token_result.len() > 0 {
        let mut info = bot.info.write().unwrap();
        info.token = token_result;
        info!("Received the token: {}", info.token);
    }
}

pub fn get_oauth_links(bot: &Arc<Bot>) -> Result<Vec<String>, ureq::Error> {
    info!("Getting OAuth links");
    set_status(bot, "Getting OAuth links");
    loop {
        let res = ureq::post("https://login.growtopiagame.com/player/login/dashboard")
            .set("User-Agent", USER_AGENT)
            .send_string(&encode(bot.info.read().unwrap().login_info.to_string().as_str()));

        match res {
            Ok(res) => {
                if res.status() != 200 {
                    warn!("Failed to get OAuth links");
                    sleep(bot);
                } else {
                    let body = res.into_string()?;
                    let pattern = regex::Regex::new("https://login\\.growtopiagame\\.com/(apple|google|player/growid)/(login|redirect)\\?token=[^\"]+");
                    let links = pattern
                        .unwrap()
                        .find_iter(&body)
                        .map(|m| m.as_str().to_owned())
                        .collect::<Vec<String>>();

                    return Ok(links);
                }
            }
            Err(err) => {
                error!("Request error: {}, retrying...", err);
                sleep(bot);
            }
        }
    }
}

pub fn spoof(bot: &Arc<Bot>) {
    info!("Spoofing bot data");
    set_status(bot, "Spoofing bot data");
    let mut info = bot.info.write().unwrap();
    info.login_info.klv = proton::generate_klv(
        &info.login_info.protocol,
        &info.login_info.game_version,
        &info.login_info.rid,
    );
    info.login_info.hash =
        proton::hash_string(format!("{}RT", info.login_info.mac).as_str()).to_string();
    info.login_info.hash2 =
        proton::hash_string(format!("{}RT", random::hex(16, true)).as_str()).to_string();
}

pub fn to_http(bot: &Arc<Bot>) {
    info!("Fetching server data");
    let server = if config::get_use_alternate_server() {
        "https://www.growtopia2.com/growtopia/server_data.php"
    } else {
        "https://www.growtopia1.com/growtopia/server_data.php"
    };
    set_status(bot, "Fetching server data");
    loop {
        let req = ureq::post(server).set(
            "User-Agent",
            "UbiServices_SDK_2022.Release.9_PC64_ansi_static",
        );

        let res = req.send_string("");
        let res = match res {
            Ok(res) => res,
            Err(err) => {
                error!("Request error: {}, retrying...", err);
                sleep(bot);
                continue;
            }
        };

        if res.status() != 200 {
            warn!("Failed to fetch server data");
            sleep(bot);
        } else {
            let body = res.into_string().unwrap();
            parse_server_data(&bot, body);
            break;
        }
    }
}

pub fn parse_server_data(bot: &Arc<Bot>, data: String) {
    info!("Parsing server data");
    set_status(bot, "Parsing server data");
    bot.info.write().unwrap().server_data = data
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

fn connect_to_server(bot: &Arc<Bot>, ip: String, port: String) {
    info!("Connecting to the server {}:{}", ip, port);
    set_status(&bot, "Connecting to the server");

    let host_result = bot.host.lock();
    if host_result.is_err() {
        error!("Host lock poisoned, attempting to recover");
        let mut host = host_result.unwrap_or_else(|poisoned| poisoned.into_inner());
        host.connect(
            SocketAddr::from_str(format!("{}:{}", ip, port).as_str()).unwrap(),
            2,
            0,
        )
            .expect("Failed to connect to the server");
    } else {
        let mut host = host_result.unwrap();
        host.connect(
            SocketAddr::from_str(format!("{}:{}", ip, port).as_str()).unwrap(),
            2,
            0,
        )
            .expect("Failed to connect to the server");
    }
}

pub fn set_ping(bot: &Arc<Bot>) {
    if let Ok(mut host) = bot.host.try_lock() {
        if let Ok(peer_id) = bot.peer_id.try_read() {
            if let Some(peer_id) = *peer_id {
                let peer = host.peer_mut(peer_id);
                if let Ok(mut info) = bot.info.try_write() {
                    info.ping = peer.round_trip_time().as_millis() as u32;
                }
            }
        }
    }
}

fn process_events(bot: &Arc<Bot>) {
    loop {
        let (is_running, is_redirecting, ip, port) = {
            let state = bot.state.read().unwrap();
            let server = bot.server.read().unwrap();

            (
                state.is_running,
                state.is_redirecting,
                server.ip.clone(),
                server.port.to_string().clone(),
            )
        };

        if !is_running {
            break;
        }

        if is_redirecting {
            info!("Redirecting to the server {}:{}", ip, port);
            connect_to_server(bot, ip, port);
        } else {
            if !reconnect(bot) {
                return;
            }
        }

        loop {
            let (event) = {
                let mut host = bot.host.lock().unwrap();
                let e = host.service().unwrap();

                if let Some(event) = e {
                    Some(event.no_ref())
                } else {
                    None
                }
            };

            if let Some(event) = event.clone() {
                match event {
                    enet::EventNoRef::Connect { peer, .. } => {
                        info!("Connected to the server");
                        set_status(bot, "Connected");
                        {
                            let mut peer_id = bot.peer_id.write().unwrap();
                            *peer_id = Some(peer);
                        }
                    }
                    enet::EventNoRef::Disconnect { .. } => {
                        warn!("Disconnected from the server");
                        set_status(bot, "Disconnected");
                        break;
                    }
                    enet::EventNoRef::Receive { packet, .. } => {
                        let data = packet.data();
                        if data.len() < 4 {
                            continue;
                        }
                        let packet_id = LittleEndian::read_u32(&data[0..4]);
                        let packet_type = EPacketType::from(packet_id);
                        packet_handler::handle(bot, packet_type, &data[4..]);
                    }
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}

pub fn disconnect(bot: &Arc<Bot>) {
    let peer_id = bot.peer_id.read().unwrap();
    match *peer_id {
        Some(peer_id) => {
            bot.host.lock().unwrap().peer_mut(peer_id.clone()).disconnect(0);
        }
        None => {}
    }
}

pub fn send_packet(bot: &Arc<Bot>, packet_type: EPacketType, message: String) {
    let mut packet_data = Vec::new();
    packet_data.extend_from_slice(&(packet_type as u32).to_le_bytes());
    packet_data.extend_from_slice(&message.as_bytes());
    let pkt = enet::Packet::reliable(packet_data.as_slice());
    let peer_id = bot.peer_id.read().unwrap().unwrap().clone();
    let mut host = bot.host.lock().unwrap();
    let peer = host.peer_mut(peer_id);
    match peer.send(0, &pkt) {
        Ok(_) => {}
        Err(err) => {
            error!("Failed to send packet: {}", err);
        }
    }
}

pub fn send_packet_raw(bot: &Arc<Bot>, packet: &TankPacket) {
    let packet_size = size_of::<EPacketType>()
        + size_of::<TankPacket>()
        + packet.extended_data_length as usize;
    let mut enet_packet_data = vec![0u8; packet_size];

    let packet_type = EPacketType::NetMessageGamePacket as u32;
    enet_packet_data[..size_of::<u32>()].copy_from_slice(&packet_type.to_le_bytes());

    let tank_packet_bytes = bincode::serialize(packet).expect("Failed to serialize TankPacket");
    enet_packet_data
        [size_of::<u32>()..size_of::<u32>() + tank_packet_bytes.len()]
        .copy_from_slice(&tank_packet_bytes);

    let enet_packet = enet::Packet::reliable(enet_packet_data.as_slice());
    let peer_id = bot.peer_id.read().unwrap().unwrap().clone();
    let mut host = bot.host.lock().unwrap();
    let peer = host.peer_mut(peer_id);
    match peer.send(0, &enet_packet) {
        Ok(_) => {}
        Err(err) => {
            error!("Failed to send packet: {}", err);
        }
    }
}

pub fn is_inworld(bot: &Arc<Bot>) -> bool {
    bot.world.read().unwrap().name != "EXIT"
}

pub fn collect(bot: &Arc<Bot>) {
    if !is_inworld(bot) {
        return;
    }

    let (bot_x, bot_y) = {
        let position = bot.position.read().unwrap();
        (position.x, position.y)
    };

    let items = {
        let world = bot.world.read().unwrap();
        world.dropped.items.clone()
    };

    for obj in items {
        let dx = (bot_x - obj.x).abs() / 32.0;
        let dy = (bot_y - obj.y).abs() / 32.0;
        let distance = (dx.powi(2) + dy.powi(2)).sqrt();
        if distance <= 5.0 {
            let can_collect = {
                let inventory = bot.inventory.read().unwrap();
                let inventory_size = inventory.size.clone();

                if inventory.items.get(&obj.id).is_none() && inventory_size > inventory.item_count as u32 {
                    true
                } else {
                    if let Some(item) = inventory.items.get(&obj.id) {
                        if item.amount < 200 {
                            true
                        } else {
                            false
                        }
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
                send_packet_raw(bot, &pkt);
                info!("Collect packet sent");
            }
        }
    }
}

pub fn place(bot: &Arc<Bot>, offset_x: i32, offset_y: i32, item_id: u32) {
    let mut pkt = TankPacket::default();
    pkt._type = ETankPacketType::NetGamePacketTileChangeRequest;
    let base_x;
    let base_y;
    {
        let position = bot.position.read().unwrap();
        pkt.vector_x = position.x;
        pkt.vector_y = position.y;
        pkt.int_x = (position.x / 32.0).floor() as i32 + offset_x;
        pkt.int_y = (position.y / 32.0).floor() as i32 + offset_y;
        pkt.value = item_id;

        base_x = (position.x / 32.0).floor() as i32;
        base_y = (position.y / 32.0).floor() as i32;
    }

    if pkt.int_x <= base_x + 4
        && pkt.int_x >= base_x - 4
        && pkt.int_y <= base_y + 4
        && pkt.int_y >= base_y - 4
    {
        send_packet_raw(bot, &pkt);
    }
}

pub fn punch(bot: &Arc<Bot>, offset_x: i32, offset_y: i32) {
    place(bot, offset_x, offset_y, 18);
}

pub fn wrench(bot: &Arc<Bot>, offset_x: i32, offset_y: i32) {
    place(bot, offset_x, offset_y, 32);
}

pub fn wear(bot: &Arc<Bot>, item_id: u32) {
    let packet = TankPacket {
        _type: ETankPacketType::NetGamePacketItemActivateRequest,
        value: item_id,
        ..Default::default()
    };

    send_packet_raw(bot, &packet);
}

pub fn warp(bot: &Arc<Bot>, world_name: String) {
    if bot.state.read().unwrap().is_not_allowed_to_warp {
        return;
    }
    info!("Warping to world: {}", world_name);
    send_packet(
        bot,
        EPacketType::NetMessageGameMessage,
        format!("action|join_request\nname|{}\ninvitedWorld|0\n", world_name),
    );
}

pub fn talk(bot: &Arc<Bot>, message: String) {
    send_packet(
        bot,
        EPacketType::NetMessageGameMessage,
        format!("action|input\n|text|{}\n", message),
    );
}

pub fn leave(bot: &Arc<Bot>) {
    if is_inworld(bot) {
        send_packet(
            bot,
            EPacketType::NetMessageGameMessage,
            "action|quit_to_exit\n".to_string(),
        );
    }
}

pub fn walk(bot: &Arc<Bot>, x: i32, y: i32, ap: bool) {
    if !ap {
        let mut position = bot.position.write().unwrap();
        position.x += (x * 32) as f32;
        position.y += (y * 32) as f32;
    }

    let mut pkt = TankPacket::default();
    {
        let position = bot.position.read().unwrap();
        pkt._type = ETankPacketType::NetGamePacketState;
        pkt.vector_x = position.x;
        pkt.vector_y = position.y;
        pkt.int_x = -1;
        pkt.int_y = -1;
        pkt.flags |= (1 << 1) | (1 << 5);
    }

    if bot.state.read().unwrap().is_running && is_inworld(bot) {
        send_packet_raw(bot, &pkt);
    }
}

pub fn find_path(bot: &Arc<Bot>, x: u32, y: u32) {
    let position = {
        let position = bot.position.read().unwrap();
        position.clone()
    };

    let paths = {
        let astar = bot.astar.read().unwrap();
        astar.find_path((position.x as u32) / 32, (position.y as u32) / 32, x, y)
    };

    let delay = utils::config::get_findpath_delay();
    if let Some(paths) = paths {
        for node in paths {
            {
                let mut position = bot.position.write().unwrap();
                position.x = node.x as f32 * 32.0;
                position.y = node.y as f32 * 32.0;
            }
            walk(bot, node.x as i32, node.y as i32, true);
            thread::sleep(Duration::from_millis(delay as u64));
        }
    }
}

pub fn drop_item(bot: &Arc<Bot>, item_id: u32, amount: u32) {
    send_packet(
        bot,
        EPacketType::NetMessageGenericText,
        format!("action|drop\n|itemID|{}\n", item_id),
    );
    thread::sleep(Duration::from_millis(100));
    bot.temporary_data.write().unwrap().drop = (item_id, amount);
}

pub fn trash_item(bot: &Arc<Bot>, item_id: u32, amount: u32) {
    send_packet(
        bot,
        EPacketType::NetMessageGenericText,
        format!("action|trash\n|itemID|{}\n", item_id),
    );
    thread::sleep(Duration::from_millis(100));
    bot.temporary_data.write().unwrap().trash = (item_id, amount);
}
