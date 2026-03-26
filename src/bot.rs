use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use rusty_enet as enet;
use crate::constants::{FHASH, GAME_VER, PROTOCOL};
use crate::crypto::{compute_klv, generate_rid, hash_string, random_hex, random_mac};
use crate::dashboard::get_dashboard_proxied;
use crate::login::{check_token, get_legacy_token_proxied, LoginError};
use crate::packet::{self, GamePacketType, GameUpdatePacket, IncomingPacket};
use crate::server_data::{get_server_data_proxied, LoginInfo};
use crate::socks5::Socks5UdpSocket;
use crate::variant::VariantList;
use crate::astar::AStar;
use crate::inventory::Inventory;
use crate::items::ItemsDat;
use crate::player::{LocalPlayer, Player, parse_pipe_map};
use crate::world::{World, TileType, WorldObject};
use crate::bot_state::{BotState, BotStatus, BotCommand, BotDelays, CmdReceiver, InvSlot, PlayerInfo, TileInfo};
use crate::events::{WsEvent, WsInvItem, WsObject, WsTile, WsTx};

#[derive(Clone, Debug)]
pub struct Socks5Config {
    pub proxy_addr: SocketAddr,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Socks5Config {
    pub fn to_url(&self) -> String {
        match (&self.username, &self.password) {
            (Some(u), Some(p)) => format!("socks5://{}:{}@{}", u, p, self.proxy_addr),
            _ => format!("socks5://{}", self.proxy_addr),
        }
    }
}

enum BotHost {
    Direct(enet::Host<UdpSocket>),
    Socks5(enet::Host<Socks5UdpSocket>),
}

impl BotHost {
    fn next_event(&mut self) -> Option<enet::EventNoRef> {
        match self {
            Self::Direct(h) => {
                if let Some(e) = h.service().expect("service failed") { Some(e.no_ref()) } else { None }
            }
            Self::Socks5(h) => {
                if let Some(e) = h.service().expect("service failed") { Some(e.no_ref()) } else { None }
            }
        }
    }

    fn connect(&mut self, addr: SocketAddr, channels: usize, data: u32) {
        match self {
            Self::Direct(h) => { h.connect(addr, channels, data).expect("connect failed"); }
            Self::Socks5(h) => { h.connect(addr, channels, data).expect("connect failed"); }
        }
    }

    fn peer_rtt(&mut self, id: enet::PeerID) -> std::time::Duration {
        match self {
            Self::Direct(h) => h.peer_mut(id).round_trip_time(),
            Self::Socks5(h) => h.peer_mut(id).round_trip_time(),
        }
    }

    fn peer_send(&mut self, id: enet::PeerID, channel: u8, packet: &enet::Packet) {
        match self {
            Self::Direct(h) => { h.peer_mut(id).send(channel, packet).ok(); }
            Self::Socks5(h) => { h.peer_mut(id).send(channel, packet).ok(); }
        }
    }

