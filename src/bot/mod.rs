mod astar;
mod inventory;
mod login;
mod packet_handler;
mod variant_handler;

use crate::types::bot_info::{Info, Position, Server, State};
use crate::types::e_login_method::ELoginMethod;
use crate::types::e_tank_packet_type::ETankPacketType;
use crate::types::login_info::LoginInfo;
use crate::types::tank_packet_type::TankPacketType;
use crate::utils::proton::hash_string;
use crate::utils::random::random_hex;
use crate::{types::e_packet_type::EPacketType, utils::proton::generate_klv};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use astar::AStar;
use byteorder::{ByteOrder, LittleEndian};
use enet::*;
use gtitem_r::structs::ItemDatabase;
use gtworld_r::World;
use inventory::Inventory;
use spdlog::info;

static USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0";

pub struct Bot {
    pub info: Info,
    pub state: State,
    pub position: Position,
    pub server: Server,
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
            info: Info {
                username,
                password,
                code,
                method,
                login_info: LoginInfo::new(),
                ..Default::default()
            },
            state: Default::default(),
            position: Default::default(),
            server: Default::default(),
            world: World::new(Arc::clone(&item_database)),
            inventory: Inventory::new(),
            astar: AStar::new(Arc::clone(&item_database)),
        }
    }
}

pub fn login(bot_mutex: Arc<Mutex<Bot>>) {
    to_http(&bot_mutex);
    if bot_mutex.lock().unwrap().info.method != ELoginMethod::UBISOFT {
        match get_oauth_links(&bot_mutex) {
            Ok(links) => {
                let mut bot = bot_mutex.lock().unwrap();
                bot.info.oauth_links = links;
                info!("Successfully got OAuth links for: apple, google and legacy");
            }
            Err(err) => {
                info!("Failed to get OAuth links: {}", err);
                return;
            }
        }
    }
    get_token(&bot_mutex);
    let mut bot = bot_mutex.lock().unwrap();
    bot.state.is_running = true;
    bot.info.login_info.meta = bot.info.parsed_server_data["meta"].clone();

    bot.info.login_info.klv = generate_klv(
        &bot.info.login_info.protocol,
        &bot.info.login_info.game_version,
        &bot.info.login_info.rid,
    );
    bot.info.login_info.hash =
        hash_string(format!("{}RT", bot.info.login_info.mac).as_str()).to_string();
    bot.info.login_info.hash2 =
        hash_string(format!("{}RT", random_hex(16, true)).as_str()).to_string();
    drop(bot);
    start_event_loop(&bot_mutex);
}

pub fn start_event_loop(bot_mutex: &Arc<Mutex<Bot>>) {
    let enet = Enet::new().expect("Failed to initialize ENet");

    loop {
        {
            let bot = bot_mutex.lock().unwrap();
            if !bot.state.is_running {
                break;
            }
        }

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
        if bot_mutex.lock().unwrap().state.is_redirect {
            let bot = bot_mutex.lock().unwrap();
            info!("Redirecting to {}:{}...", &bot.server.ip, &bot.server.port);
            connect_to_server(&mut enet_host, &bot.server.ip, &bot.server.port);
        } else {
            if bot_mutex.lock().unwrap().state.is_ingame {
                get_token(&bot_mutex);
            }
            to_http(&bot_mutex);
            let bot = bot_mutex.lock().unwrap();
            info!(
                "Connecting to {}:{}",
                bot.info.parsed_server_data["server"], bot.info.parsed_server_data["port"]
            );
            connect_to_server(
                &mut enet_host,
                &bot.info.parsed_server_data["server"],
                &bot.info.parsed_server_data["port"],
            );
        }
        loop {
            match enet_host.service(1000).expect("Service failed") {
                Some(Event::Connect(ref mut sender)) => {
                    // self.set_ping(sender.mean_rtt());
                    info!("Connected to the server");
                }
                Some(Event::Disconnect(ref mut sender, ..)) => {
                    // self.set_ping(sender.mean_rtt());
                    info!("Disconnected from the server");
                    break;
                }
                Some(Event::Receive {
                    ref packet,
                    ref mut sender,
                    ..
                }) => {
                    // self.set_ping(sender.mean_rtt());
                    let data = packet.data();
                    if data.len() < 4 {
                        continue;
                    }
                    let packet_id = LittleEndian::read_u32(&data[0..4]);
                    let packet_type = EPacketType::from(packet_id);
                    packet_handler::handle(&bot_mutex, sender, packet_type, &data[4..]);
                }
                _ => (),
            }
        }
    }
}

