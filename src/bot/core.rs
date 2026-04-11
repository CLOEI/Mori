use crate::astar::AStar;
use crate::bot_state::{
    BotCommand, BotDelays, BotState, BotStatus, CmdReceiver, InvSlot, PlayerInfo, TileInfo,
    WorldObjectInfo,
};
use crate::constants::{FHASH, GAME_VER, PROTOCOL};
use crate::protocol::crypto::{compute_klv, generate_rid, hash_string, random_hex, random_mac};
use crate::events::{WsEvent, WsInvItem, WsObject, WsTile, WsTx};
use crate::inventory::Inventory;
use crate::items::ItemsDat;
use crate::login::check_token;
use crate::protocol::packet::{self, GamePacketType, GameUpdatePacket, IncomingPacket};
use crate::player::{LocalPlayer, Player, parse_pipe_map};
use crate::server_data::{LoginInfo, get_server_data_proxied};
use crate::socks5::Socks5UdpSocket;
use crate::protocol::variant::VariantList;
use crate::world::{NpcAction, NpcType, TileType, World, WorldNpc, WorldObject};
use rusty_enet as enet;
use std::collections::HashSet;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use super::auth::fetch_credentials;
use super::shared::{BotEventRaw, Socks5Config, TemporaryData};

enum BotHost {
    Direct(enet::Host<UdpSocket>),
    Socks5(enet::Host<Socks5UdpSocket>),
}

impl BotHost {
    fn next_event(&mut self) -> Option<enet::EventNoRef> {
        match self {
            Self::Direct(h) => {
                if let Some(e) = h.service().expect("service failed") {
                    Some(e.no_ref())
                } else {
                    None
                }
            }
            Self::Socks5(h) => {
                if let Some(e) = h.service().expect("service failed") {
                    Some(e.no_ref())
                } else {
                    None
                }
            }
        }
    }

    fn connect(&mut self, addr: SocketAddr, channels: usize, data: u32) {
        match self {
            Self::Direct(h) => {
                h.connect(addr, channels, data).expect("connect failed");
            }
            Self::Socks5(h) => {
                h.connect(addr, channels, data).expect("connect failed");
            }
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
            Self::Direct(h) => {
                h.peer_mut(id).send(channel, packet).ok();
            }
            Self::Socks5(h) => {
                h.peer_mut(id).send(channel, packet).ok();
            }
        }
    }

    fn peer_disconnect(&mut self, id: enet::PeerID, data: u32) {
        match self {
            Self::Direct(h) => {
                h.peer_mut(id).disconnect(data);
            }
            Self::Socks5(h) => {
                h.peer_mut(id).disconnect(data);
            }
        }
    }
}

/// How the bot authenticates. Controls both initial login and token-refresh fallback.
enum LoginMethod {
    /// Standard GrowID login: if check_token fails, re-login with password.
    Legacy { password: String },
    /// Token provided directly: if check_token fails, stop the bot (no fallback).
    Ltoken,
}

/// Data captured from `OnSendToServer`, kept until the next ServerHello.
struct RedirectData {
    server: String,
    port: u16,
    token: String,
    user: String,
    door_id: String,
    uuid: String,
    lmode: String,
    tank_id_name: String,
}

pub struct Bot {
    host: BotHost,
    pub proxy: Option<Socks5Config>,
    pub username: String,
    login_method: LoginMethod,
    /// Legacy token from HTTP login (used in first ServerHello only).
    ltoken: String,
    /// `meta` from server_data.php — echoed in all login packets.
    meta: String,
    /// Per-session random values computed once at startup.
    pub mac: String,
    hash: i32,
    hash2: i32,
    wk: String,
    rid: String,
    last_redirect_token: Option<String>,
    last_redirect_uuid: Option<String>,
    /// Set by `OnSendToServer`; consumed by the next ServerHello.
    redirect: Option<RedirectData>,
    /// When the bot connected — used for network time in ping replies.
    start_time: std::time::Instant,
    /// Current position in the world (pixels).
    pub pos_x: f32,
    pub pos_y: f32,
    /// The bot's own identity in the current world.
    pub local: LocalPlayer,
    /// Other players present in the current world, keyed by net_id.
    pub players: std::collections::HashMap<u32, Player>,
    /// The bot's inventory, updated on SendInventoryState.
    pub inventory: Inventory,
    /// The current world, updated on SendMapData.
    pub world: Option<World>,
    /// Active peer, set on Connect and cleared on Disconnect.
    peer_id: Option<enet::PeerID>,
    /// Shared state written by the bot and read by the web layer.
    pub state: Arc<RwLock<BotState>>,
    /// Commands sent from the web layer to be executed each tick.
    cmd_rx: CmdReceiver,
    /// One-shot callback fired on the next OnDialogRequest.
    pub temporary_data: TemporaryData,
    /// Whether the run loop should auto-collect nearby dropped items.
    pub auto_collect: bool,
    /// Skip gems (item ID 112) during auto-collect when true.
    pub ignore_gems: bool,
    /// Skip essences (item IDs 5024/5026/5028/5030) during auto-collect when true.
    pub ignore_essences: bool,
    /// Leave world automatically when a mod is detected via OnSpawn.
    pub auto_leave_on_mod: bool,
    /// Send `/ban <name>` when any non-local player spawns.
    pub auto_ban: bool,
    /// Whether the bot should automatically reconnect after a disconnect.
    pub auto_reconnect: bool,
    /// Delay in ms before reconnecting after a disconnect (default 0).
    pub reconnect_interval: u64,
    /// Auto-collect range in tiles (1–5); pixel radius is `tiles × 32`.
    collect_radius_tiles: u8,
    /// Item IDs excluded from auto-collect.
    collect_blacklist: HashSet<u16>,
    /// Tracks when collect() was last run.
    collect_timer: std::time::Instant,
    /// Interval in ms between auto-collect ticks (default 500).
    collect_interval: u64,
    /// Skip objects with no reachable A* path during auto-collect.
    pub collect_path_check: bool,
    /// A* pathfinder, re-used across find_path calls.
    astar: AStar,
    /// While `Some`, we are routing to this tile; `OnSetPos` triggers replanning from server position.
    pathfind_target: Option<(u32, u32)>,
    /// Set during pathfinding when the server sends `OnSetPos` so we abandon the current segment and replan.
    pathfind_recalc: bool,
    /// Configurable delays for bot actions.
    pub delays: BotDelays,
    /// Item database for collision-type lookups.
    pub items_dat: Arc<ItemsDat>,
    /// Forwards events to the running script thread (None when no script is active).
    event_tx: Option<crossbeam_channel::Sender<BotEventRaw>>,
    /// Receives requests from the script thread.
    script_req_rx: Option<crossbeam_channel::Receiver<crate::script_channel::ScriptRequest>>,
    /// Sends replies back to the script thread.
    script_reply_tx: Option<crossbeam_channel::Sender<crate::script_channel::ScriptReply>>,
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
    /// Set when an `action|log` with "UPDATE REQUIRED" is received,
    /// so the subsequent `action|logon_fail` stops the bot entirely.
    pending_update_required: bool,
    /// Set when an `action|log` with "undergoing maintenance" is received,
    /// so the subsequent `action|logon_fail` knows to apply the 600 s cooldown.
    pending_maintenance: bool,
    /// Set to true to make the `run` loop exit on the next iteration.
    stop_requested: bool,
    /// This bot's ID in the BotManager (used to tag WS events).
    pub bot_id: u32,
    /// Broadcast sender for real-time WebSocket events (None when running standalone).
    ws_tx: Option<WsTx>,
    /// Last broadcast ping value — used to suppress redundant BotPing events.
    last_ping: u32,
}

fn sorted_blacklist_vec(set: &HashSet<u16>) -> Vec<u16> {
    let mut v: Vec<u16> = set.iter().copied().collect();
    v.sort_unstable();
    v
}

impl Bot {
    pub fn new(
        username: &str,
        password: &str,
        proxy: Option<Socks5Config>,
        state: Arc<RwLock<BotState>>,
        cmd_rx: CmdReceiver,
        items_dat: Arc<ItemsDat>,
        bot_id: u32,
        ws_tx: Option<WsTx>,
    ) -> Self {
        let log_state = Arc::clone(&state);
        let log_ws_tx = ws_tx.clone();
        let log_bot_id = bot_id;
        let mut log_fn = move |msg: String| {
            println!("{msg}");
            {
                let mut s = log_state.write().unwrap();
                s.console.push(msg.clone());
                if s.console.len() > 100 {
                    s.console.remove(0);
                }
            }
            if let Some(tx) = &log_ws_tx {
                let _ = tx.send(WsEvent::Console {
                    bot_id: log_bot_id,
                    message: msg,
                });
            }
        };
        let creds = fetch_credentials(username, password, proxy.as_ref(), &mut log_fn);

        let mac = random_mac();
        let hash = hash_string(&format!("{}RT", mac));
        let hash2 = hash_string(&format!("{}RT", random_hex(16)));
        let wk = random_hex(32);
        let rid = generate_rid();

        let host = Self::create_host(proxy.as_ref());
        let mut bot = Bot {
            host,
            proxy,
            username: username.to_string(),
            login_method: LoginMethod::Legacy {
                password: password.to_string(),
            },
            ltoken: creds.ltoken,
            meta: creds.meta,
            mac,
            hash,
            hash2,
            wk,
            rid,
            last_redirect_token: None,
            last_redirect_uuid: None,
            redirect: None,
            peer_id: None,
            pos_x: 0.0,
            pos_y: 0.0,
            start_time: std::time::Instant::now(),
            local: LocalPlayer::default(),
            players: std::collections::HashMap::new(),
            inventory: Inventory::default(),
            world: None,
            state,
            cmd_rx,
            temporary_data: TemporaryData::default(),
            auto_collect: true,
            ignore_gems: false,
            ignore_essences: false,
            auto_leave_on_mod: false,
            auto_ban: false,
            auto_reconnect: true,
            reconnect_interval: 0,
            collect_radius_tiles: 3,
            collect_blacklist: HashSet::new(),
            collect_timer: std::time::Instant::now(),
            collect_interval: 500,
            collect_path_check: true,
            astar: AStar::new(),
            pathfind_target: None,
            pathfind_recalc: false,
            delays: BotDelays::default(),
            items_dat,
            event_tx: None,
            script_req_rx: None,
            script_reply_tx: None,
            script_stop: Arc::new(AtomicBool::new(false)),
            reconnect_after: None,
            pending_2fa: false,
            pending_relogon: false,
            pending_server_overload: false,
            pending_too_many_logins: false,
            pending_update_required: false,
            pending_maintenance: false,
            stop_requested: false,
            bot_id,
            ws_tx,
            last_ping: 0,
        };

        {
            let mut s = bot.state.write().unwrap();
            s.username = username.to_string();
            s.mac = bot.mac.clone();
            s.collect_radius_tiles = bot.collect_radius_tiles;
            s.collect_blacklist = sorted_blacklist_vec(&bot.collect_blacklist);
        }
        bot.host.connect(creds.addr, 2, 0);
        bot
    }