    fn peer_disconnect(&mut self, id: enet::PeerID, data: u32) {
        match self {
            Self::Direct(h) => { h.peer_mut(id).disconnect(data); }
            Self::Socks5(h) => { h.peer_mut(id).disconnect(data); }
        }
    }
}

/// Raw event pushed to `Bot::event_queue` by packet handlers.
/// Drained by Lua's `listenEvents` loop to fire registered callbacks.
pub enum BotEventRaw {
    VariantList { vl: VariantList, net_id: u32 },
    GameUpdate  { pkt: GameUpdatePacket },
    GameMessage { text: String },
}

/// Callback invoked on the next `OnDialogRequest`, then cleared.
type DialogCallback = Box<dyn FnOnce(&mut Bot) + Send>;

pub struct TemporaryData {
    pub dialog_callback: Mutex<Option<DialogCallback>>,
}

impl Default for TemporaryData {
    fn default() -> Self {
        Self { dialog_callback: Mutex::new(None) }
    }
}

/// Data captured from `OnSendToServer`, kept until the next ServerHello.
struct RedirectData {
    server:  String,
    port:    u16,
    token:   String,
    user:    String,
    door_id: String,
    uuid:    String,
    aat:     String,
}

pub struct Bot {
    host:     BotHost,
    pub proxy: Option<Socks5Config>,
    pub username: String,
    password: String,
    /// Legacy token from HTTP login (used in first ServerHello only).
    ltoken:   String,
    /// `meta` from server_data.php — echoed in all login packets.
    meta:     String,
    /// Per-session random values computed once at startup.
    pub mac:  String,
    hash:     i32,
    hash2:    i32,
    wk:       String,
    rid:      String,
    /// Set by `OnSendToServer`; consumed by the next ServerHello.
    redirect: Option<RedirectData>,
    /// When the bot connected — used for network time in ping replies.
    start_time: std::time::Instant,
    /// Current position in the world (pixels).
    pub pos_x: f32,
    pub pos_y: f32,
    /// The bot's own identity in the current world.
    pub local:     LocalPlayer,
    /// Other players present in the current world, keyed by net_id.
    pub players:   std::collections::HashMap<u32, Player>,
    /// The bot's inventory, updated on SendInventoryState.
    pub inventory: Inventory,
    /// The current world, updated on SendMapData.
    pub world:     Option<World>,
    /// Active peer, set on Connect and cleared on Disconnect.
    peer_id: Option<enet::PeerID>,
    /// Shared state written by the bot and read by the web layer.
    pub state: Arc<RwLock<BotState>>,
    /// Commands sent from the web layer to be executed each tick.
    cmd_rx:  CmdReceiver,
    /// One-shot callback fired on the next OnDialogRequest.
    pub temporary_data: TemporaryData,
    /// Whether the run loop should auto-collect nearby dropped items.
    pub auto_collect: bool,
    /// Tracks when collect() was last run.
    collect_timer: std::time::Instant,
    /// A* pathfinder, re-used across find_path calls.
    astar: AStar,
    /// Configurable delays for bot actions.
    pub delays: BotDelays,
    /// Item database for collision-type lookups.
    pub items_dat: Arc<ItemsDat>,
    /// Events pushed by packet handlers and drained by Lua's `listenEvents`.
    pub event_queue: std::collections::VecDeque<BotEventRaw>,
    /// Set to true to interrupt a running Lua script.
    pub script_stop: Arc<AtomicBool>,
    /// When set, the bot will delay reconnecting until this instant (used for 2FA cooldown).
    reconnect_after: Option<std::time::Instant>,
    /// Set when an `action|log` with "Advanced Account Protection" is received,
    /// so the subsequent `action|logon_fail` knows to apply the 120 s cooldown.
    pending_2fa: bool,
    /// Set when an `action|log` with "Server requesting that you re-logon" is received,
    /// so the subsequent `action|logon_fail` knows a re-logon was requested.
    pending_relogon: bool,
    /// Set when an `action|log` with "SERVER OVERLOADED" is received,
    /// so the subsequent `action|logon_fail` knows to apply the 30 s cooldown.
    pending_server_overload: bool,
    /// Set when an `action|log` with "Too many people logging in" is received,
    /// so the subsequent `action|logon_fail` knows to apply the 5 s cooldown.
    pending_too_many_logins: bool,
    /// This bot's ID in the BotManager (used to tag WS events).
    pub bot_id: u32,
    /// Broadcast sender for real-time WebSocket events (None when running standalone).
    ws_tx: Option<WsTx>,
    /// Last broadcast ping value — used to suppress redundant BotPing events.
    last_ping: u32,
}

struct Credentials {
    ltoken: String,
    meta:   String,
    addr:   SocketAddr,
}

fn fetch_credentials(username: &str, password: &str, proxy: Option<&Socks5Config>) -> Credentials {
    let proxy_url = proxy.map(|p| p.to_url());
    let proxy_url = proxy_url.as_deref();

    let login_info = LoginInfo {
        protocol:     PROTOCOL,
        game_version: GAME_VER.into(),
    };

    loop {
        println!("[Bot] fetching server_data...");
        let server_data = match get_server_data_proxied(false, &login_info, proxy_url) {
            Ok(s)  => s,
            Err(e) => {
                println!("[Bot] fetch: server_data failed: {e} — retrying in 5s");
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
        };

        let dashboard = match get_dashboard_proxied(&server_data.loginurl, &login_info, &server_data.meta, proxy_url) {
            Ok(d)  => d,
            Err(e) => {
                println!("[Bot] fetch: dashboard failed: {e} — retrying in 5s");
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
        };

        let growtopia_url = match dashboard.growtopia {
            Some(u) => u,
            None    => {
                println!("[Bot] fetch: no Growtopia URL in dashboard — retrying in 5s");
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
        };

        let ltoken = match get_legacy_token_proxied(&growtopia_url, username, password, proxy_url) {
            Ok(t)  => t,
            Err(e) => {
                println!("[Bot] fetch: login failed: {e}");
                if matches!(e, LoginError::Exhausted) {
                    panic!("[Bot] login attempts exhausted — stopping");
                }
                if matches!(e, LoginError::WrongCredentials) {
                    panic!("[Bot] wrong credentials — stopping");
                }
                println!("[Bot] retrying in 5s");
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
        };

        let addr: SocketAddr = format!("{}:{}", server_data.server, server_data.port)
            .parse()
            .expect("Invalid server address");

        println!("[Bot] Got token: {ltoken}");
        return Credentials { ltoken, meta: server_data.meta, addr };
    }
}

impl Bot {
    pub fn new(username: &str, password: &str, proxy: Option<Socks5Config>, state: Arc<RwLock<BotState>>, cmd_rx: CmdReceiver, items_dat: Arc<ItemsDat>, bot_id: u32, ws_tx: Option<WsTx>) -> Self {
        let creds = fetch_credentials(username, password, proxy.as_ref());

        let mac   = random_mac();
        let hash  = hash_string(&format!("{}RT", mac));
        let hash2 = hash_string(&format!("{}RT", random_hex(16)));
        let wk    = random_hex(32);
        let rid   = generate_rid();

        let host = Self::create_host(proxy.as_ref());
        let mut bot = Bot {
            host,
            proxy,
            username: username.to_string(),
            password: password.to_string(),
            ltoken:   creds.ltoken,
            meta:     creds.meta,
            mac,
            hash,
            hash2,
            wk,
            rid,
            redirect:   None,
            peer_id:    None,
            pos_x:      0.0,
            pos_y:      0.0,
            start_time: std::time::Instant::now(),
            local:      LocalPlayer::default(),
            players:    std::collections::HashMap::new(),
            inventory:  Inventory::default(),
            world:      None,
            state,
            cmd_rx,
            temporary_data:  TemporaryData::default(),
            auto_collect:    false,
            collect_timer:   std::time::Instant::now(),
            astar:           AStar::new(),
            delays:          BotDelays::default(),
            items_dat,
            event_queue:     std::collections::VecDeque::new(),
            script_stop:     Arc::new(AtomicBool::new(false)),
            reconnect_after: None,
            pending_2fa:             false,
            pending_relogon:         false,
            pending_server_overload: false,
            pending_too_many_logins: false,
            bot_id,
            ws_tx,
            last_ping:       0,
        };

        bot.host.connect(creds.addr, 2, 0);
        bot
    }

    fn reconnect_main(&mut self) {
        self.refresh_token();

        let login_info = LoginInfo { protocol: PROTOCOL, game_version: GAME_VER.into() };
        let proxy_url  = self.proxy.as_ref().map(|p| p.to_url());
        let server_data = loop {
            match get_server_data_proxied(false, &login_info, proxy_url.as_deref()) {
                Ok(s)  => break s,
                Err(e) => {
                    println!("[Bot] reconnect: server_data failed: {e} — retrying in 5s");
                    std::thread::sleep(std::time::Duration::from_secs(5));
                }
            }
        };
        self.meta = server_data.meta.clone();

        let addr: SocketAddr = format!("{}:{}", server_data.server, server_data.port)
            .parse()
            .expect("Invalid server address");
        self.host.connect(addr, 2, 0);
    }

    fn create_host(proxy: Option<&Socks5Config>) -> BotHost {
        let settings = enet::HostSettings {
            peer_limit:       1,
            channel_limit:    2,
            compressor:       Some(Box::new(enet::RangeCoder::new())),
            checksum:         Some(Box::new(enet::crc32)),
            using_new_packet: true,
            ..Default::default()
        };
        match proxy {
            None => {
                let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket");
                BotHost::Direct(enet::Host::new(socket, settings).expect("Failed to create ENet host"))
            }
            Some(cfg) => {
                let local: SocketAddr = "0.0.0.0:0".parse().unwrap();
                let socket = Socks5UdpSocket::bind_through_proxy(
                    local,
                    cfg.proxy_addr,
                    cfg.username.as_deref(),
                    cfg.password.as_deref(),
                ).expect("Failed to create SOCKS5 UDP socket");
                BotHost::Socks5(enet::Host::new(socket, settings).expect("Failed to create ENet host"))
            }
        }
    }

    fn emit(&self, event: WsEvent) {
        if let Some(tx) = &self.ws_tx {
            let _ = tx.send(event);
        }
    }

    fn build_login_packet(&self) -> String {
        format!("protocol|{PROTOCOL}\nltoken|{}\nplatformID|2\n", self.ltoken)
    }

    fn build_redirect_packet(&self, r: &RedirectData) -> String {
        let klv = compute_klv(GAME_VER, &PROTOCOL.to_string(), &self.rid, self.hash);

        format!(
            "UUIDToken|{}\nprotocol|{PROTOCOL}\nfhash|{FHASH}\nmac|{}\n\
requestedName|\nhash2|{}\nfz|22243512\nf|1\nplayer_age|20\ngame_version|{GAME_VER}\n\
lmode|1\ncbits|1024\nrid|{}\nGDPR|2\nhash|{}\ncategory|_-5100\n\
token|{}\ntotal_playtime|0\ndoor_id|{}\nklv|{klv}\nmeta|{}\n\
platformID|0,1,1\ndeviceVersion|0\nzf|31631978\ncountry|jp\n\
user|{}\nwk|{}\naat|{}\n",
            r.uuid, self.mac, self.hash2, self.rid, self.hash,
            r.token, r.door_id, self.meta,
            r.user, self.wk, r.aat,
        )
    }

    /// Builds the `clientData` string sent to the check-token endpoint.
    /// Uses the bot's stable per-session values (rid, mac, wk, hash, hash2).
    fn build_login_data(&self) -> String {
        let klv = compute_klv(GAME_VER, &PROTOCOL.to_string(), &self.rid, self.hash);
        format!(
            "tankIDName|\ntankIDPass|\nrequestedName|\nf|1\nprotocol|{PROTOCOL}\n\
game_version|{GAME_VER}\nfz|22243512\ncbits|1024\nplayer_age|20\nGDPR|2\nFCMToken|\n\
category|_-5100\ntotalPlaytime|0\nklv|{klv}\nhash2|{}\nmeta|{}\nfhash|{FHASH}\n\
rid|{}\nplatformID|0,1,1\ndeviceVersion|0\ncountry|jp\nhash|{}\nmac|{}\nwk|{}\nzf|31631978\nlmode|1\n",
            self.hash2, self.meta, self.rid, self.hash, self.mac, self.wk,
        )
    }

    /// Refreshes `self.ltoken`: tries check_token first, falls back to full re-login.
    fn refresh_token(&mut self) {
        let login_data = self.build_login_data();
        let proxy = self.proxy.as_ref().map(|p| p.to_url());
        let proxy_url = proxy.as_deref();

        if !self.ltoken.is_empty() {
            if let Ok(new_token) = check_token(&self.ltoken, &login_data, proxy_url) {
                println!("[Bot] Token refreshed via check_token");
                self.ltoken = new_token;
                return;
            }
            println!("[Bot] check_token failed — falling back to full re-login");
        }

        let creds = fetch_credentials(&self.username, &self.password, self.proxy.as_ref());
        self.ltoken = creds.ltoken;
        self.meta   = creds.meta;
    }

    pub fn run(&mut self, stop_flag: Arc<AtomicBool>) {
        loop {
            if stop_flag.load(Ordering::Relaxed) {
                println!("[Bot] Stop flag set, exiting.");
                break;
            }
            // Check if a delayed reconnect (e.g. 2FA cooldown) is ready.
            if let Some(at) = self.reconnect_after {
                if std::time::Instant::now() >= at {
                    self.reconnect_after = None;
                    println!("[Bot] 2FA cooldown elapsed — re-fetching token and server data");
                    self.reconnect_main();
                }
            }
            while let Ok(cmd) = self.cmd_rx.try_recv() {
                self.handle_command(cmd);
            }
            if let Some(id) = self.peer_id {
                let rtt = self.host.peer_rtt(id).as_millis() as u32;
                self.state.write().unwrap().ping_ms = rtt;
                if rtt != self.last_ping {
                    self.last_ping = rtt;
                    self.emit(WsEvent::BotPing { bot_id: self.bot_id, ping_ms: rtt });
                }
            }
            self.service_once();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    /// Process all pending ENet events once.
    pub fn service_once(&mut self) {
        while let Some(event) = self.host.next_event() {
            match event {
                enet::EventNoRef::Connect { peer: id, .. } => {
                    self.peer_id = Some(id);
                    println!("[Bot] Connected: peer {}", id.0);
                }

                enet::EventNoRef::Disconnect { peer: id, .. } => {
                    self.peer_id = None;
                    println!("[Bot] Disconnected: peer {}", id.0);
                    {
                        let mut s = self.state.write().unwrap();
                        s.status     = BotStatus::Connecting;
                        s.world_name = String::new();
                        s.players    = Vec::new();
                        s.ping_ms    = 0;
                    }
                    self.emit(WsEvent::BotStatus { bot_id: self.bot_id, status: "connecting".into() });
                    self.emit(WsEvent::BotWorld  { bot_id: self.bot_id, world_name: String::new() });
                    if let Some(r) = self.redirect.as_ref() {
                        let addr: SocketAddr = format!("{}:{}", r.server, r.port)
                            .parse()
                            .expect("Invalid redirect address");
                        println!("[Bot] Redirecting to {}:{}", r.server, r.port);
                        self.host.connect(addr, 2, 0);
                    } else if self.reconnect_after.is_some() {
                        // Delayed reconnect already scheduled (e.g. 2FA cooldown) — do nothing here.
                    } else {
                        println!("[Bot] Server disconnected — re-fetching token and server data");
                        self.reconnect_main();
                    }
                }

                enet::EventNoRef::Receive { peer: id, channel_id, packet } => {
                    match IncomingPacket::parse(packet.data()) {
                        Some(IncomingPacket::ServerHello) => {
                            self.on_server_hello();
                        }
                        Some(IncomingPacket::Text(s)) => {
                            println!("[Bot] Text: {s}");
                        }
                        Some(IncomingPacket::GameMessage(s)) => {
                            println!("[Bot] GameMessage: {s}");
                            self.event_queue.push_back(BotEventRaw::GameMessage { text: s.to_string() });
                            if s.contains("Advanced Account Protection") {
                                self.pending_2fa = true;
                            }
                            if s.contains("action|log") && s.contains("SERVER OVERLOADED") {
                                self.pending_server_overload = true;
                            }
                            if s.contains("action|log") && s.contains("Too many people logging in") {
                                self.pending_too_many_logins = true;
                            }
                            if s.contains("action|log") && s.contains("Server requesting that you re-logon") {
                                println!("[Bot] Server requested re-logon — clearing redirect data.");
                                self.redirect = None;
                                self.pending_relogon = true;
                            }
                            if s.contains("action|logon_fail") {
                                if self.pending_2fa {
                                    self.pending_2fa = false;
                                    println!("[Bot] Logon failed — 2FA (Advanced Account Protection). Retrying in 120 s.");
                                    self.state.write().unwrap().status = BotStatus::TwoFactorAuth;
                                    self.reconnect_after = Some(std::time::Instant::now() + std::time::Duration::from_secs(120));
                                    self.emit(WsEvent::BotStatus { bot_id: self.bot_id, status: "two_factor_auth".into() });
                                } else if self.pending_server_overload {
                                    self.pending_server_overload = false;
                                    println!("[Bot] Logon failed — server overloaded. Retrying in 30 s.");
                                    self.state.write().unwrap().status = BotStatus::ServerOverloaded;
                                    self.reconnect_after = Some(std::time::Instant::now() + std::time::Duration::from_secs(30));
                                    self.emit(WsEvent::BotStatus { bot_id: self.bot_id, status: "server_overloaded".into() });
                                } else if self.pending_too_many_logins {
                                    self.pending_too_many_logins = false;
                                    println!("[Bot] Logon failed — too many logins at once. Retrying in 5 s.");
                                    self.state.write().unwrap().status = BotStatus::TooManyLogins;
                                    self.reconnect_after = Some(std::time::Instant::now() + std::time::Duration::from_secs(5));
                                    self.emit(WsEvent::BotStatus { bot_id: self.bot_id, status: "too_many_logins".into() });
                                } else if self.pending_relogon {
                                    self.pending_relogon = false;
                                    println!("[Bot] Logon failed — server requested re-logon. Reconnecting.");
                                } else {
                                    println!("[Bot] Logon failed — disconnecting to re-fetch token");
                                }
                                // self.redirect = None;
                                self.host.peer_disconnect(id, 0);
                            }
                        }
                        Some(IncomingPacket::GameUpdate(pkt)) => {
                            self.event_queue.push_back(BotEventRaw::GameUpdate { pkt: pkt.clone() });
                            match pkt.packet_type {
                                GamePacketType::SetCharacterState => {
                                    self.local.hack_type    = pkt.value;
                                    self.local.build_length = pkt.jump_count.saturating_sub(126);
                                    self.local.punch_length = pkt.animation_type.saturating_sub(126);
                                    self.local.gravity      = pkt.vector_x2;
                                    self.local.velocity     = pkt.vector_y2;
                                }
                                GamePacketType::CallFunction => {
                                    let extra = pkt.extra_data.clone();
                                    let net_id = id.0 as u32;
                                    if let Ok(vl) = VariantList::deserialize(&extra) {
                                        self.event_queue.push_back(BotEventRaw::VariantList { vl, net_id });
                                    }
                                    self.on_call_function(id, &extra);
                                }
                                GamePacketType::PingRequest => {
                                    self.on_ping_request(pkt.value);
                                }
                                GamePacketType::SendInventoryState => {
                                    match Inventory::parse(&pkt.extra_data) {
                                        Ok(inv) => {
                                            println!("[Bot] Inventory: {} items", inv.item_count);
                                            self.inventory = inv.clone();
                                            let slots: Vec<InvSlot> = inv.items.values()
                                                .map(|i| InvSlot {
                                                    item_id:     i.id,
                                                    amount:      i.amount,
                                                    is_active:   i.flag & 1 != 0,
                                                    action_type: self.items_dat.find_by_id(i.id as u32).map(|info| info.action_type).unwrap_or(0),
                                                })
                                                .collect();
                                            self.state.write().unwrap().inventory = slots;
                                            let ws_items: Vec<WsInvItem> = inv.items.values()
                                                .map(|i| WsInvItem {
                                                    item_id:     i.id,
                                                    amount:      i.amount,
                                                    is_active:   i.flag & 1 != 0,
                                                    action_type: self.items_dat.find_by_id(i.id as u32).map(|info| info.action_type).unwrap_or(0),
                                                })
                                                .collect();
                                            self.emit(WsEvent::InventoryUpdate { bot_id: self.bot_id, gems: inv.gems, items: ws_items });
                                        }
                                        Err(e) => println!("[Bot] Inventory parse error: {e}"),
                                    }
                                }
                                GamePacketType::SendMapData => {
                                    self.players.clear();
                                    self.local = LocalPlayer::default();
                                    match World::parse(&pkt.extra_data) {
                                        Ok(world) => {
                                            println!(
                                                "[Bot] World: {}x{} tiles, {} objects",
                                                world.tile_map.width,
                                                world.tile_map.height,
                                                world.objects.len(),
                                            );
                                            self.world = Some(world.clone());
                                            let tiles: Vec<TileInfo> = world.tile_map.tiles.iter()
                                                .map(|t| TileInfo { fg_item_id: t.fg_item_id, bg_item_id: t.bg_item_id })
                                                .collect();
                                            let mut s = self.state.write().unwrap();
                                            s.world_name   = world.tile_map.world_name.clone();
                                            s.world_width  = world.tile_map.width;
                                            s.world_height = world.tile_map.height;
                                            s.tiles        = tiles;
                                            s.players      = Vec::new();
                                            s.status       = BotStatus::InWorld;
                                            // Emit world-loaded event with full tile data.
                                            let ws_tiles: Vec<WsTile> = world.tile_map.tiles.iter()
                                                .map(|t| WsTile {
                                                    fg:        t.fg_item_id,
                                                    bg:        t.bg_item_id,
                                                    flags:     t.flags_raw,
                                                    tile_type: t.tile_type.clone(),
                                                })
                                                .collect();
                                            drop(s);
                                            self.emit(WsEvent::BotStatus  { bot_id: self.bot_id, status: "in_world".into() });
                                            self.emit(WsEvent::BotWorld   { bot_id: self.bot_id, world_name: world.tile_map.world_name.clone() });
                                            self.emit(WsEvent::WorldLoaded {
                                                bot_id: self.bot_id,
                                                name:   world.tile_map.world_name.clone(),
                                                width:  world.tile_map.width,
                                                height: world.tile_map.height,
                                                tiles:  ws_tiles,
                                            });
                                        }
                                        Err(e) => println!("[Bot] World parse error: {e}"),
                                    }
                                }
                                GamePacketType::State => self.on_state(&pkt),
                                GamePacketType::TileChangeRequest => self.on_tile_change(&pkt),
                                GamePacketType::SendTileUpdateData => self.on_send_tile_update_data(&pkt),
                                GamePacketType::SendTileUpdateDataMultiple => self.on_send_tile_update_data_multiple(&pkt),
                                GamePacketType::SendTileTreeState => self.on_send_tile_tree_state(&pkt),
                                GamePacketType::ModifyItemInventory => self.on_modify_item_inventory(&pkt),
                                GamePacketType::ItemChangeObject => self.on_item_change_object(&pkt),
                                GamePacketType::SendLock => self.on_send_lock(&pkt),
                                _ => println!("[Bot] {pkt}"),
                            }
                        }
                        Some(IncomingPacket::Track(s)) => {
                            println!("[Bot] Track: {s}");
                            let fields: std::collections::HashMap<&str, &str> = s.lines()
                                .filter_map(|line| line.split_once('|'))
                                .collect();
                            let level          = fields.get("Level")          .and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
                            let grow_id        = fields.get("GrowId")         .and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
                            let install_date   = fields.get("installDate")    .and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
                            let global_playtime= fields.get("Global_Playtime").and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
                            let awesomeness    = fields.get("Awesomeness")    .and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
                            self.emit(WsEvent::BotTrackInfo { bot_id: self.bot_id, level, grow_id, install_date, global_playtime, awesomeness });
                        }
                        Some(IncomingPacket::ClientLogRequest) => {
                            println!("[Bot] ClientLogRequest");
                        }
                        Some(IncomingPacket::Unknown { msg_type, data }) => {
                            println!("[Bot] Unknown msg_type={msg_type} len={}", data.len());
                        }
                        None => {
                            let hex = packet.data().iter()
                                .map(|b| format!("{:02x}", b))
                                .collect::<Vec<_>>()
                                .join(" ");
                            println!(
                                "[Bot] Failed to parse packet ({} bytes on ch {}): {}",
                                packet.data().len(), channel_id, hex
                            );
                        }
                    }
                }
            }
        }
        if self.auto_collect
            && self.collect_timer.elapsed() >= std::time::Duration::from_millis(500)
        {
            self.collect_timer = std::time::Instant::now();
            self.collect();
        }
    }

    pub fn send_text(&mut self, text: &str) {
        if let Some(id) = self.peer_id {
            let raw = packet::make_text_packet(text);
            self.host.peer_send(id, 0, &enet::Packet::reliable(raw));
        }
    }

    pub fn send_game_message(&mut self, text: &str) {
        if let Some(id) = self.peer_id {
            let raw = packet::make_game_message_packet(text);
            self.host.peer_send(id, 0, &enet::Packet::reliable(raw));
        }
    }

    pub fn send_game_packet(&mut self, pkt: &GameUpdatePacket, reliable: bool) {
        if let Some(id) = self.peer_id {
            let raw = packet::make_game_packet(pkt);
            let enet_pkt = if reliable {
                enet::Packet::reliable(raw)
            } else {
                enet::Packet::unreliable(raw)
            };
            self.host.peer_send(id, 0, &enet_pkt);
        }
    }

    fn on_server_hello(&mut self) {
        let data = match self.redirect.take() {
            Some(r) => {
                println!("[Bot] ServerHello (redirect → {})", r.door_id);
                self.build_redirect_packet(&r)
            }
            None => {
                println!("[Bot] ServerHello");
                self.build_login_packet()
            }
        };
        self.send_text(&data);
    }

    fn on_ping_request(&mut self, challenge: u32) {
        let time_val = self.start_time.elapsed().as_millis() as u32;

        let bx = if self.local.build_length == 0 { 2.0 } else { self.local.build_length as f32 };
        let by = if self.local.punch_length == 0 { 2.0 } else { self.local.punch_length as f32 };

        let in_world = self.world.is_some();

        let mut reply = GameUpdatePacket {
            packet_type:   GamePacketType::PingReply,
            target_net_id: hash_string(&challenge.to_string()),
            value:         time_val,
            vector_x:      bx * 32.0,
            vector_y:      by * 32.0,
            ..Default::default()
        };

        if in_world {
            reply.net_id    = self.local.hack_type;
            reply.vector_x2 = self.local.velocity;
            reply.vector_y2 = self.local.gravity;
        }

        self.send_game_packet(&reply, true);
        println!("[Bot] PingReply sent (challenge={})", challenge);
    }

    fn on_call_function(&mut self, id: enet::PeerID, extra_data: &[u8]) {
        let vl = match VariantList::deserialize(extra_data) {
            Ok(v)  => v,
            Err(e) => { println!("[Bot] VariantList parse error: {e}"); return; }
        };

        let fn_name = vl.get(0).map(|v| v.as_string()).unwrap_or_default();
        println!("[Bot] CallFunction: {fn_name}");

        match fn_name.as_str() {
            "OnSendToServer" => {
                let port       = vl.get(1).map(|v| v.as_int32()).unwrap_or(0);
                let token      = vl.get(2).map(|v| v.as_int32()).unwrap_or(0);
                let user_id    = vl.get(3).map(|v| v.as_int32()).unwrap_or(0);
                let server_str = vl.get(4).map(|v| v.as_string()).unwrap_or_default();
                let aat        = vl.get(5).map(|v| v.as_int32()).unwrap_or(0);

                let parts: Vec<&str> = server_str.splitn(3, '|').collect();
                let server  = parts.first().copied().unwrap_or("").trim_end().to_string();
                let door_id = parts.get(1).copied().map(str::trim_end).filter(|s| !s.is_empty()).unwrap_or("0").to_string();
                let uuid    = parts.get(2).copied().unwrap_or("").trim_end().to_string();

                println!("[Bot] OnSendToServer → {server}:{port} door={door_id}");

                self.redirect = Some(RedirectData {
                    server,
                    port:    port as u16,
                    token:   token.to_string(),
                    user:    user_id.to_string(),
                    door_id,
                    uuid,
                    aat:     aat.to_string(),
                });

                self.host.peer_disconnect(id, 0);
            }
            "OnSpawn" => {
                let message = vl.get(1).map(|v| v.as_string()).unwrap_or_default();
                let data = parse_pipe_map(&message);

                if data.contains_key("type") {
                    // Local player spawning — store our own identity
                    self.local.net_id  = data.get("netID").and_then(|s| s.parse().ok()).unwrap_or(0);
                    self.local.user_id = data.get("userID").and_then(|s| s.parse().ok()).unwrap_or(0);
                    println!("[Bot] OnSpawn (self) net_id={} user_id={}", self.local.net_id, self.local.user_id);
                    {
                        let mut s = self.state.write().unwrap();
                        s.status = BotStatus::InWorld;
                    }
                    self.emit(WsEvent::BotStatus { bot_id: self.bot_id, status: "in_world".into() });
                } else {
                    let position = if let Some(pos_xy) = data.get("posXY") {
                        let parts: Vec<f32> = pos_xy.split('|')
                            .filter_map(|s| s.trim().parse().ok())
                            .collect();
                        (*parts.first().unwrap_or(&0.0), *parts.get(1).unwrap_or(&0.0))
                    } else {
                        (0.0, 0.0)
                    };

                    let net_id  = data.get("netID").and_then(|s| s.parse().ok()).unwrap_or(0);
                    let user_id = data.get("userID").and_then(|s| s.parse().ok()).unwrap_or(0);
                    let m_state = data.get("mstate").and_then(|s| s.parse().ok()).unwrap_or(0u32);
                    let invisible = data.get("invis")
                        .and_then(|s| s.parse::<u32>().ok())
                        .unwrap_or(0) != 0;
                    let name    = data.get("name").cloned().unwrap_or_default();
                    let country = data.get("country").cloned().unwrap_or_default();

                    println!("[Bot] OnSpawn player={name} net_id={net_id} pos=({:.0},{:.0})", position.0, position.1);

                    let player = Player {
                        net_id,
                        user_id,
                        name,
                        country,
                        position,
                        avatar:     data.get("avatar").cloned().unwrap_or_default(),
                        online_id:  data.get("onlineID").cloned().unwrap_or_default(),
                        e_id:       data.get("eid").cloned().unwrap_or_default(),
                        ip:         data.get("ip").cloned().unwrap_or_default(),
                        col_rect:   data.get("colrect").cloned().unwrap_or_default(),
                        title_icon: data.get("titleIcon").cloned().unwrap_or_default(),
                        m_state,
                        invisible,
                    };

                    self.players.insert(net_id, player.clone());
                    {
                        let mut s = self.state.write().unwrap();
                        s.players = self.players.values()
                            .map(|p| PlayerInfo {
                                net_id:  p.net_id,
                                name:    p.name.clone(),
                                pos_x:   p.position.0 / 32.0,
                                pos_y:   p.position.1 / 32.0,
                                country: p.country.clone(),
                            })
                            .collect();
                    }
                    self.emit(WsEvent::PlayerSpawn {
                        bot_id:  self.bot_id,
                        net_id:  player.net_id,
                        name:    player.name.clone(),
                        country: player.country.clone(),
                        x:       player.position.0 / 32.0,
                        y:       player.position.1 / 32.0,
                    });
                }
            }
            "OnSetPos" => {
                let (x, y) = vl.get(1).map(|v| v.as_vec2()).unwrap_or((0.0, 0.0));
                self.pos_x = x;
                self.pos_y = y;
                {
                    let mut s = self.state.write().unwrap();
                    s.pos_x = x / 32.0;
                    s.pos_y = y / 32.0;
                }
                println!("[Bot] OnSetPos → ({x}, {y})");
                self.emit(WsEvent::BotMove { bot_id: self.bot_id, x: x / 32.0, y: y / 32.0 });
            }
            "OnSuperMainStartAcceptLogonHrdxs47254722215a" => {
                self.state.write().unwrap().status = BotStatus::Connected;
                self.send_text("action|enter_game\n");
                self.emit(WsEvent::BotStatus { bot_id: self.bot_id, status: "connected".into() });
            }
            "OnRemove" => {
                let message = vl.get(1).map(|v| v.as_string()).unwrap_or_default();
                let data = parse_pipe_map(&message);
                let net_id: u32 = data.get("netID").and_then(|s| s.parse().ok()).unwrap_or(0);
                self.players.remove(&net_id);
                self.state.write().unwrap().players = self.players.values()
                    .map(|p| PlayerInfo {
                        net_id:  p.net_id,
                        name:    p.name.clone(),
                        pos_x:   p.position.0 / 32.0,
                        pos_y:   p.position.1 / 32.0,
                        country: p.country.clone(),
                    })
                    .collect();
                println!("[Bot] OnRemove net_id={net_id}");
                self.emit(WsEvent::PlayerLeave { bot_id: self.bot_id, net_id });
            }
            "OnSetBux" => {
                let gems = vl.get(1).map(|v| v.as_int32()).unwrap_or(0);
                self.inventory.add_gems(gems);
                self.state.write().unwrap().gems = gems;
                self.emit(WsEvent::BotGems { bot_id: self.bot_id, gems });
            }
            "OnConsoleMessage" => {
                let message = vl.get(1).map(|v| v.as_string()).unwrap_or_default();
                println!("[Bot] Console: {message}");
                {
                    let mut s = self.state.write().unwrap();
                    s.console.push(message.clone());
                    if s.console.len() > 100 {
                        s.console.remove(0);
                    }
                }
                self.emit(WsEvent::Console { bot_id: self.bot_id, message });
            }
            "OnDialogRequest" => {
                let message = vl.get(1).map(|v| v.as_string()).unwrap_or_default();
                println!("[Bot] Dialog: {}", &message[..message.len().min(80)]);
                let cb = self.temporary_data.dialog_callback.lock().unwrap().take();
                if let Some(cb) = cb {
                    cb(self);
                }
            }
            "OnRequestWorldSelectMenu" => {
                self.world = None;
                {
                    let mut s = self.state.write().unwrap();
                    s.world_name = "EXIT".to_string();
                    s.status = BotStatus::Connected;
                }
                self.emit(WsEvent::BotStatus  { bot_id: self.bot_id, status: "connected".into() });
                self.emit(WsEvent::BotWorld   { bot_id: self.bot_id, world_name: "EXIT".to_string() });
                self.emit(WsEvent::WorldLoaded { bot_id: self.bot_id, name: "EXIT".to_string(), width: 0, height: 0, tiles: vec![] });
                println!("[Bot] OnRequestWorldSelectMenu → cleared world");
            }
            _ => {}
        }
    }

    fn on_state(&mut self, pkt: &GameUpdatePacket) {
        if pkt.net_id == self.local.net_id {
            self.pos_x = pkt.vector_x;
            self.pos_y = pkt.vector_y;
            {
                let mut s = self.state.write().unwrap();
                s.pos_x = pkt.vector_x / 32.0;
                s.pos_y = pkt.vector_y / 32.0;
            }
            self.emit(WsEvent::BotMove { bot_id: self.bot_id, x: pkt.vector_x / 32.0, y: pkt.vector_y / 32.0 });
        } else if let Some(player) = self.players.get_mut(&pkt.net_id) {
            player.position = (pkt.vector_x, pkt.vector_y);
            let net_id = pkt.net_id;
            {
                let mut s = self.state.write().unwrap();
                if let Some(pi) = s.players.iter_mut().find(|p| p.net_id == net_id) {
                    pi.pos_x = pkt.vector_x / 32.0;
                    pi.pos_y = pkt.vector_y / 32.0;
                }
            }
            self.emit(WsEvent::PlayerMove { bot_id: self.bot_id, net_id, x: pkt.vector_x / 32.0, y: pkt.vector_y / 32.0 });
        }
    }

    fn on_tile_change(&mut self, pkt: &GameUpdatePacket) {
        let x = pkt.int_x as u32;
        let y = pkt.int_y as u32;
        let item_id = pkt.value as u16;

        let width = match self.world.as_ref() {
            Some(w) => w.tile_map.width,
            None => return,
        };
        let idx = (y * width + x) as usize;

        let result = {
            let world = self.world.as_mut().unwrap();
            if let Some(tile) = world.get_tile_mut(x, y) {
                if item_id == 18 {
                    if tile.fg_item_id != 0 {
                        tile.fg_item_id = 0;
                        tile.tile_type = TileType::Basic;
                    } else {
                        tile.bg_item_id = 0;
                    }
                } else {
                    tile.fg_item_id = item_id;
                }
                Some((tile.fg_item_id, tile.bg_item_id))
            } else {
                None
            }
        };

        if let Some((fg, bg)) = result {
            {
                let mut s = self.state.write().unwrap();
                if let Some(ti) = s.tiles.get_mut(idx) {
                    ti.fg_item_id = fg;
                    ti.bg_item_id = bg;
                }
            }
            self.emit(WsEvent::TileUpdate { bot_id: self.bot_id, x, y, fg, bg });
        }
        println!("[Bot] TileChange ({x},{y}) item={item_id}");
    }

    fn on_send_tile_update_data(&mut self, pkt: &GameUpdatePacket) {
        let x = pkt.int_x as u32;
        let y = pkt.int_y as u32;

        let width = match self.world.as_ref() {
            Some(w) => w.tile_map.width,
            None => return,
        };
        let idx = (y * width + x) as usize;

        let result = self.world.as_mut().unwrap()
            .update_tile_from_bytes(x, y, &pkt.extra_data);

        if let Some((fg, bg)) = result {
            {
                let mut s = self.state.write().unwrap();
                if let Some(ti) = s.tiles.get_mut(idx) {
                    ti.fg_item_id = fg;
                    ti.bg_item_id = bg;
                }
            }
            self.emit(WsEvent::TileUpdate { bot_id: self.bot_id, x, y, fg, bg });
        }
        println!("[Bot] TileUpdateData ({x},{y})");
    }

    fn on_send_tile_update_data_multiple(&mut self, pkt: &GameUpdatePacket) {
        // extra_data: u32 count, then for each: i32 x, i32 y, u16 fg, u16 bg, ...
        let data = &pkt.extra_data;
        if data.len() < 4 { return; }

        let count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let mut offset = 4;

        let width = match self.world.as_ref() {
            Some(w) => w.tile_map.width,
            None => return,
        };

        for _ in 0..count {
            // Each entry: i32 x (4), i32 y (4), u16 fg (2), u16 bg (2) = 12 bytes minimum
            if offset + 12 > data.len() { break; }

            let x = u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]);
            let y = u32::from_le_bytes([data[offset+4], data[offset+5], data[offset+6], data[offset+7]]);
            let tile_data = &data[offset + 8..];
            let idx = (y as u64 * width as u64 + x as u64) as usize;

            let result = self.world.as_mut().unwrap()
                .update_tile_from_bytes(x, y, tile_data);

            if let Some((fg, bg)) = result {
                {
                    let mut s = self.state.write().unwrap();
                    if let Some(ti) = s.tiles.get_mut(idx) {
                        ti.fg_item_id = fg;
                        ti.bg_item_id = bg;
                    }
                }
                self.emit(WsEvent::TileUpdate { bot_id: self.bot_id, x, y, fg, bg });
            }

            offset += 12; // advance past the known fields; extra tile data is not parsed
        }
        println!("[Bot] TileUpdateDataMultiple count={count}");
    }

    fn on_send_tile_tree_state(&mut self, pkt: &GameUpdatePacket) {
        let x = pkt.int_x as u32;
        let y = pkt.int_y as u32;

        let width = match self.world.as_ref() {
            Some(w) => w.tile_map.width,
            None => return,
        };
        let idx = (y * width + x) as usize;

        let world = self.world.as_mut().unwrap();
        if let Some(tile) = world.get_tile_mut(x, y) {
            tile.fg_item_id = 0;
            tile.tile_type = TileType::Basic;
            let bg = tile.bg_item_id;
            {
                let mut s = self.state.write().unwrap();
                if let Some(ti) = s.tiles.get_mut(idx) {
                    ti.fg_item_id = 0;
                }
            }
            self.emit(WsEvent::TileUpdate { bot_id: self.bot_id, x, y, fg: 0, bg });
        }
        println!("[Bot] TileTreeState ({x},{y}) harvested");
    }

    fn on_modify_item_inventory(&mut self, pkt: &GameUpdatePacket) {
        let item_id = pkt.value as u16;
        let amount  = pkt.jump_count;
        self.inventory.remove_item(item_id, amount);
        println!("[Bot] ModifyItemInventory item={item_id} -{amount}");
        let slots: Vec<InvSlot> = self.inventory.items.values()
            .map(|i| InvSlot {
                item_id:     i.id,
                amount:      i.amount,
                is_active:   i.flag & 1 != 0,
                action_type: self.items_dat.find_by_id(i.id as u32).map(|info| info.action_type).unwrap_or(0),
            })
            .collect();
        self.state.write().unwrap().inventory = slots;
        let ws_items: Vec<WsInvItem> = self.inventory.items.values()
            .map(|i| WsInvItem {
                item_id:     i.id,
                amount:      i.amount,
                is_active:   i.flag & 1 != 0,
                action_type: self.items_dat.find_by_id(i.id as u32).map(|info| info.action_type).unwrap_or(0),
            })
            .collect();
        self.emit(WsEvent::InventoryUpdate { bot_id: self.bot_id, gems: self.inventory.gems, items: ws_items });
    }

    fn on_item_change_object(&mut self, pkt: &GameUpdatePacket) {
        if self.world.is_none() { return; }

        match pkt.net_id {
            u32::MAX => {
                // New item dropped into the world
                let world = self.world.as_mut().unwrap();
                let next_uid = world.objects.last().map(|o| o.uid + 1).unwrap_or(1);
                let obj = WorldObject {
                    item_id: pkt.value as u16,
                    x: pkt.vector_x.ceil(),
                    y: pkt.vector_y.ceil(),
                    count: pkt.float_variable as u8,
                    flags: pkt.object_type,
                    uid: next_uid,
                };
                println!("[Bot] ItemDrop id={} uid={} pos=({:.0},{:.0})", obj.item_id, obj.uid, obj.x, obj.y);
                world.objects.push(obj);
                let ws_objs: Vec<WsObject> = world.objects.iter()
                    .map(|o| WsObject { uid: o.uid, item_id: o.item_id, x: o.x, y: o.y, count: o.count })
                    .collect();
                self.emit(WsEvent::ObjectsUpdate { bot_id: self.bot_id, objects: ws_objs });
            }
            net_id if net_id == u32::MAX - 3 => {
                // Update count for an existing dropped item
                let world = self.world.as_mut().unwrap();
                if let Some(obj) = world.objects.iter_mut().find(|o| {
                    o.item_id == pkt.value as u16
                        && o.x == pkt.vector_x.ceil()
                        && o.y == pkt.vector_y.ceil()
                }) {
                    obj.count = pkt.jump_count;
                }
                let ws_objs: Vec<WsObject> = world.objects.iter()
                    .map(|o| WsObject { uid: o.uid, item_id: o.item_id, x: o.x, y: o.y, count: o.count })
                    .collect();
                self.emit(WsEvent::ObjectsUpdate { bot_id: self.bot_id, objects: ws_objs });
            }
            net_id if net_id > 0 => {
                // Item collected — remove from world by uid; release borrow before updating inventory
                let collected = {
                    let world = self.world.as_mut().unwrap();
                    world.objects.iter().position(|o| o.uid == pkt.value)
                        .map(|idx| world.objects.remove(idx))
                };
                if let Some(item) = collected {
                    let ws_objs: Vec<WsObject> = self.world.as_ref().unwrap().objects.iter()
                        .map(|o| WsObject { uid: o.uid, item_id: o.item_id, x: o.x, y: o.y, count: o.count })
                        .collect();
                    self.emit(WsEvent::ObjectsUpdate { bot_id: self.bot_id, objects: ws_objs });
                    if pkt.net_id == self.local.net_id {
                        self.inventory.add_item(item.item_id, item.count);
                        println!("[Bot] ItemCollect id={} count={}", item.item_id, item.count);
                        let slots: Vec<InvSlot> = self.inventory.items.values()
                            .map(|i| InvSlot {
                                item_id:     i.id,
                                amount:      i.amount,
                                is_active:   i.flag & 1 != 0,
                                action_type: self.items_dat.find_by_id(i.id as u32).map(|info| info.action_type).unwrap_or(0),
                            })
                            .collect();
                        self.state.write().unwrap().inventory = slots;
                        let ws_items: Vec<WsInvItem> = self.inventory.items.values()
                            .map(|i| WsInvItem {
                                item_id:     i.id,
                                amount:      i.amount,
                                is_active:   i.flag & 1 != 0,
                                action_type: self.items_dat.find_by_id(i.id as u32).map(|info| info.action_type).unwrap_or(0),
                            })
                            .collect();
                        self.emit(WsEvent::InventoryUpdate { bot_id: self.bot_id, gems: self.inventory.gems, items: ws_items });
                    }
                }
            }
            _ => {}
        }
    }

    fn on_send_lock(&mut self, pkt: &GameUpdatePacket) {
        let x   = pkt.int_x as u32;
        let y   = pkt.int_y as u32;
        let fg  = pkt.value as u16;

        let world = match self.world.as_mut() {
            Some(w) => w,
            None => return,
        };
        let width = world.tile_map.width;

        let bg = match world.get_tile_mut(x, y) {
            Some(t) => {
                t.fg_item_id = fg;
                t.bg_item_id
            }
            None => return,
        };

        {
            let mut s = self.state.write().unwrap();
            let idx = (y * width + x) as usize;
            if let Some(ti) = s.tiles.get_mut(idx) {
                ti.fg_item_id = fg;
            }
        }

        self.emit(WsEvent::TileUpdate { bot_id: self.bot_id, x, y, fg, bg });
        println!("[Bot] SendLock tile=({x},{y}) item={fg}");
    }

    pub fn walk(&mut self, tile_x: i32, tile_y: i32) {
        let target_x = tile_x as f32 * 32.0;
        let target_y = tile_y as f32 * 32.0;

        let facing_left = target_x < self.pos_x;
        self.pos_x = target_x;
        self.pos_y = target_y;

        {
            let mut s = self.state.write().unwrap();
            s.pos_x = target_x / 32.0;
            s.pos_y = target_y / 32.0;
        }
        self.emit(WsEvent::BotMove { bot_id: self.bot_id, x: target_x / 32.0, y: target_y / 32.0 });

        let mut flags = packet::PacketFlags::WALK | packet::PacketFlags::STANDING;
        flags.set(packet::PacketFlags::FACING_LEFT, facing_left);

        let pkt = GameUpdatePacket {
            packet_type: GamePacketType::State,
            vector_x: target_x,
            vector_y: target_y + 2.0,
            int_x: -1,
            int_y: -1,
            flags,
            ..Default::default()
        };

        self.send_game_packet(&pkt, false);
        std::thread::sleep(std::time::Duration::from_millis(self.delays.walk_ms));
    }

    pub fn place(&mut self, offset_x: i32, offset_y: i32, item_id: u32, is_punch: bool) {
        if !is_punch && !self.inventory.has_item(item_id as u16, 1) {
            return;
        }

        let base_x = (self.pos_x / 32.0).floor() as i32;
        let base_y = (self.pos_y / 32.0).floor() as i32;
        let tile_x = base_x + offset_x;
        let tile_y = base_y + offset_y;

        if tile_x > base_x + 4 || tile_x < base_x - 4
            || tile_y > base_y + 4 || tile_y < base_y - 4
        {
            return;
        }

        let mut pkt = GameUpdatePacket {
            packet_type: GamePacketType::TileChangeRequest,
            vector_x:    self.pos_x,
            vector_y:    self.pos_y,
            int_x:       tile_x,
            int_y:       tile_y,
            value:       item_id,
            ..Default::default()
        };
        self.send_game_packet(&pkt, true);

        let mut flags = if is_punch {
            packet::PacketFlags::PUNCH
        } else {
            packet::PacketFlags::PLACE
        } | packet::PacketFlags::STANDING;
        flags.set(packet::PacketFlags::FACING_LEFT, base_x > tile_x);

        pkt.packet_type = GamePacketType::State;
        pkt.flags       = flags;
        self.send_game_packet(&pkt, true);
        std::thread::sleep(std::time::Duration::from_millis(self.delays.place_ms));
    }

    pub fn punch(&mut self, offset_x: i32, offset_y: i32) {
        self.place(offset_x, offset_y, 18, true);
    }

    pub fn wrench(&mut self, offset_x: i32, offset_y: i32) {
        self.place(offset_x, offset_y, 32, false);
    }

    pub fn wear(&mut self, item_id: u32) {
        let pkt = GameUpdatePacket {
            packet_type: GamePacketType::ItemActivateObjectRequest,
            value:       item_id,
            ..Default::default()
        };
        self.send_game_packet(&pkt, true);
    }

    pub fn wrench_player(&mut self, net_id: u32) {
        self.send_text(&format!("action|wrench\n|netid|{net_id}\n"));
    }

    pub fn drop_item(&mut self, item_id: u32, amount: u32) {
        self.send_text(&format!("action|drop\n|itemID|{item_id}\n"));
        *self.temporary_data.dialog_callback.lock().unwrap() = Some(Box::new(move |bot: &mut Bot| {
            bot.send_text(&format!(
                "action|dialog_return\ndialog_name|drop_item\nitemID|{item_id}|\ncount|{amount}\n"
            ));
            *bot.temporary_data.dialog_callback.lock().unwrap() = None;
        }));
    }

    pub fn trash_item(&mut self, item_id: u32, amount: u32) {
        self.send_text(&format!("action|trash\n|itemID|{item_id}\n"));
        *self.temporary_data.dialog_callback.lock().unwrap() = Some(Box::new(move |bot: &mut Bot| {
            bot.send_text(&format!(
                "action|dialog_return\ndialog_name|trash_item\nitemID|{item_id}|\ncount|{amount}\n"
            ));
            *bot.temporary_data.dialog_callback.lock().unwrap() = None;
        }));
    }

    pub fn accept_access(&mut self) {
        let net_id = self.local.net_id;
        self.wrench_player(net_id);
        *self.temporary_data.dialog_callback.lock().unwrap() = Some(Box::new(move |bot: &mut Bot| {
            bot.send_text(&format!(
                "action|dialog_return\ndialog_name|popup\nnetID|{net_id}|\nbuttonClicked|acceptlock\n"
            ));
            *bot.temporary_data.dialog_callback.lock().unwrap() = Some(Box::new(|bot: &mut Bot| {
                bot.send_text("action|dialog_return\ndialog_name|acceptaccess\n");
                *bot.temporary_data.dialog_callback.lock().unwrap() = None;
            }));
        }));
    }

    pub fn set_auto_collect(&mut self, enabled: bool) {
        self.auto_collect = enabled;
    }

    pub fn collect(&mut self) -> usize {
        // Must be in a world
        if self.world.is_none() {
            return 0;
        }

        // Skip if inventory is already full
        let inv_size  = self.inventory.size;
        let inv_count = self.inventory.item_count as u32;
        if inv_count >= inv_size {
            return 0;
        }

        let pos_x = self.pos_x;
        let pos_y = self.pos_y;

        const RADIUS_SQ: f32 = 96.0 * 96.0; // 3-tile collect radius
        const MAX_PER_TICK: usize = 32;      // cap packets per call

        // Snapshot nearby items (releases world borrow before we touch inventory/send)
        let nearby: Vec<(u32, f32, f32, u16)> = {
            let objects = &self.world.as_ref().unwrap().objects;
            let mut v: Vec<(f32, u32, f32, f32, u16)> = objects
                .iter()
                .filter_map(|obj| {
                    let dx = pos_x - obj.x;
                    let dy = pos_y - obj.y;
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq <= RADIUS_SQ {
                        Some((dist_sq, obj.uid, obj.x, obj.y, obj.item_id))
                    } else {
                        None
                    }
                })
                .collect();
            v.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            v.into_iter().map(|(_, uid, x, y, id)| (uid, x, y, id)).collect()
        };

        if nearby.is_empty() {
            return 0;
        }

        let mut sent = 0;
        for (uid, x, y, item_id) in nearby.iter().take(MAX_PER_TICK) {
            let can_collect = if *item_id == 112 {
                // Gems always have room
                true
            } else if let Some(existing) = self.inventory.items.get(item_id) {
                existing.amount < 200
            } else {
                inv_count < inv_size
            };

            if can_collect {
                let pkt = GameUpdatePacket {
                    packet_type: GamePacketType::ItemActivateObjectRequest,
                    vector_x:    *x,
                    vector_y:    *y,
                    value:       *uid,
                    ..Default::default()
                };
                self.send_game_packet(&pkt, true);
                sent += 1;
            }
        }

        sent
    }

    pub fn has_access(&self) -> bool {
        const LOCK_ITEM_IDS: &[u16] = &[242, 1796, 2408, 7188, 10410];

        let world = match &self.world {
            Some(w) => w,
            None => return false,
        };

        let bot_uid = self.local.user_id;

        for tile in &world.tile_map.tiles {
            if LOCK_ITEM_IDS.contains(&tile.fg_item_id) {
                if let TileType::Lock { access_uids, .. } = &tile.tile_type {
                    if access_uids.contains(&bot_uid) {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn find_path(&mut self, to_x: u32, to_y: u32) {
        let world = match &self.world {
            Some(w) => w,
            None => return,
        };

        let width  = world.tile_map.width;
        let height = world.tile_map.height;

        // Build (fg_item_id, collision_type) pairs for the grid
        let tiles: Vec<(u16, u8)> = world.tile_map.tiles.iter().map(|t| {
            let ct = match &t.tile_type {
                TileType::Door { .. } => 0, // doors are always passable
                _ => self.items_dat
                    .find_by_id(t.fg_item_id as u32)
                    .map(|i| i.collision_type)
                    .unwrap_or(if t.fg_item_id == 0 { 0 } else { 1 }),
            };
            (t.fg_item_id, ct)
        }).collect();

        self.astar.update_from_tiles(width, height, &tiles);

        let from_x = (self.pos_x / 32.0) as u32;
        let from_y = (self.pos_y / 32.0) as u32;
        let has_access = self.has_access();

        let path = self.astar.find_path(from_x, from_y, to_x, to_y, has_access);

        if let Some(nodes) = path {
            for node in nodes {
                self.walk(node.x as i32, node.y as i32);
            }
        }
    }

    fn handle_command(&mut self, cmd: BotCommand) {
        match cmd {
            BotCommand::Move { x, y } => {
                println!("[Bot] Command: Move to ({x}, {y})");
            }
            BotCommand::WalkTo { x, y } => {
                self.find_path(x, y);
            }
            BotCommand::MoveDelta { dx, dy } => {
                let cx = (self.pos_x / 32.0) as i32;
                let cy = (self.pos_y / 32.0) as i32;
                self.walk(cx + dx, cy + dy);
            }
            BotCommand::RunScript { content } => {
                crate::lua_api::run_script(self, &content);
            }
            BotCommand::StopScript => { self.script_stop.store(true, Ordering::Relaxed); }
            BotCommand::Say { text } => { self.say(&text); }
            BotCommand::Warp { name, id } => { self.warp(&name, &id); }
            BotCommand::Disconnect => { self.disconnect(); }
            BotCommand::Place { x, y, item } => { self.place_at(x, y, item); }
            BotCommand::Hit { x, y } => { self.hit_at(x, y); }
            BotCommand::Wrench { x, y } => { self.wrench_at(x, y); }
            BotCommand::Wear { item_id } => { self.wear(item_id); }
            BotCommand::Unwear { item_id } => { self.unwear(item_id); }
            BotCommand::Drop { item_id, count } => { self.drop_item(item_id, count); }
            BotCommand::Trash { item_id, count } => { self.trash_item(item_id, count); }
            BotCommand::LeaveWorld => { self.leave_world(); }
            BotCommand::Respawn => { self.respawn(); }
            BotCommand::FindPath { x, y } => { self.find_path(x, y); }
            BotCommand::SetDelays(d) => {
                self.delays = d.clone();
                self.state.write().unwrap().delays = d;
            }
        }
    }

    // ── Lua-callable helpers ────────────────────────────────────────────────────

    pub fn disconnect(&mut self) {
        if let Some(id) = self.peer_id {
            self.host.peer_disconnect(id, 0);
        }
    }

    pub fn reconnect(&mut self) {
        self.reconnect_main();
    }

    pub fn say(&mut self, text: &str) {
        self.send_text(&format!("action|input\n|text|{text}\n"));
    }

    pub fn warp(&mut self, name: &str, id: &str) {
        self.send_game_message(&format!("action|join_request\nname|{name}\ninvitedWorld|{id}\n"));
    }

    pub fn leave_world(&mut self) {
        self.send_game_message("action|quit_to_exit\n");
    }

    pub fn respawn(&mut self) {
        self.send_text("action|respawn\n");
    }

    pub fn unwear(&mut self, item_id: u32) {
        let pkt = GameUpdatePacket {
            packet_type: GamePacketType::ItemActivateObjectRequestAlt,
            value: item_id,
            ..Default::default()
        };
        self.send_game_packet(&pkt, true);
    }

    pub fn active_tile(&mut self, tile_x: i32, tile_y: i32) {
        let pkt = GameUpdatePacket {
            packet_type: GamePacketType::TileActivateRequest,
            vector_x: self.pos_x,
            vector_y: self.pos_y,
            int_x: tile_x,
            int_y: tile_y,
            ..Default::default()
        };
        self.send_game_packet(&pkt, true);
    }

    pub fn place_at(&mut self, tile_x: i32, tile_y: i32, item_id: u32) {
        let base_x = (self.pos_x / 32.0).floor() as i32;
        let base_y = (self.pos_y / 32.0).floor() as i32;
        self.place(tile_x - base_x, tile_y - base_y, item_id, false);
    }

    pub fn hit_at(&mut self, tile_x: i32, tile_y: i32) {
        let base_x = (self.pos_x / 32.0).floor() as i32;
        let base_y = (self.pos_y / 32.0).floor() as i32;
        self.punch(tile_x - base_x, tile_y - base_y);
    }

    pub fn wrench_at(&mut self, tile_x: i32, tile_y: i32) {
        let base_x = (self.pos_x / 32.0).floor() as i32;
        let base_y = (self.pos_y / 32.0).floor() as i32;
        self.wrench(tile_x - base_x, tile_y - base_y);
    }

    pub fn set_direction(&mut self, facing_left: bool) {
        let mut flags = packet::PacketFlags::STANDING;
        flags.set(packet::PacketFlags::FACING_LEFT, facing_left);
        let pkt = GameUpdatePacket {
            packet_type: GamePacketType::State,
            net_id: self.local.net_id,
            vector_x: self.pos_x,
            vector_y: self.pos_y,
            int_x: -1,
            int_y: -1,
            flags,
            ..Default::default()
        };
        self.send_game_packet(&pkt, true);
    }

    pub fn fast_drop(&mut self, item_id: u32, count: u32) {
        self.send_text(&format!(
            "action|dialog_return\ndialog_name|drop_item\nitemID|{item_id}|\ncount|{count}\n"
        ));
    }

    pub fn fast_trash(&mut self, item_id: u32, count: u32) {
        self.send_text(&format!(
            "action|dialog_return\ndialog_name|trash_item\nitemID|{item_id}|\ncount|{count}\n"
        ));
    }

    pub fn collect_object_at(&mut self, uid: u32, range_tiles: f32) {
        let obj = match &self.world {
            Some(w) => w.objects.iter().find(|o| o.uid == uid).cloned(),
            None => return,
        };
        if let Some(obj) = obj {
            let dx = self.pos_x - obj.x;
            let dy = self.pos_y - obj.y;
            let range_px = range_tiles * 32.0;
            if dx * dx + dy * dy <= range_px * range_px {
                let pkt = GameUpdatePacket {
                    packet_type: GamePacketType::ItemActivateObjectRequest,
                    vector_x: obj.x,
                    vector_y: obj.y,
                    value: obj.uid,
                    ..Default::default()
                };
                self.send_game_packet(&pkt, true);
            }
        }
    }

    /// Returns path nodes as (x, y) tile pairs without walking.
    pub fn compute_path(&mut self, to_x: u32, to_y: u32) -> Vec<(u32, u32)> {
        let world = match &self.world {
            Some(w) => w,
            None => return vec![],
        };
        let width  = world.tile_map.width;
        let height = world.tile_map.height;

        let tiles: Vec<(u16, u8)> = world.tile_map.tiles.iter().map(|t| {
            let ct = match &t.tile_type {
                TileType::Lock { .. } => 3,
                TileType::Door { .. } => 0,
                _ => self.items_dat
                    .find_by_id(t.fg_item_id as u32)
                    .map(|i| i.collision_type)
                    .unwrap_or(if t.fg_item_id == 0 { 0 } else { 1 }),
            };
            (t.fg_item_id, ct)
        }).collect();

        let _ = world; // end the borrow before mutating astar

        self.astar.update_from_tiles(width, height, &tiles);

        let from_x    = (self.pos_x / 32.0) as u32;
        let from_y    = (self.pos_y / 32.0) as u32;
        let has_access = self.has_access();

        self.astar.find_path(from_x, from_y, to_x, to_y, has_access)
            .unwrap_or_default()
            .into_iter()
            .map(|n| (n.x, n.y))
            .collect()
    }

    pub fn run_script(&mut self, script: &str) {
        crate::lua_api::run_script(self, script);
    }

    /// Called from within a Lua `sleep()` to keep the bot alive.
    /// Services ENet and drains pending commands, skipping RunScript to avoid
    /// re-entrancy. StopScript sets the shared flag so the Lua hook aborts.
    pub fn tick_during_script(&mut self, script_stop: &Arc<AtomicBool>) {
        self.service_once();
        while let Ok(cmd) = self.cmd_rx.try_recv() {
            match cmd {
                BotCommand::StopScript => {
                    script_stop.store(true, Ordering::Relaxed);
                }
                BotCommand::RunScript { .. } => {
                    // Already running a script; ignore.
                }
                other => self.handle_command(other),
            }
        }
    }
}