fn connect_to_server(enet_host: &mut Host<()>, ip: &str, port: &str) {
    enet_host
        .connect(
            &Address::new(ip.parse().unwrap(), port.parse().unwrap()),
            2,
            0,
        )
        .expect("Failed to connect to the server");
}

pub fn get_token(bot: &Arc<Mutex<Bot>>) {
    let mut bot = bot.lock().unwrap();
    // TODO: Handle error, loop with delay until token is received
    info!("Getting token for {}", bot.info.username);
    match bot.info.method {
        ELoginMethod::UBISOFT => {
            let res =
                login::get_ubisoft_token(&bot.info.username, &bot.info.password, &bot.info.code)
                    .unwrap();
            bot.info.token = res;
        }
        ELoginMethod::APPLE => {
            let res = login::get_apple_token(bot.info.oauth_links[0].as_str()).unwrap();
            bot.info.token = res;
        }
        ELoginMethod::GOOGLE => {
            let res = login::get_google_token(bot.info.oauth_links[1].as_str()).unwrap();
            bot.info.token = res;
        }
        ELoginMethod::LEGACY => {
            let res = login::get_legacy_token(
                bot.info.oauth_links[2].as_str(),
                bot.info.username.as_str(),
                bot.info.password.as_str(),
            )
            .unwrap();
            bot.info.token = res;
        }
    }
    info!("Received the token: {}", bot.info.token);
}

pub fn to_http(bot_mutex: &Arc<Mutex<Bot>>) {
    let req = ureq::post("https://www.growtopia1.com/growtopia/server_data.php").set(
        "User-Agent",
        "UbiServices_SDK_2022.Release.9_PC64_ansi_static",
    );

    let res = req.send_string("").unwrap();

    let body = res.into_string().unwrap();
    parse_server_data(&bot_mutex, body);
}

pub fn find_path(bot_mutex: &Arc<Mutex<Bot>>, peer: &mut Peer<()>, x: u32, y: u32) {
    let bot = bot_mutex.lock().unwrap();
    let paths = match bot.astar.find_path(
        (bot.position.x as u32) / 32,
        (bot.position.y as u32) / 32,
        x,
        y,
    ) {
        Some(path) => path,
        None => return,
    };

    for i in 0..paths.len() {
        let node = &paths[i];
        walk(&bot_mutex, peer, node.x as f32, node.y as f32, true);
    }
}