    /// Parses a `token|rid|mac|wk` string.
    fn parse_ltoken_string(s: &str) -> Option<(String, String, String, String)> {
        let mut parts = s.splitn(4, '|');
        let token = parts.next()?.to_string();
        let rid = parts.next()?.to_string();
        let mac = parts.next()?.to_string();
        let wk = parts.next()?.to_string();
        if rid.len() != 32 || wk.len() != 32 {
            return None;
        }
        Some((token, rid, mac, wk))
    }

    pub fn new_ltoken(
        ltoken_str: &str,
        proxy: Option<Socks5Config>,
        state: Arc<RwLock<BotState>>,
        cmd_rx: CmdReceiver,
        items_dat: Arc<ItemsDat>,
        bot_id: u32,
        ws_tx: Option<WsTx>,
    ) -> Self {
        let (ltoken, rid, mac, wk) = Self::parse_ltoken_string(ltoken_str)
            .expect("[Bot] Invalid ltoken string — expected token|rid|mac|wk");

        let hash = hash_string(&format!("{}RT", mac));
        let hash2 = hash_string(&format!("{}RT", random_hex(16)));

        let proxy_url = proxy.as_ref().map(|p| p.to_url());
        let proxy_url_ref = proxy_url.as_deref();
        let login_info = LoginInfo {
            protocol: PROTOCOL,
            game_version: GAME_VER.into(),
        };

        let log_state = Arc::clone(&state);
        let log_ws_tx = ws_tx.clone();
        let log_bot_id = bot_id;
        let log_fn = move |msg: String| {
            println!("{msg}");
            {
                let mut s = log_state.write().unwrap();
                s.console.push(msg.clone());
                if s.console.len() > 100 {
                    s.console.remove(0);
                }
            }
            if let Some(tx) = &log_ws_tx {
                let _ = tx.send(WsEvent::Console {
                    bot_id: log_bot_id,
                    message: msg,
                });
            }
        };

        let mut alternate = false;
        let server_data = loop {
            log_fn(format!(
                "[Bot] fetching server_data (alternate={alternate})..."
            ));
            match get_server_data_proxied(alternate, &login_info, proxy_url_ref) {
                Ok(s) => break s,
                Err(e) => {
                    alternate = !alternate;
                    log_fn(format!(
                        "[Bot] fetch: server_data failed: {e} — retrying in 5s"
                    ));
                    std::thread::sleep(std::time::Duration::from_secs(5));
                }
            }
        };

        let klv = compute_klv(GAME_VER, &PROTOCOL.to_string(), &rid, hash);
        let login_data = format!(
            "tankIDName|\ntankIDPass|\nrequestedName|\nf|1\nprotocol|{PROTOCOL}\n\
game_version|{GAME_VER}\nfz|22243512\ncbits|1024\nplayer_age|20\nGDPR|2\nFCMToken|\n\
category|_-5100\ntotalPlaytime|0\nklv|{klv}\nhash2|{hash2}\nmeta|{}\nfhash|{FHASH}\n\
rid|{rid}\nplatformID|0,1,1\ndeviceVersion|0\ncountry|jp\nhash|{hash}\nmac|{mac}\nwk|{wk}\nzf|31631978\nlmode|1\n",
            server_data.meta,
        );

        let ltoken = match check_token(&ltoken, &login_data, proxy_url_ref) {
            Ok(new_token) => {
                log_fn(format!("[Bot] ltoken validated successfully"));
                new_token
            }
            Err(e) => panic!("[Bot] ltoken validation failed: {e} — stopping"),
        };

        let addr: SocketAddr = format!("{}:{}", server_data.server, server_data.port)
            .parse()
            .expect("Invalid server address");

        let host = Self::create_host(proxy.as_ref());
        let mut bot = Bot {
            host,
            proxy,
            username: String::new(),
            login_method: LoginMethod::Ltoken,
            ltoken,
            meta: server_data.meta,
            mac,
            hash,
            hash2,
            wk,
            rid,
            last_redirect_token: None,
            last_redirect_uuid: None,
            redirect: None,
            peer_id: None,
            pos_x: 0.0,
            pos_y: 0.0,
            start_time: std::time::Instant::now(),
            local: LocalPlayer::default(),
            players: std::collections::HashMap::new(),
            inventory: Inventory::default(),
            world: None,
            state,
            cmd_rx,
            temporary_data: TemporaryData::default(),
            auto_collect: true,
            ignore_gems: false,
            ignore_essences: false,
            auto_leave_on_mod: false,
            auto_ban: false,
            auto_reconnect: true,
            reconnect_interval: 0,
            collect_radius_tiles: 3,
            collect_blacklist: HashSet::new(),
            collect_timer: std::time::Instant::now(),
            collect_interval: 500,
            collect_path_check: true,
            astar: AStar::new(),
            pathfind_target: None,
            pathfind_recalc: false,
            delays: BotDelays::default(),
            items_dat,
            event_tx: None,
            script_req_rx: None,
            script_reply_tx: None,
            script_stop: Arc::new(AtomicBool::new(false)),
            reconnect_after: None,
            pending_2fa: false,
            pending_relogon: false,
            pending_server_overload: false,
            pending_too_many_logins: false,
            pending_update_required: false,
            pending_maintenance: false,
            stop_requested: false,
            bot_id,
            ws_tx,
            last_ping: 0,
        };

        {
            let mut s = bot.state.write().unwrap();
            s.mac = bot.mac.clone();
            s.collect_radius_tiles = bot.collect_radius_tiles;
            s.collect_blacklist = sorted_blacklist_vec(&bot.collect_blacklist);
        }
        bot.host.connect(addr, 2, 0);
        bot
    }

