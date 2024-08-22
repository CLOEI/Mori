mod login;
mod packet_handler;
mod variant_handler;

use byteorder::{ByteOrder, LittleEndian};
use enet::{
    Address, BandwidthLimit, ChannelLimit, Enet, EventKind, Host, Packet, PacketMode, PeerID,
};
use paris::{error, info, warn};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use urlencoding::encode;

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
}

impl Bot {
    pub fn new(
        username: String,
        password: String,
        recovery_code: String,
        login_method: ELoginMethod,
    ) -> Self {
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
                username,
                password,
                recovery_code,
                login_method,
                login_info: LoginInfo::new(),
                timeout: 5,
                ..Default::default()
            })),
            state: Arc::new(Mutex::new(State::default())),
            server: Arc::new(Mutex::new(Server::default())),
            position: Arc::new(Mutex::new(Vector2::default())),
            host: Arc::new(Mutex::new(host)),
            peer_id: Arc::new(Mutex::new(None)),
        }
    }
}

pub fn logon(bot: &Arc<Bot>) {
    to_http(&bot);
    spoof(&bot);

    let meta = bot
        .info
        .lock()
        .unwrap()
        .server_data
        .get("meta")
        .unwrap()
        .clone();
    bot.state.lock().unwrap().is_running = true;
    poll(bot);
    bot.info.lock().unwrap().login_info.meta = meta;

    if bot.info.lock().unwrap().login_method != ELoginMethod::UBISOFT {
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
    process_events(&bot);
}

pub fn poll(bot: &Arc<Bot>) {
    let bot_clone = bot.clone();
    thread::spawn(move || loop {
        let state = bot_clone.state.lock().unwrap();
        if !state.is_running {
            break;
        }
        drop(state);
        thread::sleep(Duration::from_millis(20));
    });
}

pub fn sleep(bot: &Arc<Bot>) {
    let mut info = bot.info.lock().unwrap();
    while info.timeout > 0 {
        info.timeout = 1;
        drop(info);
        thread::sleep(Duration::from_secs(1));
        info = bot.info.lock().unwrap();
    }
}

pub fn get_token(bot: &Arc<Bot>) {
    info!("Getting token for bot");
    let (username, password, code, method, oauth_links) = {
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
            ))?;

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

        let res = req.send_string("").unwrap();
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
            connect_to_server(
                bot,
                server_data["server"].clone(),
                server_data["port"].clone(),
            );
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
                    _ => continue,
                }
            }
        }
    }
}

fn disconnect(bot: &Arc<Bot>) {
    let peer_id = bot.peer_id.lock().unwrap().unwrap().clone();
    bot.host
        .lock()
        .unwrap()
        .peer_mut(peer_id)
        .unwrap()
        .disconnect(0);
}

fn send_packet(bot: &Arc<Bot>, packet_type: EPacketType, message: String) {
    let mut packet_data = Vec::new();
    packet_data.extend_from_slice(&(packet_type as u32).to_le_bytes());
    packet_data.extend_from_slice(&message.as_bytes());
    let pkt = Packet::new(packet_data, PacketMode::ReliableSequenced).unwrap();
    let peer_id = bot.peer_id.lock().unwrap().unwrap().clone();
    let mut host = bot.host.lock().unwrap();
    let peer = host.peer_mut(peer_id).unwrap();
    peer.send_packet(pkt, 0).unwrap();
}
