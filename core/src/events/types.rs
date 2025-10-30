use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotEvent {
    pub timestamp: u64,
    #[serde(flatten)]
    pub event_type: EventType,
}

impl BotEvent {
    pub fn new(event_type: EventType) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            timestamp,
            event_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum EventType {
    // World Events
    WorldLoaded {
        name: String,
        width: u32,
        height: u32,
    },
    TileChanged {
        x: u32,
        y: u32,
        foreground_id: u16,
        background_id: u16,
    },
    ItemDropped {
        uid: u32,
        item_id: u16,
        x: f32,
        y: f32,
        count: u8,
    },
    ItemCollected {
        uid: u32,
        item_id: u16,
        count: u8,
    },

    // Inventory Events
    InventoryChanged {
        item_id: u16,
        new_amount: u8,
        delta: i16,
    },
    GemsChanged {
        new_amount: i32,
        delta: i32,
    },
    InventoryParsed {
        size: u32,
        item_count: u16,
    },

    // Movement Events
    PositionChanged {
        x: f32,
        y: f32,
    },
    PathfindingStarted {
        target_x: u32,
        target_y: u32,
    },
    PathfindingCompleted {
        success: bool,
        steps: usize,
    },

    // Player Events
    PlayerJoined {
        net_id: u32,
        name: String,
        country: String,
    },
    PlayerLeft {
        net_id: u32,
    },
    PlayerMoved {
        net_id: u32,
        x: f32,
        y: f32,
    },

    // Network Events
    Connected {
        server: String,
        port: u16,
    },
    Disconnected {
        reason: Option<String>,
    },
    PacketReceived {
        packet_type: String,
    },
    PacketSent {
        packet_type: String,
    },

    // System Events
    Log {
        level: LogLevel,
        message: String,
    },
    Error {
        message: String,
    },
    ConfigChanged {
        key: String,
        value: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}
