use crate::bot_configuration::BotConfiguration;
use crate::bot_inventory::BotInventory;
use crate::game_world::GameWorld;
use crate::socks5_udp::Socks5UdpSocket;
use crate::types::bot::{LoginVia, Scripting, TemporaryData};
use crate::types::flags::PacketFlag;
use crate::types::login_info::LoginInfo;
use crate::types::net_game_packet::{NetGamePacket, NetGamePacketData};
use crate::types::net_message::NetMessage;
use gtitem_r::structs::ItemDatabase;
use rusty_enet::Packet;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::thread;
use std::time::{Duration, Instant};

mod astar;
mod authentication_context;
mod bot_configuration;
mod bot_inventory;
pub mod events;
mod game_world;
mod inventory;
mod login;
mod lua;
mod movement_controller;
mod network_session;
mod packet_handler;
mod runtime_context;
mod server;
pub mod socks5_udp;
pub mod types;
mod utils;
mod variant_handler;

pub use authentication_context::AuthenticationContext;
pub use events::{BotEvent, EventBroadcaster, EventType, LogLevel};
pub use gtitem_r;
pub use gtworld_r;
pub use movement_controller::MovementController;
pub use network_session::NetworkSession;
pub use runtime_context::RuntimeContext;

pub type TokenFetcher = Box<dyn Fn(String, String) -> String + Send + Sync>;

#[derive(Debug, Clone)]
pub struct Socks5Config {
    pub proxy_addr: SocketAddr,
    pub username: Option<String>,
    pub password: Option<String>,
}

pub struct Bot {
    pub network: NetworkSession,
    pub auth: AuthenticationContext,
    pub movement: MovementController,
    pub duration: Mutex<Instant>,
    pub world: GameWorld,
    pub inventory: BotInventory,
    pub runtime: RuntimeContext,
    pub scripting: Scripting,
    pub config: BotConfiguration,
    pub temporary_data: TemporaryData,
    pub proxy_url: Option<String>,
    pub events: EventBroadcaster,
}