    fn reconnect_main(&mut self) {
        self.host = Self::create_host(self.proxy.as_ref());
        self.refresh_token();

        let login_info = LoginInfo {
            protocol: PROTOCOL,
            game_version: GAME_VER.into(),
        };
        let proxy_url = self.proxy.as_ref().map(|p| p.to_url());
        let mut alternate = false;
        let server_data = loop {
            match get_server_data_proxied(alternate, &login_info, proxy_url.as_deref()) {
                Ok(s) => break s,
                Err(e) => {
                    alternate = !alternate;
                    self.log_console(format!(
                        "[Bot] reconnect: server_data failed: {e} — retrying in 5s"
                    ));
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
            peer_limit: 1,
            channel_limit: 2,
            compressor: Some(Box::new(enet::RangeCoder::new())),
            checksum: Some(Box::new(enet::crc32)),
            using_new_packet: true,
            ..Default::default()
        };
        match proxy {
            None => {
                let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket");
                BotHost::Direct(
                    enet::Host::new(socket, settings).expect("Failed to create ENet host"),
                )
            }
            Some(cfg) => {
                let local: SocketAddr = "0.0.0.0:0".parse().unwrap();
                let socket = Socks5UdpSocket::bind_through_proxy(
                    local,
                    cfg.proxy_addr,
                    cfg.username.as_deref(),
                    cfg.password.as_deref(),
                )
                .expect("Failed to create SOCKS5 UDP socket");
                BotHost::Socks5(
                    enet::Host::new(socket, settings).expect("Failed to create ENet host"),
                )
            }
        }
    }

    fn emit(&self, event: WsEvent) {
        if let Some(tx) = &self.ws_tx {
            let _ = tx.send(event);
        }
    }

    fn log_console(&self, msg: String) {
        println!("{msg}");
        {
            let mut s = self.state.write().unwrap();
            s.console.push(msg.clone());
            if s.console.len() > 100 {
                s.console.remove(0);
            }
        }
        self.emit(WsEvent::Console {
            bot_id: self.bot_id,
            message: msg,
        });
    }

    fn build_login_packet(&self) -> String {
        format!(
            "protocol|{PROTOCOL}\nltoken|{}\nplatformID|2\n",
            self.ltoken
        )
    }

    fn build_redirect_packet(&self, r: &RedirectData) -> String {
        let klv = compute_klv(GAME_VER, &PROTOCOL.to_string(), &self.rid, self.hash);
        let mut data = String::new();
        data.push_str(&format!("tankIDName|{}\n", r.tank_id_name));
        data.push_str("tankIDPass|\n");
        data.push_str("requestedName|\n");
        data.push_str("f|1\n");
        data.push_str("protocol|211\n");
        data.push_str(&format!("game_version|{}\n", GAME_VER));
        data.push_str("fz|47142936\n");
        data.push_str("cbits|1536\n");
        data.push_str("player_age|18\n");
        data.push_str("GDPR|1\n");
        data.push_str("FCMToken|\n");
        data.push_str("category|_-5100\n");
        data.push_str("totalPlaytime|0\n");
        data.push_str(&format!("klv|{klv}\n"));
        data.push_str(&format!("hash2|{}\n", self.hash2));
        data.push_str(&format!("meta|{}\n", self.meta));
        data.push_str(&format!("fhash|{FHASH}\n"));
        data.push_str(&format!("rid|{}\n", self.rid));
        data.push_str("platformID|0,1,1\n");
        data.push_str("deviceVersion|0\n");
        data.push_str("country|ma\n");
        data.push_str(&format!("hash|{}\n", self.hash));
        data.push_str(&format!("mac|{}\n", self.mac));
        data.push_str(&format!("wk|{}\n", self.wk));
        data.push_str("zf|-821693372\n");
        data.push_str(&format!("lmode|{}\n", r.lmode));
        data.push_str(&format!("user|{}\n", r.user));
        data.push_str(&format!("token|{}\n", r.token));
        data.push_str(&format!("UUIDToken|{}\n", r.uuid));
        if !r.door_id.is_empty() {
            data.push_str(&format!("doorID|{}\n", r.door_id));
        }
        data.push_str("aat|2\n");
        data
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

    /// Refreshes `self.ltoken`: tries check_token first, then falls back based on login method.
    fn refresh_token(&mut self) {
        let login_data = self.build_login_data();
        let proxy = self.proxy.as_ref().map(|p| p.to_url());
        let proxy_url = proxy.as_deref();

        if !self.ltoken.is_empty() {
            if let Ok(new_token) = check_token(&self.ltoken, &login_data, proxy_url) {
                self.log_console("[Bot] Token refreshed via check_token".to_string());
                self.ltoken = new_token;
                return;
            }
            self.log_console("[Bot] check_token failed".to_string());
        }

        match &self.login_method {
            LoginMethod::Ltoken => {
                self.log_console(
                    "[Bot] ltoken login — no fallback credentials, stopping bot".to_string(),
                );
                self.stop_requested = true;
            }
            LoginMethod::Legacy { password } => {
                let password = password.clone();
                self.log_console("[Bot] falling back to full re-login".to_string());
                let username = self.username.clone();
                let proxy_clone = self.proxy.clone();
                let state = Arc::clone(&self.state);
                let ws_tx = self.ws_tx.clone();
                let bot_id = self.bot_id;
                let mut log_fn = move |msg: String| {
                    println!("{msg}");
                    {
                        let mut s = state.write().unwrap();
                        s.console.push(msg.clone());
                        if s.console.len() > 100 {
                            s.console.remove(0);
                        }
                    }
                    if let Some(tx) = &ws_tx {
                        let _ = tx.send(WsEvent::Console {
                            bot_id,
                            message: msg,
                        });
                    }
                };
                let creds =
                    fetch_credentials(&username, &password, proxy_clone.as_ref(), &mut log_fn);
                self.ltoken = creds.ltoken;
                self.meta = creds.meta;
            }
        }
    }

    pub fn run(&mut self, stop_flag: Arc<AtomicBool>) {
        loop {
            if stop_flag.load(Ordering::Relaxed) {
                self.log_console("[Bot] Stop flag set, exiting.".to_string());
                break;
            }
            if self.stop_requested {
                self.log_console("[Bot] Stop requested internally, exiting.".to_string());
                break;
            }
            // Check if a delayed reconnect (e.g. 2FA cooldown) is ready.
            if let Some(at) = self.reconnect_after {
                if std::time::Instant::now() >= at {
                    self.reconnect_after = None;
                    if self.auto_reconnect {
                        self.log_console(
                            "[Bot] 2FA cooldown elapsed — re-fetching token and server data"
                                .to_string(),
                        );
                        self.reconnect_main();
                    }
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
                    self.emit(WsEvent::BotPing {
                        bot_id: self.bot_id,
                        ping_ms: rtt,
                    });
                }
            }
            self.service_once();
            self.drain_script_requests();
            if self.auto_collect
                && self.collect_timer.elapsed() >= std::time::Duration::from_millis(self.collect_interval)
            {
                self.collect_timer = std::time::Instant::now();
                self.collect();
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    /// Sleep for `ms` milliseconds while keeping ENet alive.
    pub fn sleep_ms(&mut self, ms: u64) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(ms);
        while std::time::Instant::now() < deadline {
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
                    self.log_console(format!("[Bot] Connected: peer {}", id.0));
                }

                enet::EventNoRef::Disconnect { peer: id, .. } => {
                    self.peer_id = None;
                    self.pathfind_target = None;
                    self.pathfind_recalc = false;
                    self.log_console(format!("[Bot] Disconnected: peer {}", id.0));
                    {
                        let mut s = self.state.write().unwrap();
                        s.status = BotStatus::Connecting;
                        s.world_name = String::new();
                        s.players = Vec::new();
                        s.ping_ms = 0;
                    }
                    self.emit(WsEvent::BotStatus {
                        bot_id: self.bot_id,
                        status: "connecting".into(),
                    });
                    self.emit(WsEvent::BotWorld {
                        bot_id: self.bot_id,
                        world_name: String::new(),
                    });
                    if let Some(r) = self.redirect.as_ref() {
                        let addr: SocketAddr = format!("{}:{}", r.server, r.port)
                            .parse()
                            .expect("Invalid redirect address");
                        self.log_console(format!("[Bot] Redirecting to {}:{}", r.server, r.port));
                        self.host = Self::create_host(self.proxy.as_ref());
                        self.host.connect(addr, 2, 0);
                    } else if self.reconnect_after.is_some() {
                        // Delayed reconnect already scheduled (e.g. 2FA cooldown) — do nothing here.
                    } else if self.auto_reconnect {
                        if self.reconnect_interval > 0 {
                            self.log_console(format!(
                                "[Bot] Server disconnected — reconnecting in {}ms",
                                self.reconnect_interval
                            ));
                            self.reconnect_after = Some(
                                std::time::Instant::now()
                                    + std::time::Duration::from_millis(self.reconnect_interval),
                            );
                        } else {
                            self.log_console(
                                "[Bot] Server disconnected — re-fetching token and server data"
                                    .to_string(),
                            );
                            self.reconnect_main();
                        }
                    } else {
                        self.log_console(
                            "[Bot] Server disconnected — auto-reconnect is disabled".to_string(),
                        );
                    }
                }

                enet::EventNoRef::Receive {
                    peer: id,
                    channel_id,
                    packet,
                } => {
                    match IncomingPacket::parse(packet.data()) {
                        Some(IncomingPacket::ServerHello) => {
                            self.on_server_hello();
                        }
                        Some(IncomingPacket::Text(s)) => {
                            self.log_console(format!("[Bot] Text: {s}"));
                        }
                        Some(IncomingPacket::GameMessage(s)) => {
                            self.log_console(format!("[Bot] GameMessage: {s}"));
                            if let Some(tx) = &self.event_tx {
                                tx.try_send(BotEventRaw::GameMessage {
                                    text: s.to_string(),
                                })
                                .ok();
                            }
                            if s.contains("Advanced Account Protection") {
                                self.pending_2fa = true;
                            }
                            if s.contains("action|log") && s.contains("SERVER OVERLOADED") {
                                self.pending_server_overload = true;
                            }
                            if s.contains("action|log") && s.contains("Too many people logging in")
                            {
                                self.pending_too_many_logins = true;
                            }
                            if s.contains("action|log")
                                && s.contains("Server requesting that you re-logon")
                            {
                                self.log_console(
                                    "[Bot] Server requested re-logon — clearing redirect data."
                                        .to_string(),
                                );
                                self.redirect = None;
                                self.pending_relogon = true;
                            }
                            if s.contains("action|log") && s.contains("UPDATE REQUIRED") {
                                self.pending_update_required = true;
                            }
                            if s.contains("action|log") && s.contains("undergoing maintenance") {
                                self.pending_maintenance = true;
                            }
                            if s.contains("action|logon_fail") {
                                if self.pending_2fa {
                                    self.pending_2fa = false;
                                    let secs = self.delays.twofa_secs;
                                    self.log_console(format!(
                                        "[Bot] Logon failed — 2FA (Advanced Account Protection). Retrying in {secs} s."
                                    ));
                                    self.state.write().unwrap().status = BotStatus::TwoFactorAuth;
                                    self.reconnect_after = Some(
                                        std::time::Instant::now()
                                            + std::time::Duration::from_secs(secs),
                                    );
                                    self.emit(WsEvent::BotStatus {
                                        bot_id: self.bot_id,
                                        status: "two_factor_auth".into(),
                                    });
                                } else if self.pending_server_overload {
                                    self.pending_server_overload = false;
                                    let secs = self.delays.server_overload_secs;
                                    self.log_console(format!(
                                        "[Bot] Logon failed — server overloaded. Retrying in {secs} s."
                                    ));
                                    self.state.write().unwrap().status =
                                        BotStatus::ServerOverloaded;
                                    self.reconnect_after = Some(
                                        std::time::Instant::now()
                                            + std::time::Duration::from_secs(secs),
                                    );
                                    self.emit(WsEvent::BotStatus {
                                        bot_id: self.bot_id,
                                        status: "server_overloaded".into(),
                                    });
                                } else if self.pending_too_many_logins {
                                    self.pending_too_many_logins = false;
                                    let secs = self.delays.too_many_logins_secs;
                                    self.log_console(format!(
                                        "[Bot] Logon failed — too many logins at once. Retrying in {secs} s."
                                    ));
                                    self.state.write().unwrap().status = BotStatus::TooManyLogins;
                                    self.reconnect_after = Some(
                                        std::time::Instant::now()
                                            + std::time::Duration::from_secs(secs),
                                    );
                                    self.emit(WsEvent::BotStatus {
                                        bot_id: self.bot_id,
                                        status: "too_many_logins".into(),
                                    });
                                } else if self.pending_relogon {
                                    self.pending_relogon = false;
                                    self.log_console(
                                        "[Bot] Logon failed — server requested re-logon. Reconnecting.".to_string()
                                    );
                                } else if self.pending_update_required {
                                    self.pending_update_required = false;
                                    self.log_console(
                                        "[Bot] Logon failed — client update required. Stopping bot.".to_string()
                                    );
                                    self.state.write().unwrap().status = BotStatus::UpdateRequired;
                                    self.emit(WsEvent::BotStatus {
                                        bot_id: self.bot_id,
                                        status: "update_required".into(),
                                    });
                                    self.stop_requested = true;
                                } else if self.pending_maintenance {
                                    self.pending_maintenance = false;
                                    let secs = self.delays.maintenance_secs;
                                    self.log_console(format!(
                                        "[Bot] Logon failed — server maintenance. Retrying in {secs} s."
                                    ));
                                    self.state.write().unwrap().status = BotStatus::Maintenance;
                                    self.reconnect_after = Some(
                                        std::time::Instant::now()
                                            + std::time::Duration::from_secs(secs),
                                    );
                                    self.emit(WsEvent::BotStatus {
                                        bot_id: self.bot_id,
                                        status: "maintenance".into(),
                                    });
                                } else {
                                    self.log_console(
                                        "[Bot] Logon failed — disconnecting to re-fetch token"
                                            .to_string(),
                                    );
                                }
                                // self.redirect = None;
                                self.host.peer_disconnect(id, 0);
                            }
                        }
                        Some(IncomingPacket::GameUpdate(pkt)) => {
                            if let Some(tx) = &self.event_tx {
                                tx.try_send(BotEventRaw::GameUpdate { pkt: pkt.clone() })
                                    .ok();
                            }
                            match pkt.packet_type {
                                GamePacketType::SetCharacterState => {
                                    self.local.hack_type = pkt.value;
                                    self.local.build_length = pkt.jump_count.saturating_sub(126);
                                    self.local.punch_length =
                                        pkt.animation_type.saturating_sub(126);
                                    self.local.gravity = pkt.vector_x2;
                                    self.local.velocity = pkt.vector_y2;
                                }
                                GamePacketType::CallFunction => {
                                    let extra = pkt.extra_data.clone();
                                    let net_id = id.0 as u32;
                                    if let Ok(vl) = VariantList::deserialize(&extra) {
                                        if let Some(tx) = &self.event_tx {
                                            tx.try_send(BotEventRaw::VariantList { vl, net_id })
                                                .ok();
                                        }
                                    }
                                    self.on_call_function(id, &extra);
                                }
                                GamePacketType::PingRequest => {
                                    self.on_ping_request(pkt.value);
                                }
                                GamePacketType::SendInventoryState => {
                                    match Inventory::parse(&pkt.extra_data) {
                                        Ok(inv) => {
                                            self.log_console(format!(
                                                "[Bot] Inventory: {} items",
                                                inv.item_count
                                            ));
                                            self.inventory = inv.clone();
                                            self.emit_inventory_update();
                                        }
                                        Err(e) => self.log_console(format!(
                                            "[Bot] Inventory parse error: {e}"
                                        )),
                                    }
                                }
                                GamePacketType::SendMapData => {
                                    let _ = std::fs::write("world.dat", &pkt.extra_data);
                                    self.players.clear();
                                    self.local = LocalPlayer::default();
                                    match World::parse(&pkt.extra_data) {
                                        Ok(world) => {
                                            self.log_console(format!(
                                                "[Bot] World: {}x{} tiles, {} objects",
                                                world.tile_map.width,
                                                world.tile_map.height,
                                                world.objects.len(),
                                            ));
                                            self.world = Some(world.clone());
                                            let tiles: Vec<TileInfo> = world
                                                .tile_map
                                                .tiles
                                                .iter()
                                                .map(|t| TileInfo {
                                                    fg_item_id: t.fg_item_id,
                                                    bg_item_id: t.bg_item_id,
                                                    flags: t.flags_raw,
                                                    tile_type: t.tile_type.clone(),
                                                })
                                                .collect();
                                            let mut s = self.state.write().unwrap();
                                            s.world_name = world.tile_map.world_name.clone();
                                            s.world_width = world.tile_map.width;
                                            s.world_height = world.tile_map.height;
                                            let objects: Vec<WorldObjectInfo> = world
                                                .objects
                                                .iter()
                                                .map(|o| WorldObjectInfo {
                                                    uid: o.uid,
                                                    item_id: o.item_id,
                                                    x: o.x,
                                                    y: o.y,
                                                    count: o.count,
                                                })
                                                .collect();
                                            s.tiles = tiles;
                                            s.objects = objects;
                                            s.players = Vec::new();
                                            s.status = BotStatus::InGame;
                                            // Emit world-loaded event with full tile data.
                                            let ws_tiles: Vec<WsTile> = world
                                                .tile_map
                                                .tiles
                                                .iter()
                                                .map(|t| WsTile {
                                                    fg: t.fg_item_id,
                                                    bg: t.bg_item_id,
                                                    flags: t.flags_raw,
                                                    tile_type: t.tile_type.clone(),
                                                })
                                                .collect();
                                            let ws_objs: Vec<WsObject> = world
                                                .objects
                                                .iter()
                                                .map(|o| WsObject {
                                                    uid: o.uid,
                                                    item_id: o.item_id,
                                                    x: o.x,
                                                    y: o.y,
                                                    count: o.count,
                                                })
                                                .collect();
                                            drop(s);
                                            self.emit(WsEvent::BotStatus {
                                                bot_id: self.bot_id,
                                                status: "in_game".into(),
                                            });
                                            self.emit(WsEvent::BotWorld {
                                                bot_id: self.bot_id,
                                                world_name: world.tile_map.world_name.clone(),
                                            });
                                            self.emit(WsEvent::WorldLoaded {
                                                bot_id: self.bot_id,
                                                name: world.tile_map.world_name.clone(),
                                                width: world.tile_map.width,
                                                height: world.tile_map.height,
                                                tiles: ws_tiles,
                                            });
                                            self.emit(WsEvent::ObjectsUpdate {
                                                bot_id: self.bot_id,
                                                objects: ws_objs,
                                            });
                                        }
                                        Err(e) => self
                                            .log_console(format!("[Bot] World parse error: {e}")),
                                    }
                                }
                                GamePacketType::State => self.on_state(&pkt),
                                GamePacketType::TileChangeRequest => self.on_tile_change(&pkt),
                                GamePacketType::SendTileUpdateData => {
                                    self.on_send_tile_update_data(&pkt)
                                }
                                GamePacketType::SendTileUpdateDataMultiple => {
                                    self.on_send_tile_update_data_multiple(&pkt)
                                }
                                GamePacketType::SendTileTreeState => {
                                    self.on_send_tile_tree_state(&pkt)
                                }
                                GamePacketType::ModifyItemInventory => {
                                    self.on_modify_item_inventory(&pkt)
                                }
                                GamePacketType::ItemChangeObject => {
                                    self.on_item_change_object(&pkt)
                                }
                                GamePacketType::SendLock => self.on_send_lock(&pkt),
                                GamePacketType::Npc => self.on_npc_packet(&pkt),
                                _ => self.log_console(format!("[Bot] {pkt}")),
                            }
                        }
                        Some(IncomingPacket::Track(s)) => {
                            self.log_console(format!("[Bot] Track: {s}"));
                            let fields: std::collections::HashMap<&str, &str> =
                                s.lines().filter_map(|line| line.split_once('|')).collect();
                            let level = fields
                                .get("Level")
                                .and_then(|v| v.parse::<u32>().ok())
                                .unwrap_or(0);
                            let grow_id = fields
                                .get("GrowId")
                                .and_then(|v| v.parse::<u64>().ok())
                                .unwrap_or(0);
                            let install_date = fields
                                .get("installDate")
                                .and_then(|v| v.parse::<u64>().ok())
                                .unwrap_or(0);
                            let global_playtime = fields
                                .get("Global_Playtime")
                                .and_then(|v| v.parse::<u64>().ok())
                                .unwrap_or(0);
                            let awesomeness = fields
                                .get("Awesomeness")
                                .and_then(|v| v.parse::<u32>().ok())
                                .unwrap_or(0);
                            self.state.write().unwrap().track_info =
                                Some(crate::bot_state::TrackInfo {
                                    level,
                                    grow_id,
                                    install_date,
                                    global_playtime,
                                    awesomeness,
                                });
                            self.emit(WsEvent::BotTrackInfo {
                                bot_id: self.bot_id,
                                level,
                                grow_id,
                                install_date,
                                global_playtime,
                                awesomeness,
                            });
                        }
                        Some(IncomingPacket::ClientLogRequest) => {
                            self.log_console("[Bot] ClientLogRequest".to_string());
                        }
                        Some(IncomingPacket::Unknown { msg_type, data }) => {
                            self.log_console(format!(
                                "[Bot] Unknown msg_type={msg_type} len={}",
                                data.len()
                            ));
                        }
                        None => {
                            let hex = packet
                                .data()
                                .iter()
                                .map(|b| format!("{:02x}", b))
                                .collect::<Vec<_>>()
                                .join(" ");
                            self.log_console(format!(
                                "[Bot] Failed to parse packet ({} bytes on ch {}): {}",
                                packet.data().len(),
                                channel_id,
                                hex
                            ));
                        }
                    }
                }
            }
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
                self.log_console(format!("[Bot] ServerHello (redirect → {})", r.door_id));
                self.build_redirect_packet(&r)
            }
            None => {
                self.log_console("[Bot] ServerHello".to_string());
                self.build_login_packet()
            }
        };
        self.send_text(&data);
    }

    fn on_ping_request(&mut self, challenge: u32) {
        let time_val = self.start_time.elapsed().as_millis() as u32;

        let bx = if self.local.build_length == 0 {
            2.0
        } else {
            self.local.build_length as f32
        };
        let by = if self.local.punch_length == 0 {
            2.0
        } else {
            self.local.punch_length as f32
        };

        let in_world = self.world.is_some();

        let mut reply = GameUpdatePacket {
            packet_type: GamePacketType::PingReply,
            target_net_id: hash_string(&challenge.to_string()),
            value: time_val,
            vector_x: bx * 32.0,
            vector_y: by * 32.0,
            ..Default::default()
        };

        if in_world {
            reply.net_id = self.local.hack_type;
            reply.vector_x2 = self.local.velocity;
            reply.vector_y2 = self.local.gravity;
        }

        self.send_game_packet(&reply, true);
        self.log_console(format!("[Bot] PingReply sent (challenge={})", challenge));
    }

    fn on_call_function(&mut self, id: enet::PeerID, extra_data: &[u8]) {
        let vl = match VariantList::deserialize(extra_data) {
            Ok(v) => v,
            Err(e) => {
                self.log_console(format!("[Bot] VariantList parse error: {e}"));
                return;
            }
        };

        let fn_name = vl.get(0).map(|v| v.as_string()).unwrap_or_default();
        self.log_console(format!("[Bot] CallFunction: {fn_name}"));

        match fn_name.as_str() {
            "OnSendToServer" => {
                let port = vl.get(1).map(|v| v.as_int32()).unwrap_or(0);
                let token = vl.get(2).map(|v| v.as_int32()).unwrap_or(0);
                let user_id = vl.get(3).map(|v| v.as_int32()).unwrap_or(0);
                let server_str = vl.get(4).map(|v| v.as_string()).unwrap_or_default();
                let lmode = vl.get(5).map(|v| v.as_int32()).unwrap_or(0);
                let tank_id_name = vl.get(6).map(|v| v.as_string()).unwrap_or_default();

                let parts: Vec<&str> = server_str.splitn(3, '|').collect();
                let server = parts.first().copied().unwrap_or("").trim_end().to_string();
                let door_id = parts
                    .get(1)
                    .copied()
                    .map(str::trim_end)
                    .filter(|s| !s.is_empty())
                    .unwrap_or("0")
                    .to_string();
                let uuid_raw = parts.get(2).copied().unwrap_or("").trim_end().to_string();
                let token_str = if token >= 0 {
                    let s = token.to_string();
                    self.last_redirect_token = Some(s.clone());
                    s
                } else if let Some(prev) = self.last_redirect_token.clone() {
                    self.log_console(format!(
                        "[Bot] OnSendToServer → reusing previous redirect token for lmode={lmode}"
                    ));
                    prev
                } else {
                    token.to_string()
                };
                let uuid = if !uuid_raw.is_empty() && uuid_raw != "-1" {
                    self.last_redirect_uuid = Some(uuid_raw.clone());
                    uuid_raw
                } else if let Some(prev) = self.last_redirect_uuid.clone() {
                    self.log_console(format!(
                        "[Bot] OnSendToServer → reusing previous UUIDToken for lmode={lmode}"
                    ));
                    prev
                } else {
                    uuid_raw
                };

                self.log_console(format!(
                    "[Bot] OnSendToServer → {server}:{port} door={door_id}"
                ));

                self.redirect = Some(RedirectData {
                    server,
                    port: port as u16,
                    token: token_str,
                    user: user_id.to_string(),
                    door_id,
                    uuid,
                    lmode: lmode.to_string(),
                    tank_id_name,
                });

                self.host.peer_disconnect(id, 0);
            }
            "OnSpawn" => {
                let message = vl.get(1).map(|v| v.as_string()).unwrap_or_default();
                let data = parse_pipe_map(&message);

                if data.contains_key("type") {
                    // Local player spawning — store our own identity
                    self.local.net_id = data.get("netID").and_then(|s| s.parse().ok()).unwrap_or(0);
                    self.local.user_id =
                        data.get("userID").and_then(|s| s.parse().ok()).unwrap_or(0);
                    self.log_console(format!(
                        "[Bot] OnSpawn (self) net_id={} user_id={}",
                        self.local.net_id, self.local.user_id
                    ));
                    self.log_console(format!(
                        "[Bot] ltoken string: {}|{}|{}|{}",
                        self.ltoken, self.rid, self.mac, self.wk
                    ));
                    {
                        let mut s = self.state.write().unwrap();
                        s.status = BotStatus::InGame;
                    }
                    self.emit(WsEvent::BotStatus {
                        bot_id: self.bot_id,
                        status: "in_game".into(),
                    });
                } else {
                    let position = if let Some(pos_xy) = data.get("posXY") {
                        let parts: Vec<f32> = pos_xy
                            .split('|')
                            .filter_map(|s| s.trim().parse().ok())
                            .collect();
                        (
                            *parts.first().unwrap_or(&0.0),
                            *parts.get(1).unwrap_or(&0.0),
                        )
                    } else {
                        (0.0, 0.0)
                    };

                    let net_id = data.get("netID").and_then(|s| s.parse().ok()).unwrap_or(0);
                    let user_id = data.get("userID").and_then(|s| s.parse().ok()).unwrap_or(0);
                    let m_state = data
                        .get("mstate")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0u32);
                    let sm_state = data
                        .get("smstate")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0u32);
                    let invisible = data
                        .get("invis")
                        .and_then(|s| s.parse::<u32>().ok())
                        .unwrap_or(0)
                        != 0;
                    let name = data.get("name").cloned().unwrap_or_default();
                    let country = data.get("country").cloned().unwrap_or_default();

                    self.log_console(format!(
                        "[Bot] OnSpawn player={name} net_id={net_id} pos=({:.0},{:.0})",
                        position.0, position.1
                    ));

                    let player = Player {
                        net_id,
                        user_id,
                        name,
                        country,
                        position,
                        avatar: data.get("avatar").cloned().unwrap_or_default(),
                        online_id: data.get("onlineID").cloned().unwrap_or_default(),
                        e_id: data.get("eid").cloned().unwrap_or_default(),
                        ip: data.get("ip").cloned().unwrap_or_default(),
                        col_rect: data.get("colrect").cloned().unwrap_or_default(),
                        title_icon: data.get("titleIcon").cloned().unwrap_or_default(),
                        m_state,
                        sm_state,
                        invisible,
                    };

                    let is_mod = m_state > 0 || sm_state > 0 || invisible;
                    if net_id != self.local.net_id {
                        if self.auto_leave_on_mod && is_mod {
                            self.log_console(format!(
                                "[Bot] Mod detected ({}): mstate={} smstate={} invis={} — leaving",
                                player.name, m_state, sm_state, invisible
                            ));
                            self.leave_world();
                        }
                        if self.auto_ban {
                            self.say(&format!("/ban {}", player.name));
                        }
                    }

                    self.players.insert(net_id, player.clone());
                    {
                        let mut s = self.state.write().unwrap();
                        s.players = self
                            .players
                            .values()
                            .map(|p| PlayerInfo {
                                net_id: p.net_id,
                                name: p.name.clone(),
                                pos_x: p.position.0 / 32.0,
                                pos_y: p.position.1 / 32.0,
                                country: p.country.clone(),
                            })
                            .collect();
                    }
                    self.emit(WsEvent::PlayerSpawn {
                        bot_id: self.bot_id,
                        net_id: player.net_id,
                        name: player.name.clone(),
                        country: player.country.clone(),
                        x: player.position.0 / 32.0,
                        y: player.position.1 / 32.0,
                    });
                }
            }
            "OnSetPos" => {
                let (x, y) = vl.get(1).map(|v| v.as_vec2()).unwrap_or((0.0, 0.0));
                self.pos_x = x;
                self.pos_y = y;
                if self.pathfind_target.is_some() {
                    self.pathfind_recalc = true;
                }
                {
                    let mut s = self.state.write().unwrap();
                    s.pos_x = x / 32.0;
                    s.pos_y = y / 32.0;
                }
                self.log_console(format!("[Bot] OnSetPos → ({x}, {y})"));
                self.emit(WsEvent::BotMove {
                    bot_id: self.bot_id,
                    x: x / 32.0,
                    y: y / 32.0,
                });
            }
            "OnSuperMainStartAcceptLogonHrdxs47254722215a" => {
                self.state.write().unwrap().status = BotStatus::Connected;
                self.send_text("action|enter_game\n");
                self.emit(WsEvent::BotStatus {
                    bot_id: self.bot_id,
                    status: "connected".into(),
                });
            }
            "OnRemove" => {
                let message = vl.get(1).map(|v| v.as_string()).unwrap_or_default();
                let data = parse_pipe_map(&message);
                let net_id: u32 = data.get("netID").and_then(|s| s.parse().ok()).unwrap_or(0);
                self.players.remove(&net_id);
                self.state.write().unwrap().players = self
                    .players
                    .values()
                    .map(|p| PlayerInfo {
                        net_id: p.net_id,
                        name: p.name.clone(),
                        pos_x: p.position.0 / 32.0,
                        pos_y: p.position.1 / 32.0,
                        country: p.country.clone(),
                    })
                    .collect();
                self.log_console(format!("[Bot] OnRemove net_id={net_id}"));
                self.emit(WsEvent::PlayerLeave {
                    bot_id: self.bot_id,
                    net_id,
                });
            }
            "OnSetBux" => {
                let gems = vl.get(1).map(|v| v.as_int32()).unwrap_or(0);
                self.inventory.add_gems(gems);
                self.state.write().unwrap().gems = gems;
                self.emit(WsEvent::BotGems {
                    bot_id: self.bot_id,
                    gems,
                });
            }
            "OnConsoleMessage" => {
                let message = vl.get(1).map(|v| v.as_string()).unwrap_or_default();
                self.log_console(message);
            }
            "OnDialogRequest" => {
                let message = vl.get(1).map(|v| v.as_string()).unwrap_or_default();
                self.log_console(format!("[Bot] Dialog: {}", message));
                let cb = self.temporary_data.dialog_callback.lock().unwrap().take();
                if let Some(cb) = cb {
                    cb(self);
                }
            }
            "SetHasGrowID" => {
                if let Some(growid) = vl.get(2).map(|v| v.as_string()) {
                    self.username = growid.clone();
                    self.state.write().unwrap().username = growid.clone();
                    self.emit(WsEvent::BotUsername {
                        bot_id: self.bot_id,
                        username: growid,
                    });
                }
            }

            "OnRequestWorldSelectMenu" => {
                self.world = None;
                self.pathfind_target = None;
                self.pathfind_recalc = false;
                let removed = {
                    let mut s = self.state.write().unwrap();
                    s.world_name = "EXIT".to_string();
                    s.status = BotStatus::InGame;
                    self.inventory.remove_temp_items()
                };
                if removed {
                    self.emit_inventory_update();
                }
                self.emit(WsEvent::BotStatus {
                    bot_id: self.bot_id,
                    status: "in_game".into(),
                });
                self.emit(WsEvent::BotWorld {
                    bot_id: self.bot_id,
                    world_name: "EXIT".to_string(),
                });
                self.emit(WsEvent::WorldLoaded {
                    bot_id: self.bot_id,
                    name: "EXIT".to_string(),
                    width: 0,
                    height: 0,
                    tiles: vec![],
                });
                self.log_console("[Bot] OnRequestWorldSelectMenu → cleared world".to_string());
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
            self.emit(WsEvent::BotMove {
                bot_id: self.bot_id,
                x: pkt.vector_x / 32.0,
                y: pkt.vector_y / 32.0,
            });
        } else if let Some(player) = self.players.get_mut(&(pkt.net_id as u32)) {
            player.position = (pkt.vector_x, pkt.vector_y);
            let net_id = pkt.net_id as u32;
            {
                let mut s = self.state.write().unwrap();
                if let Some(pi) = s.players.iter_mut().find(|p| p.net_id == net_id) {
                    pi.pos_x = pkt.vector_x / 32.0;
                    pi.pos_y = pkt.vector_y / 32.0;
                }
            }
            self.emit(WsEvent::PlayerMove {
                bot_id: self.bot_id,
                net_id,
                x: pkt.vector_x / 32.0,
                y: pkt.vector_y / 32.0,
            });
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
            self.emit(WsEvent::TileUpdate {
                bot_id: self.bot_id,
                x,
                y,
                fg,
                bg,
            });
        }
        self.log_console(format!("[Bot] TileChange ({x},{y}) item={item_id}"));
    }

    fn on_send_tile_update_data(&mut self, pkt: &GameUpdatePacket) {
        let x = pkt.int_x as u32;
        let y = pkt.int_y as u32;

        let width = match self.world.as_ref() {
            Some(w) => w.tile_map.width,
            None => return,
        };
        let idx = (y * width + x) as usize;

        let result = self
            .world
            .as_mut()
            .unwrap()
            .update_tile_from_bytes(x, y, &pkt.extra_data);

        if let Some((fg, bg)) = result {
            {
                let mut s = self.state.write().unwrap();
                if let Some(ti) = s.tiles.get_mut(idx) {
                    ti.fg_item_id = fg;
                    ti.bg_item_id = bg;
                }
            }
            self.emit(WsEvent::TileUpdate {
                bot_id: self.bot_id,
                x,
                y,
                fg,
                bg,
            });
        }
        self.log_console(format!("[Bot] TileUpdateData ({x},{y})"));
    }

    fn on_send_tile_update_data_multiple(&mut self, pkt: &GameUpdatePacket) {
        // extra_data: u32 count, then for each: i32 x, i32 y, u16 fg, u16 bg, ...
        let data = &pkt.extra_data;
        if data.len() < 4 {
            return;
        }

        let count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let mut offset = 4;

        let width = match self.world.as_ref() {
            Some(w) => w.tile_map.width,
            None => return,
        };

        for _ in 0..count {
            // Each entry: i32 x (4), i32 y (4), u16 fg (2), u16 bg (2) = 12 bytes minimum
            if offset + 12 > data.len() {
                break;
            }

            let x = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            let y = u32::from_le_bytes([
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]);
            let tile_data = &data[offset + 8..];
            let idx = (y as u64 * width as u64 + x as u64) as usize;

            let result = self
                .world
                .as_mut()
                .unwrap()
                .update_tile_from_bytes(x, y, tile_data);

            if let Some((fg, bg)) = result {
                {
                    let mut s = self.state.write().unwrap();
                    if let Some(ti) = s.tiles.get_mut(idx) {
                        ti.fg_item_id = fg;
                        ti.bg_item_id = bg;
                    }
                }
                self.emit(WsEvent::TileUpdate {
                    bot_id: self.bot_id,
                    x,
                    y,
                    fg,
                    bg,
                });
            }

            offset += 12; // advance past the known fields; extra tile data is not parsed
        }
        self.log_console(format!("[Bot] TileUpdateDataMultiple count={count}"));
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
            self.emit(WsEvent::TileUpdate {
                bot_id: self.bot_id,
                x,
                y,
                fg: 0,
                bg,
            });
        }
        self.log_console(format!("[Bot] TileTreeState ({x},{y}) harvested"));
    }

