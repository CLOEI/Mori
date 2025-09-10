use crate::astar::AStar;
use crate::inventory::Inventory;
use crate::types::bot::{Automation, DelayConfig, Info, LoginVia, Scripting, State, TemporaryData, World};
use crate::types::flags::PacketFlag;
use crate::types::login_info::LoginInfo;
use crate::types::net_game_packet::{NetGamePacket, NetGamePacketData};
use crate::types::net_message::NetMessage;
use gtitem_r::structs::ItemDatabase;
use rusty_enet::Packet;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use crate::socks5_udp::Socks5UdpSocket;
use std::str::FromStr;
use std::sync::atomic::AtomicI32;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

mod inventory;
mod login;
mod packet_handler;
mod server;
pub mod socks5_udp;
pub mod types;
mod utils;
mod variant_handler;
mod lua;
mod astar;

pub use gtitem_r;

type TokenFetcher = Box<dyn Fn(String, String) -> String + Send + Sync>;

#[derive(Debug, Clone)]
pub struct Socks5Config {
    pub proxy_addr: SocketAddr,
    pub username: Option<String>,
    pub password: Option<String>,
}

pub enum BotSocket {
    Direct(rusty_enet::Host<UdpSocket>),
    Socks5(rusty_enet::Host<Socks5UdpSocket>),
}

pub struct Bot {
    pub host: Mutex<BotSocket>,
    pub peer_id: Mutex<Option<rusty_enet::PeerID>>,
    pub info: Info,
    pub state: Mutex<State>,
    pub position: RwLock<(f32, f32)>,
    pub logs: RwLock<Vec<String>>,
    pub duration: Mutex<Instant>,
    pub net_id: Mutex<u32>,
    pub is_running: Mutex<bool>,
    pub is_redirecting: Mutex<bool>,
    pub item_database: Arc<RwLock<ItemDatabase>>,
    pub world: World,
    pub inventory: Mutex<Inventory>,
    pub gems: AtomicI32,
    pub token_fetcher: Option<TokenFetcher>,
    pub scripting: Scripting,
    pub delay_config: Mutex<DelayConfig>,
    pub automation: Mutex<Automation>,
    pub astar: Mutex<AStar>,
    pub temporary_data: TemporaryData,
    pub proxy_url: Option<String>,
}

