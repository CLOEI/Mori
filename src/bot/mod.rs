mod packet_handler;

use byteorder::{ByteOrder, LittleEndian};
use enet::{
    Address, BandwidthLimit, ChannelLimit, Enet, EventKind, Host, Packet, PacketMode, PeerID,
};
use paris::{info, warn};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
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

pub struct Bot {
    pub info: Info,
    pub state: State,
    pub server: Server,
    pub position: Vector2,
    pub host: Host<()>,
    pub peer_id: Option<PeerID>,
}
unsafe impl Send for Bot {}

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
            info: Info {
                username,
                password,
                recovery_code,
                login_method,
                login_info: LoginInfo::new(),
                ..Default::default()
            },
            state: State::default(),
            server: Server::default(),
            position: Vector2::default(),
            host,
            peer_id: None,
        }
    }
}

pub fn logon(bot: &Arc<Mutex<Bot>>) {
    to_http(&bot);
    spoof(&bot);
    let (ip, port) = {
        let mut b = bot.lock().unwrap();
        b.state.is_running = true;
        b.info.login_info.meta = b.info.server_data["meta"].clone();

        (
            b.info.server_data.get("server").unwrap().clone(),
            b.info.server_data.get("port").unwrap().clone(),
        )
    };
    connect_to_server(&bot, ip, port);
    process_events(&bot);
}

pub fn spoof(bot: &Arc<Mutex<Bot>>) {
    info!("Spoofing bot data");
    let mut b = bot.lock().unwrap();
    b.info.login_info.klv = proton::generate_klv(
        &b.info.login_info.protocol,
        &b.info.login_info.game_version,
        &b.info.login_info.rid,
    );
    b.info.login_info.hash =
        proton::hash_string(format!("{}RT", b.info.login_info.mac).as_str()).to_string();
    b.info.login_info.hash2 =
        proton::hash_string(format!("{}RT", random::hex(16, true)).as_str()).to_string();
}

pub fn to_http(bot: &Arc<Mutex<Bot>>) {
    info!("Fetching server data");
    let req = ureq::post("https://www.growtopia1.com/growtopia/server_data.php").set(
        "User-Agent",
        "UbiServices_SDK_2022.Release.9_PC64_ansi_static",
    );

    let res = req.send_string("").unwrap();
    let body = res.into_string().unwrap();
    parse_server_data(&bot, body);
}

pub fn parse_server_data(bot: &Arc<Mutex<Bot>>, data: String) {
    info!("Parsing server data");
    let mut b = bot.lock().unwrap();
    b.info.server_data = data
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

fn connect_to_server(bot: &Arc<Mutex<Bot>>, ip: String, port: String) {
    info!("Connecting to the server {}:{}", ip, port);
    let mut b = bot.lock().unwrap();
    b.host
        .connect(
            &Address::new(ip.parse().unwrap(), port.parse().unwrap()),
            2,
            0,
        )
        .expect("Failed to connect to the server");
}

fn process_events(bot: &Arc<Mutex<Bot>>) {
    loop {
        let (event_kind, new_peer_id) = {
            let mut b = bot.lock().unwrap();
            let e = b
                .host
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
            let mut b = bot.lock().unwrap();
            b.peer_id = Some(peer_id);
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

fn disconnect(bot: &Arc<Mutex<Bot>>) {
    let mut b = bot.lock().unwrap();
    let peer_id = b.peer_id.unwrap().clone();
    b.host.peer_mut(peer_id).unwrap().disconnect(0);
}

fn send_packet(bot: &Arc<Mutex<Bot>>, packet_type: EPacketType, message: String) {
    let mut b = bot.lock().unwrap();
    let mut packet_data = Vec::new();
    packet_data.extend_from_slice(&(packet_type as u32).to_le_bytes());
    packet_data.extend_from_slice(&message.as_bytes());
    let pkt = Packet::new(packet_data, PacketMode::ReliableSequenced).unwrap();
    let peer_id = b.peer_id.unwrap().clone();
    let peer = b.host.peer_mut(peer_id).unwrap();
    peer.send_packet(pkt, 0).unwrap();
}