    fn on_modify_item_inventory(&mut self, pkt: &GameUpdatePacket) {
        let item_id = pkt.value as u16;
        if pkt.jump_count != 0 {
            let amount = pkt.jump_count;
            self.inventory.sub_item(item_id, amount);
            self.log_console(format!(
                "[Bot] ModifyItemInventory item={item_id} -{amount}"
            ));
        } else {
            // animation_type != 0
            let amount = pkt.jump_count;
            self.inventory.add_item(item_id, amount);
            self.log_console(format!(
                "[Bot] ModifyItemInventory item={item_id} +{amount}"
            ));
        }
        self.emit_inventory_update();
    }

    fn emit_inventory_update(&mut self) {
        let slots: Vec<InvSlot> = self
            .inventory
            .items
            .values()
            .map(|i| InvSlot {
                item_id: i.id,
                amount: i.amount,
                is_active: i.flag & 1 != 0,
                action_type: self
                    .items_dat
                    .find_by_id(i.id as u32)
                    .map(|info| info.action_type)
                    .unwrap_or(0),
            })
            .collect();
        {
            let mut s = self.state.write().unwrap();
            s.inventory = slots;
            s.inventory_size = self.inventory.size;
        }
        let ws_items: Vec<WsInvItem> = self
            .inventory
            .items
            .values()
            .map(|i| WsInvItem {
                item_id: i.id,
                amount: i.amount,
                is_active: i.flag & 1 != 0,
                action_type: self
                    .items_dat
                    .find_by_id(i.id as u32)
                    .map(|info| info.action_type)
                    .unwrap_or(0),
            })
            .collect();
        self.emit(WsEvent::InventoryUpdate {
            bot_id: self.bot_id,
            gems: self.inventory.gems,
            inventory_size: self.inventory.size,
            items: ws_items,
        });
    }

