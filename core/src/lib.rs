use crate::astar::AStar;
use crate::inventory::Inventory;
use crate::types::bot::{Automation, DelayConfig, Info, Scripting, State, TemporaryData, World};
use crate::types::flags::PacketFlag;
use crate::types::login_info::LoginInfo;
use crate::types::net_game_packet::{NetGamePacket, NetGamePacketData};
use crate::types::net_message::NetMessage;
use gtitem_r::structs::ItemDatabase;
use rusty_enet::Packet;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::str::FromStr;
use std::sync::atomic::AtomicI32;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

mod inventory;
mod login;
mod packet_handler;
mod server;
mod types;
mod utils;
mod variant_handler;
mod lua;
mod astar;

type TokenFetcher = Box<dyn Fn(String) -> String + Send + Sync>;

pub struct Bot {
    pub host: Mutex<rusty_enet::Host<UdpSocket>>,
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
}

impl Bot {
    pub fn new(
        payload: Vec<String>,
        token_fetcher: Option<TokenFetcher>,
        item_database: Arc<RwLock<ItemDatabase>>,
    ) -> Arc<Self> {
        let socket =
            UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))).unwrap();
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
        )
        .expect("Failed to create host");

        Arc::new(Self {
            host: Mutex::new(host),
            peer_id: Mutex::new(None),
            info: Info {
                payload,
                login_method: types::bot::ELoginMethod::GOOGLE,
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
                let server_data = server::get_server_data(false, info_data);
                match server_data {
                    Ok(data) => {
                        info_data.meta = data.meta.clone();
                        let mut server = self.info.server_data.lock().unwrap();
                        *server = Some(data.clone());
                        let dashboard_data = server::get_dashboard(&data.loginurl, info_data)
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
        match host.connect(socket_address, 2, 0) {
            Err(err) => {
                panic!("Failed to connect to server: {}", err);
            }
            _ => {}
        }
    }

    pub fn get_token(&self) {
        let (ltoken, login_data) = {
            let login_info_lock = self.info.login_info.lock().unwrap();
            let login_info = login_info_lock.as_ref().expect("Login info not set");
            (login_info.ltoken.clone(), login_info.to_string())
        };

        if let Ok(ltoken) = server::check_token(&ltoken, &login_data) {
            println!("Refreshed token: {}", ltoken);
            let mut login_info_lock = self.info.login_info.lock().unwrap();
            let login_info = login_info_lock.as_mut().expect("Login info not set");
            login_info.ltoken = ltoken;
            return;
        }

        let dashboard_links = self.info.dashboard_links.lock().unwrap();
        let urls = dashboard_links.as_ref();

        if let Some(token_fetcher) = &self.token_fetcher {
            let login_method = &self.info.login_method;
            let url = match login_method {
                types::bot::ELoginMethod::APPLE => urls.and_then(|links| links.apple.clone()),
                types::bot::ELoginMethod::GOOGLE => urls.and_then(|links| links.google.clone()),
                types::bot::ELoginMethod::LEGACY => urls.and_then(|links| links.growtopia.clone()),
                _ => None,
            };

            let token = token_fetcher(url.unwrap());
            let mut login_info_lock = self.info.login_info.lock().unwrap();
            let login_info = login_info_lock.as_mut().expect("Login info not set");
            login_info.ltoken = token.clone();
            return;
        }

        let token = match self.info.login_method {
            types::bot::ELoginMethod::LEGACY => {
                let payload = self.info.payload.clone();
                let username = payload.get(0).expect("Username not found");
                let password = payload.get(1).expect("Password not found");
                let growtopia_url = urls.as_ref().and_then(|links| links.growtopia.clone());
                login::get_legacy_token(&growtopia_url.unwrap(), username, password)
                    .expect("Failed to get legacy token")
            }
            _ => {
                todo!("Implement token retrieval for different login methods");
            }
        };

        let mut login_info_lock = self.info.login_info.lock().unwrap();
        let login_info = login_info_lock.as_mut().expect("Login info not set");
        login_info.ltoken = token.clone();
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
        let peer = host_guard.peer_mut(peer_id);

        if let Err(err) = peer.send(0, &enet_packet) {
            println!("Failed to send packet: {}", err);
        }
    }

    fn disconnect(&self) {
        let peer_id = self.peer_id.lock().unwrap().clone();
        if let Some(peer_id) = peer_id {
            if let Ok(mut host) = self.host.lock() {
                let peer = host.peer_mut(peer_id);
                peer.disconnect(0);
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
        let is_running = {
            let running = self.is_running.lock().unwrap();
            *running
        };

        while is_running {
            self.connect_to_server();

            loop {
                let event = {
                    let mut host = self.host.lock().unwrap();
                    let event = host.service();
                    event.ok().flatten().map(|e| e.no_ref())
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
            let astar = self.astar.lock().expect("Failed to lock astar");
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
}