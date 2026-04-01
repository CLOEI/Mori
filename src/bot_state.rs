use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::mpsc;
use crate::world::TileType;

#[derive(Default, Clone, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BotStatus {
    #[default]
    Connecting,
    Connected,
    InGame,
    /// Blocked by 2FA (Advanced Account Protection). Retrying after 120 s.
    TwoFactorAuth,
    /// Server overloaded. Retrying after 30 s.
    ServerOverloaded,
    /// Too many logins at once. Retrying after 5 s.
    TooManyLogins,
    /// Client is outdated — server requires an update. Bot stopped.
    UpdateRequired,
}

impl fmt::Display for BotStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BotStatus::Connecting    => write!(f, "connecting"),
            BotStatus::Connected     => write!(f, "connected"),
            BotStatus::InGame        => write!(f, "in_game"),
            BotStatus::TwoFactorAuth    => write!(f, "two_factor_auth"),
            BotStatus::ServerOverloaded => write!(f, "server_overloaded"),
            BotStatus::TooManyLogins    => write!(f, "too_many_logins"),
            BotStatus::UpdateRequired   => write!(f, "update_required"),
        }
    }
}

#[derive(Clone, Serialize)]
pub struct TileInfo {
    pub fg_item_id: u16,
    pub bg_item_id: u16,
    pub flags:      u16,
    pub tile_type:  TileType,
}

impl Default for TileInfo {
    fn default() -> Self {
        Self { fg_item_id: 0, bg_item_id: 0, flags: 0, tile_type: TileType::Basic }
    }
}

#[derive(Clone, Serialize)]
pub struct PlayerInfo {
    pub net_id:  u32,
    pub name:    String,
    pub pos_x:   f32,
    pub pos_y:   f32,
    pub country: String,
}

#[derive(Clone, Serialize)]
pub struct InvSlot {
    pub item_id:     u16,
    pub amount:      u8,
    pub is_active:   bool,
    pub action_type: u8,
}

#[derive(Clone, Serialize)]
pub struct WorldObjectInfo {
    pub uid:     u32,
    pub item_id: u16,
    pub x:       f32,
    pub y:       f32,
    pub count:   u8,
}

#[derive(Default, Clone, Serialize)]
pub struct TrackInfo {
    pub level:           u32,
    pub grow_id:         u64,
    pub install_date:    u64,
    pub global_playtime: u64,
    pub awesomeness:     u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BotDelays {
    pub place_ms:             u64,
    pub walk_ms:              u64,
    pub twofa_secs:           u64,
    pub server_overload_secs: u64,
    pub too_many_logins_secs: u64,
}

impl Default for BotDelays {
    fn default() -> Self {
        Self {
            place_ms:             500,
            walk_ms:              500,
            twofa_secs:           120,
            server_overload_secs: 30,
            too_many_logins_secs: 5,
        }
    }
}

#[derive(Clone, Serialize)]
pub struct BotState {
    pub status:          BotStatus,
    pub username:        String,
    pub mac:             String,
    pub world_name:      String,
    /// Tile-coordinate position (pixels ÷ 32).
    pub pos_x:           f32,
    pub pos_y:           f32,
    pub world_width:     u32,
    pub world_height:    u32,
    pub tiles:           Vec<TileInfo>,
    pub objects:         Vec<WorldObjectInfo>,
    pub players:         Vec<PlayerInfo>,
    pub inventory:       Vec<InvSlot>,
    /// Maximum number of inventory slots the bot has (from SendInventoryState).
    pub inventory_size:  u32,
    pub gems:            i32,
    pub console:         Vec<String>,
    /// Round-trip time in milliseconds from ENet, updated every run loop tick.
    pub ping_ms: u32,
    /// Configurable delays for bot actions.
    pub delays: BotDelays,
    pub track_info: Option<TrackInfo>,
    /// Whether the run loop should auto-collect nearby dropped items.
    pub auto_collect: bool,
    /// Auto-collect half-extent in tiles (1–5): axis-aligned square |Δx|,|Δy| ≤ tiles×32 px.
    pub collect_radius_tiles: u8,
    /// Item IDs to skip when auto-collecting (sorted, unique in API responses).
    pub collect_blacklist: Vec<u16>,
    /// Whether the bot should automatically reconnect after a disconnect.
    pub auto_reconnect: bool,
}

impl Default for BotState {
    fn default() -> Self {
        Self {
            status: BotStatus::default(),
            username: String::new(),
            mac: String::new(),
            world_name: String::new(),
            pos_x: 0.0,
            pos_y: 0.0,
            world_width: 0,
            world_height: 0,
            tiles: Vec::new(),
            objects: Vec::new(),
            players: Vec::new(),
            inventory: Vec::new(),
            inventory_size: 0,
            gems: 0,
            console: Vec::new(),
            ping_ms: 0,
            delays: BotDelays::default(),
            track_info: None,
            auto_collect: true,
            collect_radius_tiles: 3,
            collect_blacklist: Vec::new(),
            auto_reconnect: true,
        }
    }
}

pub enum BotCommand {
    Move { x: i32, y: i32 },
    WalkTo { x: u32, y: u32 },
    RunScript { content: String },
    StopScript,
    Say { text: String },
    Warp { name: String, id: String },
    Disconnect,
    Place { x: i32, y: i32, item: u32 },
    Hit { x: i32, y: i32 },
    Wrench { x: i32, y: i32 },
    Wear { item_id: u32 },
    Unwear { item_id: u32 },
    Drop { item_id: u32, count: u32 },
    Trash { item_id: u32, count: u32 },
    LeaveWorld,
    Respawn,
    FindPath { x: u32, y: u32 },
    SetDelays(BotDelays),
    SetAutoCollect { enabled: bool },
    SetCollectConfig {
        radius_tiles: u8,
        blacklist: Vec<u16>,
    },
    SetAutoReconnect { enabled: bool },
    AcceptAccess,
}

pub type CmdSender   = mpsc::Sender<BotCommand>;
pub type CmdReceiver = mpsc::Receiver<BotCommand>;