    fn on_item_change_object(&mut self, pkt: &GameUpdatePacket) {
        if self.world.is_none() {
            return;
        }

        match pkt.net_id {
            u32::MAX => {
                // New item dropped into the world
                let world = self.world.as_mut().unwrap();
                let next_uid = world.next_object_uid;
                world.next_object_uid += 1;
                let obj = WorldObject {
                    item_id: pkt.value as u16,
                    x: pkt.vector_x.ceil(),
                    y: pkt.vector_y.ceil(),
                    count: pkt.float_variable as u8,
                    flags: pkt.object_type,
                    uid: next_uid,
                };
                // let log_msg = format!(
                //     "[Bot] ItemDrop id={} uid={} count={} pos=({:.0},{:.0}) target_net_id={} object_type={} flags={:?}",
                //     obj.item_id, obj.uid, obj.count, obj.x, obj.y,
                //     pkt.target_net_id, pkt.object_type, pkt.flags
                // );
                world.objects.push(obj);
                let ws_objs: Vec<WsObject> = world
                    .objects
                    .iter()
                    .map(|o| WsObject {
                        uid: o.uid,
                        item_id: o.item_id,
                        x: o.x,
                        y: o.y,
                        count: o.count,
                    })
                    .collect();
                self.state.write().unwrap().objects = world
                    .objects
                    .iter()
                    .map(|o| WorldObjectInfo {
                        uid: o.uid,
                        item_id: o.item_id,
                        x: o.x,
                        y: o.y,
                        count: o.count,
                    })
                    .collect();
                self.emit(WsEvent::ObjectsUpdate {
                    bot_id: self.bot_id,
                    objects: ws_objs,
                });
                // self.log_console(log_msg);
            }
            net_id if net_id == u32::MAX - 2 => {
                // Update count for an existing dropped item
                let world = self.world.as_mut().unwrap();
                if let Some(obj) = world.objects.iter_mut().find(|o| {
                    o.item_id == pkt.value as u16
                        && o.x == pkt.vector_x.ceil()
                        && o.y == pkt.vector_y.ceil()
                }) {
                    // TODO : check object count
                    obj.count = pkt.float_variable as u8;
                }
                let ws_objs: Vec<WsObject> = world
                    .objects
                    .iter()
                    .map(|o| WsObject {
                        uid: o.uid,
                        item_id: o.item_id,
                        x: o.x,
                        y: o.y,
                        count: o.count,
                    })
                    .collect();
                self.state.write().unwrap().objects = world
                    .objects
                    .iter()
                    .map(|o| WorldObjectInfo {
                        uid: o.uid,
                        item_id: o.item_id,
                        x: o.x,
                        y: o.y,
                        count: o.count,
                    })
                    .collect();
                self.emit(WsEvent::ObjectsUpdate {
                    bot_id: self.bot_id,
                    objects: ws_objs,
                });
            }
            net_id if net_id > 0 => {
                // Item collected — remove from world by uid; release borrow before updating inventory
                let collected = {
                    let world = self.world.as_mut().unwrap();
                    world
                        .objects
                        .iter()
                        .position(|o| o.uid == pkt.value)
                        .map(|idx| world.objects.remove(idx))
                };
                if let Some(item) = collected {
                    let ws_objs: Vec<WsObject> = self
                        .world
                        .as_ref()
                        .unwrap()
                        .objects
                        .iter()
                        .map(|o| WsObject {
                            uid: o.uid,
                            item_id: o.item_id,
                            x: o.x,
                            y: o.y,
                            count: o.count,
                        })
                        .collect();
                    self.state.write().unwrap().objects = self
                        .world
                        .as_ref()
                        .unwrap()
                        .objects
                        .iter()
                        .map(|o| WorldObjectInfo {
                            uid: o.uid,
                            item_id: o.item_id,
                            x: o.x,
                            y: o.y,
                            count: o.count,
                        })
                        .collect();
                    self.emit(WsEvent::ObjectsUpdate {
                        bot_id: self.bot_id,
                        objects: ws_objs,
                    });
                    if pkt.net_id == self.local.net_id {
                        let current = self.inventory.items
                            .get(&item.item_id)
                            .map(|i| i.amount)
                            .unwrap_or(0);
                        let to_add = item.count.min(200u8.saturating_sub(current));
                        self.inventory.add_item(item.item_id, to_add);
                        self.log_console(format!(
                            "[Bot] ItemCollect id={} count={}",
                            item.item_id, item.count
                        ));
                        self.emit_inventory_update();
                    }
                }
            }
            _ => {}
        }
    }