impl Bot {
    pub fn new(
        login_via: types::bot::LoginVia,
        token_fetcher: Option<TokenFetcher>,
        item_database: Arc<RwLock<ItemDatabase>>,
        socks5_config: Option<Socks5Config>,
    ) -> (Arc<Self>, mpsc::Receiver<BotEvent>) {
        let local_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0));

        let proxy_url = socks5_config.as_ref().map(|cfg| {
            if let (Some(username), Some(password)) = (&cfg.username, &cfg.password) {
                format!(
                    "socks5://{}:{}@{}:{}",
                    username,
                    password,
                    cfg.proxy_addr.ip(),
                    cfg.proxy_addr.port()
                )
            } else {
                format!("socks5://{}:{}", cfg.proxy_addr.ip(), cfg.proxy_addr.port())
            }
        });

        let network = NetworkSession::new(local_addr, socks5_config);
        let (event_broadcaster, event_receiver) = events::create_event_channel();

        let bot = Arc::new(Self {
            network,
            auth: AuthenticationContext::new(login_via, token_fetcher),
            movement: MovementController::new(),
            duration: Mutex::new(Instant::now()),
            world: GameWorld::new(item_database),
            inventory: BotInventory::new(),
            runtime: RuntimeContext::new(),
            scripting: Scripting::default(),
            config: BotConfiguration::new(),
            temporary_data: TemporaryData::default(),
            proxy_url,
            events: event_broadcaster,
        });

        (bot, event_receiver)
    }

    pub fn logon(self: Arc<Self>, data: Option<&str>) {
        lua::initialize(&self);
        if data.is_some() {
            todo!("Implement logon with pre-existing credentials");
        } else {
            let mut login_info = self.auth.login_info();
            *login_info = Some(LoginInfo::new());
        }

        self.spawn_polling();
        self.process_event();
    }

    pub fn connect_to_server(&self) {
        if !self.runtime.is_redirecting() {
            {
                let mut login_info = self.auth.login_info();
                let info_data = login_info.as_mut().expect("Login info not set");
                let server_data =
                    server::get_server_data_with_proxy(false, info_data, self.proxy_url.as_deref());
                match server_data {
                    Ok(data) => {
                        info_data.meta = data.meta.clone();
                        let mut server = self.auth.server_data();
                        *server = Some(data.clone());
                        let dashboard_data = server::get_dashboard_with_proxy(
                            &data.loginurl,
                            info_data,
                            self.proxy_url.as_deref(),
                        )
                        .expect("Failed to get dashboard data");
                        let mut dashboard = self.auth.dashboard_links();
                        *dashboard = Some(dashboard_data);
                    }
                    Err(e) => {
                        todo!("Handle error: {}", e);
                    }
                }
            }
            self.get_token();
        }

        let server_address = {
            let server_data = self.auth.server_data();
            let server = server_data.as_ref().expect("Server data not set");
            SocketAddr::from_str(&format!("{}:{}", server.server, server.port)).unwrap()
        };

        self.network.connect(server_address);
    }

    pub fn get_token(&self) {
        let (ltoken, login_data) = {
            let login_info_lock = self.auth.login_info();
            let login_info = login_info_lock.as_ref().expect("Login info not set");
            (login_info.ltoken.clone(), login_info.to_string())
        };

        if let Ok(ltoken) =
            server::check_token(&ltoken, &login_data, self.proxy_url.as_deref())
        {
            println!("Refreshed token: {}", ltoken);
            let mut login_info_lock = self.auth.login_info();
            let login_info = login_info_lock.as_mut().expect("Login info not set");
            login_info.ltoken = ltoken;
            return;
        }

        let urls = self.auth.dashboard_links_clone();

        if let Some(token_fetcher) = self.auth.token_fetcher() {
            let login_via = self.auth.login_via();
            let url = match login_via {
                LoginVia::APPLE => urls.as_ref().and_then(|links| links.apple.clone()),
                LoginVia::GOOGLE => urls.as_ref().and_then(|links| links.google.clone()),
                LoginVia::LEGACY(_) => urls.as_ref().and_then(|links| links.growtopia.clone()),
                LoginVia::LTOKEN(_) => None,
            };

            if let Some(url) = url {
                let token = token_fetcher("place_holder".to_string(), url);
                let mut login_info_lock = self.auth.login_info();
                let login_info = login_info_lock.as_mut().expect("Login info not set");
                login_info.ltoken = token.clone();
                return;
            }
        }

        let token = match self.auth.login_via() {
            LoginVia::LEGACY(credentials) => {
                let username = &credentials[0];
                let password = &credentials[1];
                let growtopia_url = urls.and_then(|links| links.growtopia.clone());
                login::get_legacy_token_with_proxy(
                    &growtopia_url.unwrap(),
                    username,
                    password,
                    self.proxy_url.as_deref(),
                )
                .expect("Failed to get legacy token")
            }
            _ => todo!("Login method not implemented"),
        };

        let mut login_info_lock = self.auth.login_info();
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

        if !self.network.send(enet_packet) {
            println!("Cannot send packet: No active peer connection.");
        }
    }

    fn disconnect(&self) {
        self.network.disconnect();
    }

    pub fn set_auto_collect(&self, enabled: bool) {
        self.config.set_auto_collect(enabled);
    }

    pub fn set_auto_reconnect(&self, enabled: bool) {
        self.config.set_auto_reconnect(enabled);
    }

    pub fn set_findpath_delay(&self, delay: u32) {
        self.config.set_findpath_delay(delay);
    }

    pub fn set_punch_delay(&self, delay: u32) {
        self.config.set_punch_delay(delay);
    }

    pub fn set_place_delay(&self, delay: u32) {
        self.config.set_place_delay(delay);
    }

    fn spawn_polling(self: &Arc<Self>) {
        let bot_arc = Arc::clone(self);
        thread::spawn(move || {
            const COLLECT_INTERVAL: Duration = Duration::from_millis(500);
            const LOOP_DELAY: Duration = Duration::from_millis(100);

            loop {
                let is_running = bot_arc.runtime.is_running();

                if !is_running {
                    break;
                }

                if !bot_arc.network.is_connected() {
                    thread::sleep(LOOP_DELAY);
                    continue;
                }

                if let Some(ping) = bot_arc.network.update_ping() {
                    bot_arc.runtime.set_ping(ping);
                }

                if bot_arc.config.auto_collect() {
                    bot_arc.collect();
                }

                thread::sleep(COLLECT_INTERVAL);
            }
        });
    }

    fn process_event(&self) {
        loop {
            let is_running = self.runtime.is_running();

            if !is_running {
                break;
            }
            self.connect_to_server();

            loop {
                let event = self.network.service();

                if let Some(event) = event {
                    match event {
                        rusty_enet::EventNoRef::Connect { peer, .. } => {
                            println!("Connected to server");
                            self.network.set_peer_id(Some(peer));
                        }
                        rusty_enet::EventNoRef::Receive {
                            peer: _,
                            channel_id: _,
                            packet,
                        } => {
                            let data = packet.data();
                            if data.len() < 4 {
                                continue;
                            }
                            packet_handler::handle(self, data);
                        }
                        rusty_enet::EventNoRef::Disconnect { peer: _, data: _ } => {
                            println!("Disconnected from server");
                            self.network.set_peer_id(None);
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
        let position = self.movement.position();
        pkt.vector_x = position.0;
        pkt.vector_y = position.1;
        let base_x = (position.0 / 32.0).floor() as i32;
        let base_y = (position.1 / 32.0).floor() as i32;
        pkt.int_x = base_x + offset_x;
        pkt.int_y = base_y + offset_y;
        pkt.value = item_id;

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
            self.movement.translate((x * 32) as f32, (y * 32) as f32);
        }

        let position = self.movement.position();

        // Emit PositionChanged event
        self.events.emit(BotEvent::new(EventType::PositionChanged {
            x: position.0,
            y: position.1,
        }));

        let mut pkt = NetGamePacketData::default();
        {
            pkt._type = NetGamePacket::State;
            pkt.vector_x = position.0;
            pkt.vector_y = position.1;
            pkt.int_x = -1;
            pkt.int_y = -1;
            pkt.flags = PacketFlag::WALK | PacketFlag::STANDING;

            let face_left = position.0 >= x as f32 * 32.0;
            pkt.flags.set(PacketFlag::FACING_LEFT, face_left);
        }

        self.send_packet(
            NetMessage::GamePacket,
            pkt.to_bytes().as_slice(),
            None,
            false,
        );
    }

    pub fn find_path(&self, x: u32, y: u32) {
        let position = self.movement.position();

        // Emit PathfindingStarted event
        self.events.emit(BotEvent::new(EventType::PathfindingStarted {
            target_x: x,
            target_y: y,
        }));

        let paths = {
            let mut astar = self.movement.astar();
            astar.find_path((position.0 as u32) / 32, (position.1 as u32) / 32, x, y)
        };

        let delay = self.config.findpath_delay();
        if let Some(paths) = &paths {
            for node in paths {
                self.movement
                    .set_position(node.x as f32 * 32.0, node.y as f32 * 32.0);
                self.walk(node.x as i32, node.y as i32, true);
                thread::sleep(Duration::from_millis(delay as u64));
            }
        }

        // Emit PathfindingCompleted event
        self.events.emit(BotEvent::new(EventType::PathfindingCompleted {
            success: paths.is_some(),
            steps: paths.as_ref().map(|p| p.len()).unwrap_or(0),
        }));
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
                )
                .as_bytes(),
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
                )
                .as_bytes(),
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

        let bot_position = match self.movement.try_position() {
            Some(pos) => pos,
            None => return 0,
        };

        let bot_tile_x = bot_position.0 / 32.0;
        let bot_tile_y = bot_position.1 / 32.0;

        let inventory_snapshot = match self.inventory.try_get_snapshot() {
            Some(snapshot) => snapshot,
            None => return 0,
        };

        let inventory_size = inventory_snapshot.size;
        let current_item_count = inventory_snapshot.item_count;
        let item_amounts = inventory_snapshot.item_amounts;

        let collectible_items = {
            match self.world.data.try_lock() {
                Ok(world) => {
                    let mut nearby_items =
                        Vec::with_capacity(std::cmp::min(world.dropped.items.len(), 50));

                    for item in &world.dropped.items {
                        let dx = bot_tile_x - item.x;
                        let dy = bot_tile_y - item.y;
                        let distance_squared = dx * dx + dy * dy;

                        if distance_squared <= 25.0 {
                            nearby_items.push((item.clone(), distance_squared));
                        }
                    }

                    nearby_items
                        .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
                    nearby_items
                        .into_iter()
                        .map(|(item, _)| item)
                        .collect::<Vec<_>>()
                }
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
            self.send_packet(NetMessage::GamePacket, &packet.to_bytes(), None, true);
        }

        collected_count
    }
}

pub fn test_socks5_proxy(socks5_config: &Socks5Config) -> (bool, bool) {
    let server_data_success = test_server_data_fetch(socks5_config);
    let server_connection_success = test_server_connection(socks5_config);

    (server_data_success, server_connection_success)
}

fn test_server_data_fetch(socks5_config: &Socks5Config) -> bool {
    let proxy_url = if let (Some(username), Some(password)) =
        (&socks5_config.username, &socks5_config.password)
    {
        format!(
            "socks5://{}:{}@{}:{}",
            username,
            password,
            socks5_config.proxy_addr.ip(),
            socks5_config.proxy_addr.port()
        )
    } else {
        format!(
            "socks5://{}:{}",
            socks5_config.proxy_addr.ip(),
            socks5_config.proxy_addr.port()
        )
    };

    let login_info = LoginInfo::new();
    match server::get_server_data_with_proxy(false, &login_info, Some(&proxy_url)) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn test_server_connection(socks5_config: &Socks5Config) -> bool {
    let local_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0));

    let socks5_socket = match Socks5UdpSocket::bind_through_proxy(
        local_addr,
        socks5_config.proxy_addr,
        socks5_config.username.as_deref(),
        socks5_config.password.as_deref(),
    ) {
        Ok(socket) => socket,
        Err(_) => return false,
    };

    let mut host = match rusty_enet::Host::<Socks5UdpSocket>::new(
        socks5_socket,
        rusty_enet::HostSettings {
            peer_limit: 1,
            channel_limit: 2,
            compressor: Some(Box::new(rusty_enet::RangeCoder::new())),
            checksum: Some(Box::new(rusty_enet::crc32)),
            using_new_packet: true,
            ..Default::default()
        },
    ) {
        Ok(host) => host,
        Err(_) => return false,
    };

    let test_server_addr = match SocketAddr::from_str("213.179.209.168:17091") {
        Ok(addr) => addr,
        Err(_) => return false,
    };

    match host.connect(test_server_addr, 2, 0) {
        Ok(_) => true,
        Err(_) => false,
    }
}
