use crate::inventory::Inventory;
use crate::protocol::packet::GameUpdatePacket;
use crate::player::Player;
use crate::world::World;
use std::sync::Arc;

/// A snapshot of the bot's world state, cloned and sent to the script thread.
pub struct WorldSnapshot {
    pub world:         Arc<World>,
    pub players:       Arc<Vec<Player>>,
    pub local_net_id:  u32,
    pub local_user_id: u32,
    pub local_name:    String,
    pub local_pos:     (f32, f32),
}

/// A snapshot of the bot's local identity, cloned and sent to the script thread.
pub struct LocalSnapshot {
    pub net_id:   u32,
    pub user_id:  u32,
    pub pos_x:    f32,
    pub pos_y:    f32,
    pub username: String,
    pub mac:      String,
}

/// Commands the script thread sends to the bot thread.
pub enum ScriptRequest {
    // Network / connection
    Reconnect,
    Disconnect,
    SendRaw { pkt: GameUpdatePacket },
    SendPacket { ptype: u8, text: String },

    // World actions
    Say { text: String },
    Warp { name: String, id: String },
    LeaveWorld,
    Respawn,
    Active { tile_x: i32, tile_y: i32 },
    Enter { pass: Option<String> },

    // Tile actions
    Place { x: i32, y: i32, item: u32 },
    Hit { x: i32, y: i32 },
    Wrench { x: i32, y: i32 },
    WrenchPlayer { net_id: u32 },

    // Inventory
    Wear { item_id: u32 },
    Unwear { item_id: u32 },
    Drop { item_id: u32, count: u32 },
    Trash { item_id: u32, count: u32 },
    FastDrop { item_id: u32, count: u32 },
    FastTrash { item_id: u32, count: u32 },

    // Movement
    Walk { tile_x: i32, tile_y: i32 },
    SetDirection { facing_left: bool },
    FindPath { x: u32, y: u32 },

    // Object collection
    CollectObject { uid: u32, range: f32 },
    Collect { range: f32, interval_ms: u64 },

    // State mutation
    SetMac { mac: String },
    SetAutoCollect { enabled: bool },
    SetIgnoreGems { enabled: bool },
    GetIgnoreGems,
    SetIgnoreEssences { enabled: bool },
    GetIgnoreEssences,
    SetAutoLeaveOnMod { enabled: bool },
    GetAutoLeaveOnMod,
    SetAutoBan { enabled: bool },
    GetAutoBan,
    SetCollectInterval { ms: u64 },
    GetCollectInterval,
    SetCollectRange { tiles: u8 },
    GetCollectRange,
    SetCollectPathCheck { enabled: bool },
    GetCollectPathCheck,
    SetReconnectInterval { ms: u64 },
    GetReconnectInterval,
    SetPlaceDelay { ms: u64 },
    SetWalkDelay { ms: u64 },

    // Queries — bot replies with data
    GetWorld,
    GetInventory,
    GetLocal,
    GetPath { x: u32, y: u32 },
    IsInWorld { name: Option<String> },
    IsInTile { x: u32, y: u32 },
    GetAutoCollect,
    GetPing,
    GetGems,
    GetPlaceDelay,
    GetWalkDelay,


}

/// Replies sent back from the bot thread to the script thread.
pub enum ScriptReply {
    Ack,
    Bool(bool),
    U32(u32),
    I32(i32),
    World(Option<WorldSnapshot>),
    Inventory(Inventory),
    Local(LocalSnapshot),
    Path(Vec<(u32, u32)>),
    CollectCount(usize),
}