impl Bot {
    pub fn new(
        login_via: types::bot::LoginVia,
        token_fetcher: Option<TokenFetcher>,
        item_database: Arc<RwLock<ItemDatabase>>,
        socks5_config: Option<Socks5Config>,
    ) -> Arc<Self> {
        let local_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0));
        
        let proxy_url = socks5_config.as_ref().map(|cfg| {
            if let (Some(username), Some(password)) = (&cfg.username, &cfg.password) {
                format!("socks5://{}:{}@{}:{}", username, password, cfg.proxy_addr.ip(), cfg.proxy_addr.port())
            } else {
                format!("socks5://{}:{}", cfg.proxy_addr.ip(), cfg.proxy_addr.port())
            }
        });
        
        let host = if let Some(socks5_cfg) = socks5_config {
            let socks5_socket = Socks5UdpSocket::bind_through_proxy(
                local_addr,
                socks5_cfg.proxy_addr,
                socks5_cfg.username.as_deref(),
                socks5_cfg.password.as_deref(),
            ).expect("Failed to create SOCKS5 socket");
            
            let host = rusty_enet::Host::<Socks5UdpSocket>::new(
                socks5_socket,
                rusty_enet::HostSettings {
                    peer_limit: 1,
                    channel_limit: 2,
                    compressor: Some(Box::new(rusty_enet::RangeCoder::new())),
                    checksum: Some(Box::new(rusty_enet::crc32)),
                    using_new_packet: true,
                    ..Default::default()
                },
            ).expect("Failed to create SOCKS5 host");
            
            BotSocket::Socks5(host)
        } else {
            let socket = UdpSocket::bind(local_addr).expect("Failed to bind UDP socket");
            
            let host = rusty_enet::Host::<UdpSocket>::new(
                socket,
                rusty_enet::HostSettings {
                    peer_limit: 1,
                    channel_limit: 2,
                    compressor: Some(Box::new(rusty_enet::RangeCoder::new())),
                    checksum: Some(Box::new(rusty_enet::crc32)),
                    using_new_packet: true,
                    ..Default::default()
                },
            ).expect("Failed to create direct host");
            
            BotSocket::Direct(host)
        };

        Arc::new(Self {
            host: Mutex::new(host),
            peer_id: Mutex::new(None),
            info: Info {
                login_via,
                login_info: Mutex::new(None),
                server_data: Mutex::new(None),
                dashboard_links: Mutex::new(None),
            },
            position: RwLock::new((0.0, 0.0)),
            state: Mutex::new(State::default()),
            logs: RwLock::new(Vec::new()),
            duration: Mutex::new(Instant::now()),
            net_id: Mutex::new(0),
            is_running: Mutex::new(true),
            is_redirecting: Mutex::new(false),
            world: World::default(),
            item_database,
            inventory: Mutex::new(Inventory::new()),
            gems: AtomicI32::new(0),
            token_fetcher,
            scripting: Scripting::default(),
            delay_config: Mutex::new(DelayConfig::default()),
            automation: Mutex::new(Automation::default()),
            astar: Mutex::new(AStar::new()),
            temporary_data: TemporaryData::default(),
            proxy_url,
        })
    }

    pub fn logon(self: Arc<Self>, data: Option<&str>) {
        lua::initialize(&self);
        if data.is_some() {
            todo!("Implement logon with pre-existing credentials");
        } else {
            let mut info = self.info.login_info.lock().unwrap();
            *info = Some(LoginInfo::new());
        }
        self.process_event();
    }

    pub fn connect_to_server(&self) {
        if !*self.is_redirecting.lock().unwrap() {
            {
                let mut login_info = self.info.login_info.lock().unwrap();
                let info_data = login_info.as_mut().expect("Login info not set");
                let server_data = server::get_server_data_with_proxy(false, info_data, self.proxy_url.as_deref());
                match server_data {
                    Ok(data) => {
                        info_data.meta = data.meta.clone();
                        let mut server = self.info.server_data.lock().unwrap();
                        *server = Some(data.clone());
                        let dashboard_data = server::get_dashboard_with_proxy(&data.loginurl, info_data, self.proxy_url.as_deref())
                            .expect("Failed to get dashboard data");
                        let mut dashboard = self.info.dashboard_links.lock().unwrap();
                        *dashboard = Some(dashboard_data);
                    }
                    Err(e) => {
                        todo!("Handle error: {}", e);
                    }
                }
            }
            self.get_token();
        }

        let server_data = self.info.server_data.lock().unwrap();
        let server = server_data.as_ref().expect("Server data not set");

        let socket_address =
            SocketAddr::from_str(&format!("{}:{}", server.server, server.port)).unwrap();

        let mut host = self.host.lock().unwrap();
        match &mut *host {
            BotSocket::Direct(h) => {
                match h.connect(socket_address, 2, 0) {
                    Err(err) => {
                        panic!("Failed to connect to server: {}", err);
                    }
                    _ => {}
                }
            }
            BotSocket::Socks5(h) => {
                match h.connect(socket_address, 2, 0) {
                    Err(err) => {
                        panic!("Failed to connect to server: {}", err);
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn get_token(&self) {
        let (ltoken, login_data) = {
            let login_info_lock = self.info.login_info.lock().unwrap();
            let login_info = login_info_lock.as_ref().expect("Login info not set");
            (login_info.ltoken.clone(), login_info.to_string())
        };

        if let Ok(ltoken) = server::check_token_with_proxy(&ltoken, &login_data, self.proxy_url.as_deref()) {
            println!("Refreshed token: {}", ltoken);
            let mut login_info_lock = self.info.login_info.lock().unwrap();
            let login_info = login_info_lock.as_mut().expect("Login info not set");
            login_info.ltoken = ltoken;
            return;
        }

        let dashboard_links = self.info.dashboard_links.lock().unwrap();
        let urls = dashboard_links.as_ref();

        if let Some(token_fetcher) = &self.token_fetcher {
            let login_via = &self.info.login_via;
            let url = match login_via {
                LoginVia::APPLE => urls.and_then(|links| links.apple.clone()),
                LoginVia::GOOGLE => urls.and_then(|links| links.google.clone()),
                LoginVia::LEGACY(_) => urls.and_then(|links| links.growtopia.clone()),
                LoginVia::LTOKEN(_) => None,
            };

            if let Some(url) = url {
                let token = token_fetcher("place_holder".to_string(), url);
                let mut login_info_lock = self.info.login_info.lock().unwrap();
                let login_info = login_info_lock.as_mut().expect("Login info not set");
                login_info.ltoken = token.clone();
                return;
            }
        }

        let token = match &self.info.login_via {
            LoginVia::LEGACY(credentials) => {
                let username = &credentials[0];
                let password = &credentials[1];
                let growtopia_url = urls.as_ref().and_then(|links| links.growtopia.clone());
                login::get_legacy_token_with_proxy(&growtopia_url.unwrap(), username, password, self.proxy_url.as_deref())
                    .expect("Failed to get legacy token")
            }
            _ => todo!("Login method not implemented"),

        };

        let mut login_info_lock = self.info.login_info.lock().unwrap();
        let login_info = login_info_lock.as_mut().expect("Login info not set");
        login_info.ltoken = token.clone();
    }

    pub fn execute_lua(&self, lua_code: String) {
        let lua = &self.scripting.lua;
        if let Err(err) = lua.load(&lua_code).exec() {
            println!("Failed to execute Lua code: {}", err);
        }
    }

    pub fn send_packet(
        &self,
        packet_type: NetMessage,
        packet_data: &[u8],
        extended_data: Option<&[u8]>,
        reliable: bool,
    ) {
        const MAX_PACKET_SIZE: usize = 1_000_000;

        if packet_data.len() > MAX_PACKET_SIZE {
            println!(
                "Error: Attempted to send huge packet of size {}",
                packet_data.len()
            );
            return;
        }

        let mut final_payload = Vec::new();
        let mut is_special_case = false;

        if let NetMessage::GamePacket = packet_type {
            if packet_data.len() >= 16 {
                let flags_bytes: [u8; 4] = packet_data[12..16]
                    .try_into()
                    .expect("Slice with incorrect length");
                let flags = u32::from_le_bytes(flags_bytes);

                if (flags & 8) != 0 {
                    is_special_case = true;
                }
            }
        }

        final_payload.extend_from_slice(&(packet_type as u32).to_le_bytes());
        final_payload.extend_from_slice(packet_data);

        if is_special_case {
            if let Some(ext_data) = extended_data {
                if packet_data.len() >= 56 {
                    let len_bytes: [u8; 4] = packet_data[52..56]
                        .try_into()
                        .expect("Slice with incorrect length");
                    let extended_len = u32::from_le_bytes(len_bytes) as usize;

                    final_payload.extend_from_slice(&ext_data[..extended_len]);
                }
            }
        }

        let enet_packet = if reliable {
            Packet::reliable(final_payload)
        } else {
            Packet::unreliable(final_payload)
        };

        let peer_id_guard = self.peer_id.lock().expect("Failed to lock peer_id");
        let Some(peer_id) = *peer_id_guard else {
            println!("Cannot send packet: No active peer connection.");
            return;
        };

        let mut host_guard = self.host.lock().expect("Failed to lock host");
        let send_result = match &mut *host_guard {
            BotSocket::Direct(h) => {
                let peer = h.peer_mut(peer_id);
                peer.send(0, &enet_packet)
            },
            BotSocket::Socks5(h) => {
                let peer = h.peer_mut(peer_id);
                peer.send(0, &enet_packet)
            },
        };

        if let Err(err) = send_result {
            println!("Failed to send packet: {}", err);
        }
    }

    fn disconnect(&self) {
        let peer_id = self.peer_id.lock().unwrap().clone();
        if let Some(peer_id) = peer_id {
            if let Ok(mut host) = self.host.lock() {
                match &mut *host {
                    BotSocket::Direct(h) => {
                        let peer = h.peer_mut(peer_id);
                        peer.disconnect(0);
                    },
                    BotSocket::Socks5(h) => {
                        let peer = h.peer_mut(peer_id);
                        peer.disconnect(0);
                    },
                };
            }
        }
    }

    pub fn set_auto_collect(&self, enabled: bool) {
        let mut automation = self.automation.lock().unwrap();
        automation.auto_collect = enabled;
    }

    pub fn set_auto_reconnect(&self, enabled: bool) {
        let mut automation = self.automation.lock().unwrap();
        automation.auto_reconnect = enabled;
    }

    pub fn set_findpath_delay(&self, delay: u32) {
        let mut delay_config = self.delay_config.lock().unwrap();
        delay_config.findpath_delay = delay;
    }

    pub fn set_punch_delay(&self, delay: u32) {
        let mut delay_config = self.delay_config.lock().unwrap();
        delay_config.punch_delay = delay;
    }

    pub fn set_place_delay(&self, delay: u32) {
        let mut delay_config = self.delay_config.lock().unwrap();
        delay_config.place_delay = delay;
    }

    fn process_event(&self) {
        loop {
            let is_running = {
                let running = self.is_running.lock().unwrap();
                *running
            };
            
            if !is_running {
                break;
            }
            self.connect_to_server();

            loop {
                let event = {
                    let mut host = self.host.lock().unwrap();
                    match &mut *host {
                        BotSocket::Direct(h) => {
                            h.service().ok().flatten().map(|e| e.no_ref())
                        }
                        BotSocket::Socks5(h) => {
                            h.service().ok().flatten().map(|e| e.no_ref())
                        }
                    }
                };

                if let Some(event) = event {
                    match event {
                        rusty_enet::EventNoRef::Connect { peer, .. } => {
                            println!("Connected to server");
                            let mut peer_id_lock = self.peer_id.lock().unwrap();
                            *peer_id_lock = Some(peer);
                        }
                        rusty_enet::EventNoRef::Receive {
                            peer,
                            channel_id,
                            packet,
                        } => {
                            let data = packet.data();
                            if data.len() < 4 {
                                continue;
                            }
                            packet_handler::handle(self, data);
                        }
                        rusty_enet::EventNoRef::Disconnect { peer, data } => {
                            println!("Disconnected from server");
                            break;
                        }
                    }
                }
            }
        }
    }
}

// packet methods
impl Bot {
    pub fn say(&self, message: String) {
        self.send_packet(
            NetMessage::GenericText,
            format!("action|input\n|text|{}\n", message).as_bytes(),
            None,
            true,
        );
    }

    pub fn warp(&self, world_name: String) {
        self.send_packet(
            NetMessage::GameMessage,
            format!("action|join_request\nname|{}\ninvitedWorld|0\n", world_name).as_bytes(),
            None,
            true,
        );
    }

    pub fn place(&self, offset_x: i32, offset_y: i32, item_id: u32) {
        let mut pkt = NetGamePacketData::default();
        pkt._type = NetGamePacket::TileChangeRequest;
        let (base_x, base_y) = {
            let position = self.position.read().unwrap();
            pkt.vector_x = position.0;
            pkt.vector_y = position.1;
            pkt.int_x = (position.0 / 32.0).floor() as i32 + offset_x;
            pkt.int_y = (position.1 / 32.0).floor() as i32 + offset_y;
            pkt.value = item_id;

            (
                (position.0 / 32.0).floor() as i32,
                (position.1 / 32.0).floor() as i32,
            )
        };

        if pkt.int_x <= base_x + 4
            && pkt.int_x >= base_x - 4
            && pkt.int_y <= base_y + 4
            && pkt.int_y >= base_y - 4
        {
            self.send_packet(
                NetMessage::GamePacket,
                pkt.to_bytes().as_slice(),
                None,
                true,
            );
            pkt.flags = PacketFlag::PLACE | PacketFlag::STANDING;
            if base_x > pkt.int_x {
                pkt.flags |= PacketFlag::FACING_LEFT;
            }
            pkt._type = NetGamePacket::State;
            self.send_packet(
                NetMessage::GamePacket,
                pkt.to_bytes().as_slice(),
                None,
                true,
            );
        }
    }

    pub fn punch(&self, offset_x: i32, offset_y: i32) {
        self.place(offset_x, offset_y, 18);
    }

    pub fn wrench(&self, offset_x: i32, offset_y: i32) {
        self.place(offset_x, offset_y, 32);
    }

    pub fn wear(&self, item_id: u32) {
        let packet = NetGamePacketData {
            _type: NetGamePacket::ItemActivateRequest,
            value: item_id,
            ..Default::default()
        };

        self.send_packet(
            NetMessage::GamePacket,
            packet.to_bytes().as_slice(),
            None,
            true,
        );
    }

    pub fn walk(&self, x: i32, y: i32, ap: bool) {
        if !ap {
            let mut position = self.position.write().expect("Failed to lock position");
            *position = (position.0 + (x * 32) as f32, position.1 + (y * 32) as f32);
        }

        let position = {
            let position = self
                .position
                .read()
                .expect("Failed to lock position")
                .clone();

            position
        };

        let mut pkt = NetGamePacketData::default();
        {
            pkt._type = NetGamePacket::State;
            pkt.vector_x = position.0;
            pkt.vector_y = position.1;
            pkt.int_x = -1;
            pkt.int_y = -1;
            pkt.flags = PacketFlag::WALK | PacketFlag::STANDING;
            

            let face_left = if position.0 < x as f32 * 32.0 {
                false
            } else {
                true
            };

            pkt.flags.set(PacketFlag::FACING_LEFT, face_left);
        }

        self.send_packet(
            NetMessage::GamePacket,
            pkt.to_bytes().as_slice(),
            None,
            true,
        );
    }

    pub fn find_path(&self, x: u32, y: u32) {
        let position = {
            let position = self.position.read().expect("Failed to lock position");
            position.clone()
        };

        let paths = {
            let mut astar = self.astar.lock().expect("Failed to lock astar");
            astar.find_path((position.0 as u32) / 32, (position.1 as u32) / 32, x, y)
        };

        let findpath_delay = {
            let findpath_delay = self
                .delay_config
                .lock()
                .unwrap()
                .findpath_delay;
            findpath_delay
        };

        let delay = findpath_delay;
        if let Some(paths) = paths {
            for node in paths {
                {
                    let mut position = self.position.write().expect("Failed to lock position");
                    position.0 = node.x as f32 * 32.0;
                    position.1 = node.y as f32 * 32.0;
                }
                self.walk(node.x as i32, node.y as i32, true);
                thread::sleep(Duration::from_millis(delay as u64));
            }
        }
    }

    pub fn drop_item(&self, item_id: u32, amount: u32) {
        self.send_packet(
            NetMessage::GenericText,
            format!("action|drop\n|itemID|{}\n", item_id).as_bytes(),
            None,
            true,
        );
        let mut drop = self.temporary_data.drop.lock().unwrap();
        let mut dialog_callback = self.temporary_data.dialog_callback.lock().unwrap();
        *drop = (item_id, amount);

        *dialog_callback = Some(|bot| {
            let mut drop = bot.temporary_data.drop.lock().unwrap();
            let mut dialog_callback = bot.temporary_data.dialog_callback.lock().unwrap();
            bot.send_packet(
                NetMessage::GenericText,
                format!(
                    "action|dialog_return\ndialog_name|drop_item\nitemID|{}|\ncount|{}\n",
                    drop.0, drop.1
                ).as_bytes(),
                None,
                true,
            );
            *drop = (0, 0);
            *dialog_callback = None;
        });
    }

    pub fn trash_item(&self, item_id: u32, amount: u32) {
        self.send_packet(
            NetMessage::GenericText,
            format!("action|trash\n|itemID|{}\n", item_id).as_bytes(),
            None,
            true,
        );
        let mut trash = self.temporary_data.trash.lock().unwrap();
        let mut dialog_callback = self.temporary_data.dialog_callback.lock().unwrap();
        *trash = (item_id, amount);

        *dialog_callback = Some(|bot| {
            let mut trash = bot.temporary_data.trash.lock().unwrap();
            let mut dialog_callback = bot.temporary_data.dialog_callback.lock().unwrap();
            bot.send_packet(
                NetMessage::GenericText,
                format!(
                    "action|dialog_return\ndialog_name|trash_item\nitemID|{}|\ncount|{}\n",
                    trash.0, trash.1
                ).as_bytes(),
                None,
                true,
            );
            *trash = (0, 0);
            *dialog_callback = None;
        });
    }

    pub fn collect(&self) -> usize {
        let is_in_world = {
            match self.world.data.try_lock() {
                Ok(world) => world.name != "EXIT",
                Err(_) => return 0,
            }
        };

        if !is_in_world {
            return 0;
        }

        let bot_position = {
            match self.position.try_read() {
                Ok(pos) => (pos.0, pos.1),
                Err(_) => return 0,
            }
        };

        let bot_tile_x = bot_position.0 / 32.0;
        let bot_tile_y = bot_position.1 / 32.0;

        let inventory_state = {
            match self.inventory.try_lock() {
                Ok(inv) => {
                    let mut item_amounts = std::collections::HashMap::with_capacity(inv.items.len());
                    for (&item_id, item) in &inv.items {
                        item_amounts.insert(item_id, item.amount);
                    }
                    (inv.size, inv.items.len() as u32, item_amounts)
                },
                Err(_) => return 0, 
            }
        };

        let (inventory_size, current_item_count, item_amounts) = inventory_state;

        let collectible_items = {
            match self.world.data.try_lock() {
                Ok(world) => {
                    let mut nearby_items = Vec::with_capacity(std::cmp::min(world.dropped.items.len(), 50));
                    
                    for item in &world.dropped.items {
                        let dx = bot_tile_x - item.x;
                        let dy = bot_tile_y - item.y;
                        let distance_squared = dx * dx + dy * dy;
                        
                        if distance_squared <= 25.0 {
                            nearby_items.push((item.clone(), distance_squared));
                        }
                    }
                    
                    nearby_items.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
                    nearby_items.into_iter().map(|(item, _)| item).collect::<Vec<_>>()
                },
                Err(_) => return 0,
            }
        };

        if collectible_items.is_empty() {
            return 0;
        }

        let mut collected_count = 0;
        let mut packets_to_send = Vec::with_capacity(collectible_items.len());

        for item in collectible_items {
            let can_collect = if let Some(&current_amount) = item_amounts.get(&item.id) {
                current_amount < 200
            } else {
                current_item_count < inventory_size
            };

            if can_collect {
                let packet = NetGamePacketData {
                    _type: NetGamePacket::ItemActivateObjectRequest,
                    vector_x: item.x,
                    vector_y: item.y,
                    value: item.uid,
                    ..Default::default()
                };

                packets_to_send.push(packet);
                collected_count += 1;
            }
        }

        for packet in packets_to_send {
            self.send_packet(
                NetMessage::GamePacket,
                &packet.to_bytes(),
                None,
                true,
            );
        }

        collected_count
    }
}