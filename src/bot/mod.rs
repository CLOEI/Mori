mod astar;
mod inventory;
mod login;
mod packet_handler;
mod variant_handler;

use astar::AStar;
use byteorder::{ByteOrder, LittleEndian};
use enet::{
    Address, BandwidthLimit, ChannelLimit, Enet, EventKind, Host, Packet, PacketMode, Peer, PeerID,
};
use gtitem_r::structs::ItemDatabase;
use inventory::Inventory;
use paris::{error, info, warn};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
    vec,
};
use urlencoding::encode;

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

static USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0";

pub struct Bot {
    pub info: Arc<Mutex<Info>>,
    pub state: Arc<Mutex<State>>,
    pub server: Arc<Mutex<Server>>,
    pub position: Arc<Mutex<Vector2>>,
    pub host: Arc<Mutex<Host<()>>>,
    pub peer_id: Arc<Mutex<Option<PeerID>>>,
    pub world: Arc<Mutex<gtworld_r::World>>,
    pub inventory: Arc<Mutex<Inventory>>,
    pub players: Arc<Mutex<Vec<Player>>>,
    pub astar: Arc<Mutex<AStar>>,
    pub item_database: Arc<ItemDatabase>,
}

impl Bot {
    pub fn new(bot_config: types::config::BotConfig, item_database: Arc<ItemDatabase>) -> Self {
        let enet = Enet::new().expect("could not initialize ENet");
        let host = enet
            .create_host::<()>(
                None,
                1,
                ChannelLimit::Limited(2),
                BandwidthLimit::Unlimited,
                BandwidthLimit::Unlimited,
                1,
                0,
            )
            .expect("could not create host");

        Self {
            info: Arc::new(Mutex::new(Info {
                username: bot_config.username,
                password: bot_config.password,
                recovery_code: bot_config.recovery_code,
                login_method: bot_config.login_method,
                token: bot_config.token,
                login_info: LoginInfo::new(),
                timeout: 0,
                ..Default::default()
            })),
            state: Arc::new(Mutex::new(State::default())),
            server: Arc::new(Mutex::new(Server::default())),
            position: Arc::new(Mutex::new(Vector2::default())),
            host: Arc::new(Mutex::new(host)),
            peer_id: Arc::new(Mutex::new(None)),
            world: Arc::new(Mutex::new(gtworld_r::World::new(item_database.clone()))),
            inventory: Arc::new(Mutex::new(Inventory::new())),
            players: Arc::new(Mutex::new(Vec::new())),
            astar: Arc::new(Mutex::new(AStar::new(item_database.clone()))),
            item_database,
        }
    }
}

pub fn logon(bot: &Arc<Bot>, data: String) {
    if data.is_empty() {
        spoof(&bot);
    } else {
        update_login_info(&bot, data);
    }
    bot.state.lock().unwrap().is_running = true;
    poll(bot);
    process_events(&bot);
}

pub fn reconnect(bot: &Arc<Bot>) {
    to_http(bot);

    let meta = bot
        .info
        .lock()
        .unwrap()
        .server_data
        .get("meta")
        .unwrap()
        .clone();

    bot.info.lock().unwrap().login_info.meta = meta;

    if bot.info.lock().unwrap().login_method != ELoginMethod::UBISOFT
        && bot.info.lock().unwrap().oauth_links.is_empty()
    {
        match get_oauth_links(&bot) {
            Ok(links) => {
                bot.info.lock().unwrap().oauth_links = links;
                info!("Successfully got OAuth links for: apple, google and legacy");
            }
            Err(err) => {
                info!("Failed to get OAuth links: {}", err);
                return;
            }
        }
    }

    get_token(bot);
    {
        let info = bot.info.lock().unwrap();
        connect_to_server(
            bot,
            info.server_data["server"].clone(),
            info.server_data["port"].clone(),
        );
    }
}

