use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct WsObject {
    pub uid:     u32,
    pub item_id: u16,
    pub x:       f32,
    pub y:       f32,
    pub count:   u8,
}

#[derive(Serialize, Clone, Debug)]
pub struct WsInvItem {
    pub item_id:     u16,
    pub amount:      u8,
    pub is_active:   bool,
    pub action_type: u8,
}

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "event", content = "data")]
pub enum WsEvent {
    /// A new bot was spawned.
    BotAdded { bot_id: u32, username: String },
    /// A bot was stopped/removed.
    BotRemoved { bot_id: u32 },
    /// Bot connection status changed.
    BotStatus { bot_id: u32, status: String },
    /// Bot entered or left a world (world_name is empty when leaving).
    BotWorld { bot_id: u32, world_name: String },
    /// Bot position updated (tile coordinates).
    BotMove { bot_id: u32, x: f32, y: f32 },
    /// Bot gems balance updated.
    BotGems { bot_id: u32, gems: i32 },
    /// Bot ping updated (emitted when value changes).
    BotPing { bot_id: u32, ping_ms: u32 },
    /// A player appeared in the bot's world.
    PlayerSpawn { bot_id: u32, net_id: u32, name: String, country: String, x: f32, y: f32 },
    /// A player moved.
    PlayerMove { bot_id: u32, net_id: u32, x: f32, y: f32 },
    /// A player left the bot's world.
    PlayerLeave { bot_id: u32, net_id: u32 },
    /// Full world data (sent once on world load).
    WorldLoaded { bot_id: u32, name: String, width: u32, height: u32, tiles: Vec<[u16; 2]> },
    /// A single tile was updated.
    TileUpdate { bot_id: u32, x: u32, y: u32, fg: u16, bg: u16 },
    /// The full set of dropped world objects changed.
    ObjectsUpdate { bot_id: u32, objects: Vec<WsObject> },
    /// Bot inventory updated.
    InventoryUpdate { bot_id: u32, gems: i32, items: Vec<WsInvItem> },
    /// A new console message arrived.
    Console { bot_id: u32, message: String },
    /// Account info extracted from the Track packet (sent once per login).
    BotTrackInfo { bot_id: u32, level: u32, grow_id: u64, install_date: u64, global_playtime: u64, awesomeness: u32 },
}

pub type WsTx = tokio::sync::broadcast::Sender<WsEvent>;