pub fn parse_server_data(bot_mutex: &Arc<Mutex<Bot>>, data: String) {
    let mut bot = bot_mutex.lock().unwrap();
    bot.info.parsed_server_data = data
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
pub fn walk(bot_mutex: &Arc<Mutex<Bot>>, peer: &mut Peer<()>, x: f32, y: f32, ap: bool) {
    let mut bot = bot_mutex.lock().unwrap();
    if ap {
        bot.position.x = x * 32.0;
        bot.position.y = y * 32.0;
    } else {
        bot.position.x += x * 32.0;
        bot.position.y += y * 32.0;
    }

    let mut pkt = TankPacketType::new();
    let mut flags: u32 = 0;
    flags |= 1 << 1; // unknown
    flags |= 1 << 5; // is on a solid block

    pkt.packet_type = ETankPacketType::NetGamePacketState;
    pkt.vector_x = bot.position.x;
    pkt.vector_y = bot.position.y;
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

pub fn talk(peer: &mut Peer<()>, message: &str) {
    send_packet(
        peer,
        EPacketType::NetMessageGenericText,
        format!("action|input\n|text|{}\n", message),
    );
}

pub fn place(
    bot_mutex: &Arc<Mutex<Bot>>,
    peer: &mut Peer<()>,
    offset_x: i32,
    offset_y: i32,
    block_id: u32,
) {
    let bot = bot_mutex.lock().unwrap();
    let mut pkt = TankPacketType::new();

    pkt.packet_type = ETankPacketType::NetGamePacketTileChangeRequest;
    pkt.vector_x = bot.position.x;
    pkt.vector_y = bot.position.y;
    pkt.int_x = ((bot.position.x / 32.0).floor() as i32) + offset_x;
    pkt.int_y = ((bot.position.y / 32.0).floor() as i32) + offset_y;
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

    if pkt.int_x <= (bot.position.x / 32.0).floor() as i32 + 4
        && pkt.int_x >= (bot.position.x / 32.0).floor() as i32 - 4
        && pkt.int_y <= (bot.position.y / 32.0).floor() as i32 + 4
        && pkt.int_y >= (bot.position.y / 32.0).floor() as i32 - 4
    {
        let pkt = Packet::new(&packet_data, PacketMode::ReliableSequenced).unwrap();
        peer.send_packet(pkt, 0).unwrap();
    }
}

pub fn punch(bot_mutex: &Arc<Mutex<Bot>>, peer: &mut Peer<()>, offset_x: i32, offset_y: i32) {
    place(&bot_mutex, peer, offset_x, offset_y, 18)
}

pub fn warp(peer: &mut Peer<()>, world: &str) {
    send_packet(
        peer,
        EPacketType::NetMessageGameMessage,
        format!("action|join_request\nname|{}\ninvitedWorld|0\n", world),
    );
}

pub fn send_packet(peer: &mut Peer<()>, packet_type: EPacketType, message: String) {
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

pub fn disconnect(peer: &mut Peer<()>) {
    peer.disconnect(0);
}

pub fn get_oauth_links(bot_mutex: &Arc<Mutex<Bot>>) -> Result<Vec<String>, ureq::Error> {
    let bot = bot_mutex.lock().unwrap();
    let body = ureq::post("https://login.growtopiagame.com/player/login/dashboard")
            .set("User-Agent", USER_AGENT)
            .send_string(format!("tankIDName|\ntankIDPass|\nrequestedName|BoardSickle\nf|1\nprotocol|209\ngame_version|4.62\nfz|41745432\nlmode|0\ncbits|1040\nplayer_age|20\nGDPR|3\ncategory|_-5100\ntotalPlaytime|0\nklv|b351d8dacd7a776848b31c74d3d550ec61dbb9b96c3ac67aea85034a84401a87\nhash2|841545814\nmeta|{}\nfhash|-716928004\nrid|01F9EBD204B52C940285667E15C00D62\nplatformID|0,1,1\ndeviceVersion|0\ncountry|us\nhash|-1829975549\nmac|b4:8c:9d:90:79:cf\nwk|66A6ABCD9753A066E39975DED77852A8\nzf|617169524\n", bot.info.parsed_server_data["meta"]).as_str())?
            .into_string()?;

    let pattern = regex::Regex::new("https:\\/\\/login\\.growtopiagame\\.com\\/(apple|google|player\\/growid)\\/(login|redirect)\\?token=[^\"]+");
    let links = pattern
        .unwrap()
        .find_iter(&body)
        .map(|m| m.as_str().to_owned())
        .collect::<Vec<String>>();

    Ok(links)
}

pub fn set_ping(bot_mutex: &Arc<Mutex<Bot>>, ping: Duration) {
    let mut bot = bot_mutex.lock().unwrap();
    bot.info.ping = ping.as_millis() as u32;
}