fn update_login_info(bot: &Arc<Bot>, data: String) {
    let mut info = bot.info.lock().unwrap();
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

    let response;
    {
        let info = bot.info.lock().unwrap();
        if info.token.is_empty() {
            return false;
        }

        response = ureq::post("https://login.growtopiagame.com/player/growid/checktoken?valKey=40db4045f2d8c572efe8c4a060605726")
            .set("User-Agent", "UbiServices_SDK_2022.Release.9_PC64_ansi_static")
            .send_form(&[("refreshToken", info.token.as_str()), ("clientData", info.login_info.to_string().as_str())]);
    }

    match response {
        Ok(res) => {
            if res.status() != 200 {
                error!("Failed to refresh token");
                return false;
            }

            let response_text = res.into_string().unwrap_or_default();
            let json_response: serde_json::Value = serde_json::from_str(&response_text).unwrap();

            if json_response["status"] == "success" {
                let token = json_response["token"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string();
                info!("Token is still valid | new token: {}", token);
                bot.info.lock().unwrap().token = token;
                return true;
            }
        }
        Err(err) => {
            error!("Request error: {}", err);
            return false;
        }
    };

    false
}

pub fn poll(bot: &Arc<Bot>) {
    let bot_clone = bot.clone();
    thread::spawn(move || loop {
        let state = bot_clone.state.lock().unwrap();
        if !state.is_running {
            break;
        }
        drop(state);
        collect(&bot_clone);
        thread::sleep(Duration::from_millis(20));
    });
}

pub fn sleep(bot: &Arc<Bot>) {
    let mut info = bot.info.lock().unwrap();
    info.timeout = utils::config::get_timeout();
    while info.timeout > 0 {
        info.timeout -= 1;
        drop(info);
        thread::sleep(Duration::from_secs(1));
        info = bot.info.lock().unwrap();
    }
}

pub fn get_token(bot: &Arc<Bot>) {
    if token_still_valid(bot) {
        return;
    }

    info!("Getting token for bot");
    let (username, password, recovery_code, method, oauth_links) = {
        let info = bot.info.lock().unwrap();
        (
            info.username.clone(),
            info.password.clone(),
            info.recovery_code.clone(),
            info.login_method.clone(),
            info.oauth_links.clone(),
        )
    };

    let token_result = match method {
        ELoginMethod::GOOGLE => {
            match login::get_google_token(oauth_links[1].as_str(), &username, &password) {
                Ok(res) => res,
                Err(err) => {
                    if err.to_string().contains("too many people") {
                        error!("Too many people trying to login");
                    }
                    return;
                }
            }
        }
        ELoginMethod::LEGACY => {
            match login::get_legacy_token(
                oauth_links[2].as_str(),
                username.as_str(),
                password.as_str(),
            ) {
                Ok(res) => res,
                Err(err) => {
                    error!("Failed to get legacy token: {}", err);
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
        let mut info = bot.info.lock().unwrap();
        info.token = token_result;
        info!("Received the token: {}", info.token);
    }
}

pub fn get_oauth_links(bot: &Arc<Bot>) -> Result<Vec<String>, ureq::Error> {
    loop {
        let res = ureq::post("https://login.growtopiagame.com/player/login/dashboard")
            .set("User-Agent", USER_AGENT)
            .send_string(&encode(
                bot.info.lock().unwrap().login_info.to_string().as_str(),
            ));

        match res {
            Ok(res) => {
                if res.status() != 200 {
                    warn!("Failed to get OAuth links");
                    sleep(bot);
                } else {
                    let body = res.into_string().unwrap();
                    let pattern = regex::Regex::new("https:\\/\\/login\\.growtopiagame\\.com\\/(apple|google|player\\/growid)\\/(login|redirect)\\?token=[^\"]+");
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
    let mut info = bot.info.lock().unwrap();
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
    loop {
        let req = ureq::post("https://www.growtopia1.com/growtopia/server_data.php").set(
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
    bot.info.lock().unwrap().server_data = data
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
    let mut host = bot.host.lock().unwrap();
    host.connect(
        &Address::new(ip.parse().unwrap(), port.parse().unwrap()),
        2,
        0,
    )
    .expect("Failed to connect to the server");
}

fn process_events(bot: &Arc<Bot>) {
    loop {
        let (is_running, is_redirecting, ip, port, server_data) = {
            let state = bot.state.lock().unwrap();
            let info = bot.info.lock().unwrap();
            let server = bot.server.lock().unwrap();

            (
                state.is_running,
                state.is_redirecting,
                server.ip.clone(),
                server.port.to_string().clone(),
                info.server_data.clone(),
            )
        };

        if !is_running {
            break;
        }

        if is_redirecting {
            info!("Redirecting to the server {}:{}", ip, port);
            connect_to_server(bot, ip, port);
        } else {
            reconnect(bot);
        }

        loop {
            let (event_kind, new_peer_id) = {
                let mut host = bot.host.lock().unwrap();
                let e = host
                    .service(Duration::from_millis(100))
                    .expect("Service failed");

                if let Some(event) = e {
                    let peer_id = event.peer_id().clone();
                    let event_kind = event.take_kind();
                    (Some(event_kind), Some(peer_id))
                } else {
                    (None, None)
                }
            };

            if let Some(peer_id) = new_peer_id {
                let mut x = bot.peer_id.lock().unwrap();
                *x = Some(peer_id);
            }

            if let Some(event_kind) = event_kind {
                match event_kind {
                    EventKind::Connect => {
                        info!("Connected to the server");
                    }
                    EventKind::Disconnect { .. } => {
                        warn!("Disconnected from the server");
                        break;
                    }
                    EventKind::Receive { packet, .. } => {
                        let data = packet.data();
                        if data.len() < 4 {
                            continue;
                        }
                        let packet_id = LittleEndian::read_u32(&data[0..4]);
                        let packet_type = EPacketType::from(packet_id);
                        info!("Received packet: {:?}", packet_type);
                        packet_handler::handle(bot, packet_type, &data[4..]);
                    }
                }
            }
        }
    }
}

pub fn disconnect(bot: &Arc<Bot>) {
    let peer_id = bot.peer_id.lock().unwrap().unwrap().clone();
    bot.host
        .lock()
        .unwrap()
        .peer_mut(peer_id)
        .unwrap()
        .disconnect(0);
}

pub fn send_packet(bot: &Arc<Bot>, packet_type: EPacketType, message: String) {
    let mut packet_data = Vec::new();
    packet_data.extend_from_slice(&(packet_type as u32).to_le_bytes());
    packet_data.extend_from_slice(&message.as_bytes());
    let pkt = Packet::new(packet_data, PacketMode::ReliableSequenced).unwrap();
    let peer_id = bot.peer_id.lock().unwrap().unwrap().clone();
    let mut host = bot.host.lock().unwrap();
    let peer = host.peer_mut(peer_id).unwrap();
    peer.send_packet(pkt, 0).expect("Failed to send packet");
}

pub fn send_packet_raw(bot: &Arc<Bot>, packet: &TankPacket) {
    let packet_size = std::mem::size_of::<EPacketType>()
        + std::mem::size_of::<TankPacket>()
        + packet.extended_data_length as usize;
    let mut enet_packet_data = vec![0u8; packet_size];

    let packet_type = EPacketType::NetMessageGamePacket as u32;
    enet_packet_data[..std::mem::size_of::<u32>()].copy_from_slice(&packet_type.to_le_bytes());

    let tank_packet_bytes = bincode::serialize(packet).expect("Failed to serialize TankPacket");
    enet_packet_data
        [std::mem::size_of::<u32>()..std::mem::size_of::<u32>() + tank_packet_bytes.len()]
        .copy_from_slice(&tank_packet_bytes);

    let enet_packet = Packet::new(enet_packet_data, PacketMode::ReliableSequenced)
        .expect("Failed to create ENet packet");
    let peer_id = bot.peer_id.lock().unwrap().unwrap().clone();
    let mut host = bot.host.lock().unwrap();
    let peer = host.peer_mut(peer_id).unwrap();
    peer.send_packet(enet_packet, 0)
        .expect("Failed to send raw packet");
}

pub fn is_inworld(bot: &Arc<Bot>) -> bool {
    bot.world.lock().unwrap().name != "EXIT"
}

pub fn collect(bot: &Arc<Bot>) {
    if !is_inworld(bot) {
        return;
    }

    let world = bot.world.lock().unwrap();
    for obj in &world.dropped.items {
        let distance;
        {
            let position = bot.position.lock().unwrap();
            distance = ((position.x - obj.x).powi(2) + (position.y - obj.y).powi(2)).sqrt() / 32.0;
        }
        if distance <= 5.0 {
            let can_collect = true;

            if bot
                .inventory
                .lock()
                .unwrap()
                .items
                .get(obj.id as usize)
                .map_or(0, |item| item.amount)
                < 200
            {
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
}

fn place(bot: &Arc<Bot>, offset_x: i32, offset_y: i32, item_id: u32) {
    let mut pkt = TankPacket::default();
    pkt._type = ETankPacketType::NetGamePacketTileChangeRequest;
    let base_x;
    let base_y;
    {
        let position = bot.position.lock().unwrap();
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

fn punch(bot: &Arc<Bot>, offset_x: i32, offset_y: i32) {
    place(bot, offset_x, offset_y, 18);
}

fn warp(bot: &Arc<Bot>, world_name: String) {
    if bot.state.lock().unwrap().is_not_allowed_to_warp {
        return;
    }
    info!("Warping to world: {}", world_name);
    send_packet(
        bot,
        EPacketType::NetMessageGameMessage,
        format!("action|join_request\nname|{}\ninvitedWorld|0\n", world_name),
    );
}

fn talk(bot: &Arc<Bot>, message: String) {
    send_packet(
        bot,
        EPacketType::NetMessageGameMessage,
        format!("action|input\n|text|{}\n", message),
    );
}

fn leave(bot: &Arc<Bot>) {
    if is_inworld(bot) {
        send_packet(
            bot,
            EPacketType::NetMessageGameMessage,
            "action|quit_to_exit\n".to_string(),
        );
    }
}

fn walk(bot: &Arc<Bot>, x: i32, y: i32, ap: bool) {
    if !ap {
        let mut position = bot.position.lock().unwrap();
        position.x += (x * 32) as f32;
        position.y += (y * 32) as f32;
    }

    let mut pkt = TankPacket::default();
    {
        let position = bot.position.lock().unwrap();
        pkt._type = ETankPacketType::NetGamePacketState;
        pkt.vector_x = position.x;
        pkt.vector_y = position.y;
        pkt.int_x = -1;
        pkt.int_y = -1;
        pkt.flags |= (1 << 1) | (1 << 5);
    }

    if bot.state.lock().unwrap().is_running && is_inworld(bot) {
        send_packet_raw(bot, &pkt);
    }
}