    fn on_send_lock(&mut self, pkt: &GameUpdatePacket) {
        let x = pkt.int_x as u32;
        let y = pkt.int_y as u32;
        let fg = pkt.value as u16;

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

        self.emit(WsEvent::TileUpdate {
            bot_id: self.bot_id,
            x,
            y,
            fg,
            bg,
        });
        self.log_console(format!("[Bot] SendLock tile=({x},{y}) item={fg}"));
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
        self.emit(WsEvent::BotMove {
            bot_id: self.bot_id,
            x: target_x / 32.0,
            y: target_y / 32.0,
        });

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
        self.sleep_ms(self.delays.walk_ms);
    }

    pub fn place(&mut self, offset_x: i32, offset_y: i32, item_id: u32, is_punch: bool) {
        if !is_punch && !self.inventory.has_item(item_id as u16, 1) {
            return;
        }

        let base_x = (self.pos_x / 32.0).floor() as i32;
        let base_y = (self.pos_y / 32.0).floor() as i32;
        let tile_x = base_x + offset_x;
        let tile_y = base_y + offset_y;

        if tile_x > base_x + 4 || tile_x < base_x - 4 || tile_y > base_y + 4 || tile_y < base_y - 4
        {
            return;
        }

        let mut pkt = GameUpdatePacket {
            packet_type: GamePacketType::TileChangeRequest,
            vector_x: self.pos_x,
            vector_y: self.pos_y,
            int_x: tile_x,
            int_y: tile_y,
            value: item_id,
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
        pkt.flags = flags;
        self.send_game_packet(&pkt, true);
        self.sleep_ms(self.delays.place_ms);

        if !is_punch && item_id != 18 && item_id != 32 {
            self.inventory.sub_item(item_id as u16, 1);
            self.emit_inventory_update();
        }
    }

    pub fn punch(&mut self, offset_x: i32, offset_y: i32) {
        self.place(offset_x, offset_y, 18, true);
    }

    pub fn wrench(&mut self, offset_x: i32, offset_y: i32) {
        self.place(offset_x, offset_y, 32, false);
    }

    pub fn wear(&mut self, item_id: u32) {
        let pkt = GameUpdatePacket {
            packet_type: GamePacketType::ItemActivateRequest,
            value: item_id,
            ..Default::default()
        };
        self.send_game_packet(&pkt, true);
    }

    pub fn wrench_player(&mut self, net_id: u32) {
        self.send_text(&format!("action|wrench\n|netid|{net_id}\n"));
    }

    pub fn drop_item(&mut self, item_id: u32, amount: u32) {
        self.send_text(&format!("action|drop\n|itemID|{item_id}\n"));
        *self.temporary_data.dialog_callback.lock().unwrap() =
            Some(Box::new(move |bot: &mut Bot| {
                bot.send_text(&format!(
                "action|dialog_return\ndialog_name|drop_item\nitemID|{item_id}|\ncount|{amount}\n"
            ));
                *bot.temporary_data.dialog_callback.lock().unwrap() = None;
            }));
    }

    pub fn trash_item(&mut self, item_id: u32, amount: u32) {
        self.send_text(&format!("action|trash\n|itemID|{item_id}\n"));
        *self.temporary_data.dialog_callback.lock().unwrap() =
            Some(Box::new(move |bot: &mut Bot| {
                bot.send_text(&format!(
                "action|dialog_return\ndialog_name|trash_item\nitemID|{item_id}|\ncount|{amount}\n"
            ));
                *bot.temporary_data.dialog_callback.lock().unwrap() = None;
            }));
    }

    pub fn accept_access(&mut self) {
        let net_id = self.local.net_id;
        self.wrench_player(net_id);
        *self.temporary_data.dialog_callback.lock().unwrap() = Some(Box::new(
            move |bot: &mut Bot| {
                bot.send_text(&format!(
                "action|dialog_return\ndialog_name|popup\nnetID|{net_id}|\nbuttonClicked|acceptlock\n"
            ));
                *bot.temporary_data.dialog_callback.lock().unwrap() =
                    Some(Box::new(|bot: &mut Bot| {
                        bot.send_text("action|dialog_return\ndialog_name|acceptaccess\n");
                        *bot.temporary_data.dialog_callback.lock().unwrap() = None;
                    }));
            },
        ));
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
        let inv_size = self.inventory.size;
        let inv_count = self.inventory.item_count as u32;
        if inv_count >= inv_size {
            return 0;
        }

        let pos_x = self.pos_x;
        let pos_y = self.pos_y;

        let radius_tiles = self.collect_radius_tiles.clamp(1, 5);
        let r_px = radius_tiles as f32 * 32.0;
        const MAX_PER_TICK: usize = 32; // cap packets per call

        let nearby: Vec<(u32, f32, f32, u16)> = {
            let objects = &self.world.as_ref().unwrap().objects;
            let mut v: Vec<(f32, u32, f32, f32, u16)> = objects
                .iter()
                .filter_map(|obj| {
                    if self.collect_blacklist.contains(&obj.item_id) {
                        return None;
                    }
                    if self.ignore_gems && obj.item_id == 112 {
                        return None;
                    }
                    if self.ignore_essences && matches!(obj.item_id, 5024 | 5026 | 5028 | 5030) {
                        return None;
                    }
                    let dx = pos_x - obj.x;
                    let dy = pos_y - obj.y;
                    if dx.abs() > r_px || dy.abs() > r_px {
                        return None;
                    }
                    let ring = dx.abs().max(dy.abs());
                    Some((ring, obj.uid, obj.x, obj.y, obj.item_id))
                })
                .collect();
            v.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            v.into_iter()
                .map(|(_, uid, x, y, id)| (uid, x, y, id))
                .collect()
        };

        if nearby.is_empty() {
            return 0;
        }

        {
            let world = self.world.as_ref().unwrap();
            let width = world.tile_map.width;
            let height = world.tile_map.height;
            let tiles: Vec<(u16, u8)> = world
                .tile_map
                .tiles
                .iter()
                .map(|t| {
                    let ct = match &t.tile_type {
                        TileType::Lock { .. } => 3,
                        TileType::Door { .. } => 0,
                        _ => self
                            .items_dat
                            .find_by_id(t.fg_item_id as u32)
                            .map(|i| i.collision_type)
                            .unwrap_or(if t.fg_item_id == 0 { 0 } else { 1 }),
                    };
                    (t.fg_item_id, ct)
                })
                .collect();
            let _ = world;
            self.astar.update_from_tiles(width, height, &tiles);
        }

        let from_x = (pos_x / 32.0) as u32;
        let from_y = (pos_y / 32.0) as u32;
        let has_access = self.has_access();

        let mut sent = 0;
        for (uid, x, y, item_id) in nearby.iter().take(MAX_PER_TICK) {
            let tile_x = (*x / 32.0) as u32;
            let tile_y = (*y / 32.0) as u32;
            // Skip if there's no reachable path to the object's tile
            if self.collect_path_check
                && self.astar.find_path(from_x, from_y, tile_x, tile_y, has_access).is_none()
            {
                continue;
            }

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
                    vector_x: *x,
                    vector_y: *y,
                    value: *uid,
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
        self.pathfind_target = Some((to_x, to_y));
        self.pathfind_recalc = false;

        while self.pathfind_target.is_some() {
            let (goal_x, goal_y) = match self.pathfind_target {
                Some(g) => g,
                None => break,
            };

            let world = match &self.world {
                Some(w) => w,
                None => {
                    self.pathfind_target = None;
                    return;
                }
            };

            let width = world.tile_map.width;
            let height = world.tile_map.height;

            // Build (fg_item_id, collision_type) pairs for the grid
            let tiles: Vec<(u16, u8)> = world
                .tile_map
                .tiles
                .iter()
                .map(|t| {
                    let ct = match &t.tile_type {
                        TileType::Door { .. } => 0, // doors are always passable
                        _ => self
                            .items_dat
                            .find_by_id(t.fg_item_id as u32)
                            .map(|i| i.collision_type)
                            .unwrap_or(if t.fg_item_id == 0 { 0 } else { 1 }),
                    };
                    (t.fg_item_id, ct)
                })
                .collect();

            self.astar.update_from_tiles(width, height, &tiles);

            let from_x = (self.pos_x / 32.0) as u32;
            let from_y = (self.pos_y / 32.0) as u32;
            let has_access = self.has_access();

            if from_x == goal_x && from_y == goal_y {
                self.pathfind_target = None;
                break;
            }

            let path = self.astar.find_path(from_x, from_y, goal_x, goal_y, has_access);

            let Some(nodes) = path else {
                self.pathfind_target = None;
                break;
            };

            self.pathfind_recalc = false;

            for node in nodes {
                let nx = node.x as u32;
                let ny = node.y as u32;
                let cx = (self.pos_x / 32.0) as u32;
                let cy = (self.pos_y / 32.0) as u32;
                if nx == cx && ny == cy {
                    continue;
                }

                self.walk(node.x as i32, node.y as i32);

                if self.pathfind_recalc {
                    break;
                }
            }

            if self.pathfind_recalc {
                continue;
            }

            self.pathfind_target = None;
        }
    }

    /// Process one request from the script thread and return the reply.
    fn handle_script_request(
        &mut self,
        req: crate::script_channel::ScriptRequest,
    ) -> crate::script_channel::ScriptReply {
        use crate::script_channel::{
            LocalSnapshot, ScriptReply as Rep, ScriptRequest as Req, WorldSnapshot,
        };
        match req {
            Req::Reconnect => {
                self.reconnect();
                Rep::Ack
            }
            Req::Disconnect => {
                self.disconnect();
                Rep::Ack
            }
            Req::SendRaw { pkt } => {
                self.send_game_packet(&pkt, true);
                Rep::Ack
            }
            Req::SendPacket { ptype, text } => {
                match ptype {
                    2 => self.send_text(&text),
                    3 => self.send_game_message(&text),
                    _ => {}
                }
                Rep::Ack
            }
            Req::Say { text } => {
                self.say(&text);
                Rep::Ack
            }
            Req::Warp { name, id } => {
                self.warp(&name, &id);
                Rep::Ack
            }
            Req::LeaveWorld => {
                self.leave_world();
                Rep::Ack
            }
            Req::Respawn => {
                self.respawn();
                Rep::Ack
            }
            Req::Active { tile_x, tile_y } => {
                self.active_tile(tile_x, tile_y);
                Rep::Ack
            }
            Req::Enter { pass } => {
                let cx = (self.pos_x / 32.0) as i32;
                let cy = (self.pos_y / 32.0) as i32;
                if let Some(pw) = pass {
                    self.send_text(&format!("action|input\n|text|{pw}\n"));
                } else {
                    self.active_tile(cx, cy);
                }
                Rep::Ack
            }
            Req::Place { x, y, item } => {
                self.place(x, y, item, false);
                Rep::Ack
            }
            Req::Hit { x, y } => {
                self.punch(x, y);
                Rep::Ack
            }
            Req::Wrench { x, y } => {
                self.wrench_at(x, y);
                Rep::Ack
            }
            Req::WrenchPlayer { net_id } => {
                self.wrench_player(net_id);
                Rep::Ack
            }
            Req::Wear { item_id } => {
                self.wear(item_id);
                Rep::Ack
            }
            Req::Unwear { item_id } => {
                self.unwear(item_id);
                Rep::Ack
            }
            Req::Drop { item_id, count } => {
                self.drop_item(item_id, count);
                Rep::Ack
            }
            Req::Trash { item_id, count } => {
                self.trash_item(item_id, count);
                Rep::Ack
            }
            Req::FastDrop { item_id, count } => {
                self.fast_drop(item_id, count);
                Rep::Ack
            }
            Req::FastTrash { item_id, count } => {
                self.fast_trash(item_id, count);
                Rep::Ack
            }
            Req::Walk { tile_x, tile_y } => {
                self.walk(tile_x, tile_y);
                Rep::Ack
            }
            Req::SetDirection { facing_left } => {
                self.set_direction(facing_left);
                Rep::Ack
            }
            Req::FindPath { x, y } => {
                self.find_path(x, y);
                Rep::Ack
            }
            Req::CollectObject { uid, range } => {
                self.collect_object_at(uid, range);
                Rep::Ack
            }
            Req::Collect { range, interval_ms } => {
                let count = if let Some(w) = &self.world {
                    let r2 = (range * 32.0).powi(2);
                    let uids: Vec<u32> = w
                        .objects
                        .iter()
                        .filter(|o| {
                            let dx = self.pos_x - o.x;
                            let dy = self.pos_y - o.y;
                            dx * dx + dy * dy <= r2
                        })
                        .map(|o| o.uid)
                        .collect();
                    let n = uids.len();
                    for uid in uids {
                        self.collect_object_at(uid, range);
                        self.sleep_ms(interval_ms);
                    }
                    n
                } else {
                    0
                };
                Rep::CollectCount(count)
            }
            Req::SetMac { mac } => {
                self.mac = mac.clone();
                self.state.write().unwrap().mac = mac;
                Rep::Ack
            }
            Req::SetAutoCollect { enabled } => {
                self.auto_collect = enabled;
                self.state.write().unwrap().auto_collect = enabled;
                self.emit(WsEvent::BotAutoCollect { bot_id: self.bot_id, enabled });
                Rep::Ack
            }
            Req::GetWorld => {
                let snap = self.world.clone().map(|world| {
                    let players = self.players.values().cloned().collect();
                    WorldSnapshot {
                        world,
                        players,
                        local_net_id: self.local.net_id,
                        local_user_id: self.local.user_id,
                        local_name: self.username.clone(),
                        local_pos: (self.pos_x, self.pos_y),
                    }
                });
                Rep::World(snap)
            }
            Req::GetInventory => Rep::Inventory(self.inventory.clone()),
            Req::GetLocal => Rep::Local(LocalSnapshot {
                net_id: self.local.net_id,
                user_id: self.local.user_id,
                pos_x: self.pos_x,
                pos_y: self.pos_y,
                username: self.username.clone(),
                mac: self.mac.clone(),
            }),
            Req::GetPath { x, y } => Rep::Path(self.compute_path(x, y)),
            Req::IsInWorld { name } => Rep::Bool(match (&self.world, name) {
                (Some(w), Some(n)) => w.tile_map.world_name.to_uppercase() == n.to_uppercase(),
                (Some(_), None) => true,
                (None, _) => false,
            }),
            Req::IsInTile { x, y } => {
                let cx = (self.pos_x / 32.0) as u32;
                let cy = (self.pos_y / 32.0) as u32;
                Rep::Bool(cx == x && cy == y)
            }
            Req::GetAutoCollect => Rep::Bool(self.auto_collect),
            Req::SetIgnoreGems { enabled } => {
                self.ignore_gems = enabled;
                self.state.write().unwrap().ignore_gems = enabled;
                Rep::Ack
            }
            Req::GetIgnoreGems => Rep::Bool(self.ignore_gems),
            Req::SetIgnoreEssences { enabled } => {
                self.ignore_essences = enabled;
                self.state.write().unwrap().ignore_essences = enabled;
                Rep::Ack
            }
            Req::GetIgnoreEssences => Rep::Bool(self.ignore_essences),
            Req::SetAutoLeaveOnMod { enabled } => {
                self.auto_leave_on_mod = enabled;
                self.state.write().unwrap().auto_leave_on_mod = enabled;
                Rep::Ack
            }
            Req::GetAutoLeaveOnMod => Rep::Bool(self.auto_leave_on_mod),
            Req::SetAutoBan { enabled } => {
                self.auto_ban = enabled;
                self.state.write().unwrap().auto_ban = enabled;
                Rep::Ack
            }
            Req::GetAutoBan => Rep::Bool(self.auto_ban),
            Req::SetCollectInterval { ms } => {
                self.collect_interval = ms;
                Rep::Ack
            }
            Req::GetCollectInterval => Rep::U32(self.collect_interval as u32),
            Req::SetCollectRange { tiles } => {
                self.collect_radius_tiles = tiles.clamp(1, 5);
                Rep::Ack
            }
            Req::GetCollectRange => Rep::U32(self.collect_radius_tiles as u32),
            Req::SetCollectPathCheck { enabled } => {
                self.collect_path_check = enabled;
                self.state.write().unwrap().collect_path_check = enabled;
                Rep::Ack
            }
            Req::GetCollectPathCheck => Rep::Bool(self.collect_path_check),
            Req::SetReconnectInterval { ms } => {
                self.reconnect_interval = ms;
                Rep::Ack
            }
            Req::GetReconnectInterval => Rep::U32(self.reconnect_interval as u32),
            Req::GetPing => Rep::U32(self.state.read().unwrap().ping_ms),
            Req::GetGems => Rep::I32(self.inventory.gems),
            Req::SetPlaceDelay { ms } => {
                self.delays.place_ms = ms;
                self.state.write().unwrap().delays.place_ms = ms;
                self.emit(WsEvent::BotDelays {
                    bot_id: self.bot_id,
                    place_ms: self.delays.place_ms,
                    walk_ms: self.delays.walk_ms,
                    twofa_secs: self.delays.twofa_secs,
                    server_overload_secs: self.delays.server_overload_secs,
                    too_many_logins_secs: self.delays.too_many_logins_secs,
                    maintenance_secs: self.delays.maintenance_secs,
                });
                Rep::Ack
            }
            Req::SetWalkDelay { ms } => {
                self.delays.walk_ms = ms;
                self.state.write().unwrap().delays.walk_ms = ms;
                self.emit(WsEvent::BotDelays {
                    bot_id: self.bot_id,
                    place_ms: self.delays.place_ms,
                    walk_ms: self.delays.walk_ms,
                    twofa_secs: self.delays.twofa_secs,
                    server_overload_secs: self.delays.server_overload_secs,
                    too_many_logins_secs: self.delays.too_many_logins_secs,
                    maintenance_secs: self.delays.maintenance_secs,
                });
                Rep::Ack
            }
            Req::GetPlaceDelay => Rep::U32(self.delays.place_ms as u32),
            Req::GetWalkDelay => Rep::U32(self.delays.walk_ms as u32),
        }
    }

    /// Drain all pending requests from the script thread, handling each one.
    /// Detects when the script thread exits (channel closed) and clears channel fields.
    fn drain_script_requests(&mut self) {
        loop {
            let req = match &self.script_req_rx {
                Some(rx) => match rx.try_recv() {
                    Ok(r) => r,
                    Err(crossbeam_channel::TryRecvError::Empty) => break,
                    Err(crossbeam_channel::TryRecvError::Disconnected) => {
                        self.script_req_rx = None;
                        self.script_reply_tx = None;
                        self.event_tx = None;
                        break;
                    }
                },
                None => break,
            };
            let reply = self.handle_script_request(req);
            if let Some(tx) = &self.script_reply_tx {
                tx.send(reply).ok();
            }
        }
    }

    fn handle_command(&mut self, cmd: BotCommand) {
        match cmd {
            BotCommand::Move { x, y } => {
                let cx = (self.pos_x / 32.0) as i32;
                let cy = (self.pos_y / 32.0) as i32;
                self.walk(cx + x, cy + y);
            }
            BotCommand::WalkTo { x, y } => {
                self.find_path(x, y);
            }
            BotCommand::RunScript { content } => {
                // Stop any currently running script first.
                self.script_stop.store(true, Ordering::Relaxed);
                // Drop old channels so the previous script thread (if any) sees disconnection.
                self.script_req_rx = None;
                self.script_reply_tx = None;
                self.event_tx = None;

                self.script_stop.store(false, Ordering::Relaxed);

                let (req_tx, req_rx) =
                    crossbeam_channel::unbounded::<crate::script_channel::ScriptRequest>();
                let (reply_tx, reply_rx) =
                    crossbeam_channel::unbounded::<crate::script_channel::ScriptReply>();
                let (event_tx, event_rx) = crossbeam_channel::bounded::<BotEventRaw>(256);

                self.script_req_rx = Some(req_rx);
                self.script_reply_tx = Some(reply_tx);
                self.event_tx = Some(event_tx);

                let items = self.items_dat.clone();
                let state = self.state.clone();
                let stop_flag = self.script_stop.clone();
                let username = self.username.clone();

                std::thread::spawn(move || {
                    crate::lua::run_script_threaded(
                        req_tx, reply_rx, event_rx, items, state, stop_flag, username, content,
                    );
                });
            }
            BotCommand::StopScript => {
                self.script_stop.store(true, Ordering::Relaxed);
            }
            BotCommand::Say { text } => {
                self.say(&text);
            }
            BotCommand::Warp { name, id } => {
                self.warp(&name, &id);
            }
            BotCommand::Disconnect => {
                self.disconnect();
            }
            BotCommand::Reconnect => {
                if self.peer_id.is_some() {
                    self.disconnect();
                } else {
                    self.reconnect_main();
                }
            }
            BotCommand::Place { x, y, item } => {
                self.place(x, y, item, false);
            }
            BotCommand::Hit { x, y } => {
                self.punch(x, y);
            }
            BotCommand::Wrench { x, y } => {
                self.wrench_at(x, y);
            }
            BotCommand::Wear { item_id } => {
                self.wear(item_id);
            }
            BotCommand::Unwear { item_id } => {
                self.unwear(item_id);
            }
            BotCommand::Drop { item_id, count } => {
                self.drop_item(item_id, count);
            }
            BotCommand::Trash { item_id, count } => {
                self.trash_item(item_id, count);
            }
            BotCommand::LeaveWorld => {
                self.leave_world();
            }
            BotCommand::Respawn => {
                self.respawn();
            }
            BotCommand::FindPath { x, y } => {
                self.find_path(x, y);
            }
            BotCommand::SetDelays(d) => {
                self.delays = d.clone();
                self.state.write().unwrap().delays = d.clone();
                self.emit(WsEvent::BotDelays {
                    bot_id: self.bot_id,
                    place_ms: d.place_ms,
                    walk_ms: d.walk_ms,
                    twofa_secs: d.twofa_secs,
                    server_overload_secs: d.server_overload_secs,
                    too_many_logins_secs: d.too_many_logins_secs,
                    maintenance_secs: d.maintenance_secs,
                });
            }
            BotCommand::SetAutoCollect { enabled } => {
                self.auto_collect = enabled;
                self.state.write().unwrap().auto_collect = enabled;
                self.emit(WsEvent::BotAutoCollect { bot_id: self.bot_id, enabled });
            }
            BotCommand::SetAutoReconnect { enabled } => {
                self.auto_reconnect = enabled;
                self.state.write().unwrap().auto_reconnect = enabled;
            }
            BotCommand::SetCollectConfig {
                radius_tiles,
                blacklist,
            } => {
                self.collect_radius_tiles = radius_tiles.clamp(1, 5);
                self.collect_blacklist = blacklist.into_iter().collect();
                let mut st = self.state.write().unwrap();
                st.collect_radius_tiles = self.collect_radius_tiles;
                st.collect_blacklist = sorted_blacklist_vec(&self.collect_blacklist);
            }
            BotCommand::AcceptAccess => self.accept_access(),
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
        self.send_game_message(&format!(
            "action|join_request\nname|{name}|{id}\ninvitedWorld|0\n"
        ));
    }

    pub fn leave_world(&mut self) {
        self.send_game_message("action|quit_to_exit\n");
    }

    pub fn respawn(&mut self) {
        self.send_text("action|respawn\n");
    }

    pub fn unwear(&mut self, item_id: u32) {
        let pkt = GameUpdatePacket {
            packet_type: GamePacketType::ItemActivateRequest,
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
        let width = world.tile_map.width;
        let height = world.tile_map.height;

        let tiles: Vec<(u16, u8)> = world
            .tile_map
            .tiles
            .iter()
            .map(|t| {
                let ct = match &t.tile_type {
                    TileType::Lock { .. } => 3,
                    TileType::Door { .. } => 0,
                    _ => self
                        .items_dat
                        .find_by_id(t.fg_item_id as u32)
                        .map(|i| i.collision_type)
                        .unwrap_or(if t.fg_item_id == 0 { 0 } else { 1 }),
                };
                (t.fg_item_id, ct)
            })
            .collect();

        let _ = world; // end the borrow before mutating astar

        self.astar.update_from_tiles(width, height, &tiles);

        let from_x = (self.pos_x / 32.0) as u32;
        let from_y = (self.pos_y / 32.0) as u32;
        let has_access = self.has_access();

        self.astar
            .find_path(from_x, from_y, to_x, to_y, has_access)
            .unwrap_or_default()
            .into_iter()
            .map(|n| (n.x, n.y))
            .collect()
    }

    // ── NPC packet ────────────────────────────────────────────────────────────

    fn on_npc_packet(&mut self, pkt: &GameUpdatePacket) {
        let world = match self.world.as_mut() {
            Some(w) => w,
            None    => return,
        };

        if !pkt.extra_data.is_empty() {
            let data = &pkt.extra_data;
            if data.is_empty() { return; }
            let count = data[0] as usize;
            let entry_size = 2 + 7 * 4; // 30 bytes
            for i in 0..count {
                let off = 1 + i * entry_size;
                if off + entry_size > data.len() { break; }
                let id       = data[off + 1];
                let read_f32 = |o: usize| f32::from_le_bytes(data[o..o+4].try_into().unwrap_or([0;4]));
                world.set_npc(WorldNpc {
                    npc_type: NpcType::from_u8(data[off]),
                    id,
                    x:      read_f32(off + 2),
                    y:      read_f32(off + 6),
                    dest_x: read_f32(off + 10),
                    dest_y: read_f32(off + 14),
                    unk1:   read_f32(off + 18),
                    unk2:   read_f32(off + 22),
                    var:    read_f32(off + 26),
                });
            }
            return;
        }

        // animation_type = NpcAction, object_type = NpcType, jump_count = NpcIndex
        let id = pkt.jump_count;

        match NpcAction::from_u8(pkt.animation_type) {
            Some(NpcAction::Add | NpcAction::MoveTo) => {
                world.set_npc(WorldNpc {
                    npc_type: NpcType::from_u8(pkt.object_type),
                    id,
                    x:        pkt.vector_x,
                    y:        pkt.vector_y,
                    dest_x:   pkt.vector_x2,
                    dest_y:   pkt.vector_y2,
                    unk1:     0.0,
                    unk2:     0.0,
                    var:      pkt.float_variable,
                });
            }
            Some(NpcAction::Delete | NpcAction::Sucked | NpcAction::Die) => {
                world.remove_npc(id);
            }
            _ => {}
        }
    }
}

