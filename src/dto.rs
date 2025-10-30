use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Request DTOs
#[derive(Debug, Deserialize)]
pub struct CreateBotRequest {
    pub login_method: String, // "google", "apple", "ltoken", "legacy"
    pub credentials: Option<BotCredentials>,
    pub socks5: Option<String>, // Format: "host:port" or "host:port:username:password"
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BotCredentials {
    LTOKEN { token: String }, // 4 values separated by ":"
    Legacy { growid: String, password: String },
}

#[derive(Debug, Deserialize)]
pub struct WarpRequest {
    pub world_name: String,
}

#[derive(Debug, Deserialize)]
pub struct SayRequest {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct WalkRequest {
    pub x: i32,
    pub y: i32,
    pub autopilot: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct MoveRequest {
    pub direction: String, // "left", "right", "up", "down"
    pub tiles: Option<i32>, // Optional: number of tiles to move (default: 1)
}

#[derive(Debug, Deserialize)]
pub struct PunchRequest {
    pub offset_x: i32,
    pub offset_y: i32,
}

#[derive(Debug, Deserialize)]
pub struct PlaceRequest {
    pub offset_x: i32,
    pub offset_y: i32,
    pub item_id: u32,
}

#[derive(Debug, Deserialize)]
pub struct ConfigUpdate {
    pub auto_collect: Option<bool>,
    pub auto_reconnect: Option<bool>,
    pub findpath_delay: Option<u32>,
    pub punch_delay: Option<u32>,
    pub place_delay: Option<u32>,
}

// Response DTOs
#[derive(Debug, Serialize)]
pub struct BotListResponse {
    pub bots: Vec<BotSummary>,
}

#[derive(Debug, Serialize)]
pub struct BotSummary {
    pub id: Uuid,
    pub name: String,
    pub login_method: String,
    pub status: String,
    pub gems: i32,
    pub ping: u32,
    pub world: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BotDetailResponse {
    pub id: Uuid,
    pub name: String,
    pub login_method: String,
    pub status: String,
    pub gems: i32,
    pub ping: u32,
    pub position: (f32, f32),
    pub world: Option<WorldInfo>,
    pub config: BotConfig,
}

#[derive(Debug, Serialize)]
pub struct WorldInfo {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub player_count: usize,
}

#[derive(Debug, Serialize)]
pub struct BotConfig {
    pub auto_collect: bool,
    pub auto_reconnect: bool,
    pub findpath_delay: u32,
    pub punch_delay: u32,
    pub place_delay: u32,
}

#[derive(Debug, Serialize)]
pub struct InventoryResponse {
    pub size: u32,
    pub item_count: u16,
    pub items: Vec<InventoryItemDto>,
}

#[derive(Debug, Serialize)]
pub struct InventoryItemDto {
    pub id: u16,
    pub name: String,
    pub amount: u8,
    pub flag: u8,
}

#[derive(Debug, Serialize)]
pub struct WorldResponse {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub players: Vec<PlayerDto>,
    pub tile_counts: TileCounts,
}

#[derive(Debug, Serialize)]
pub struct PlayerDto {
    pub net_id: u32,
    pub name: String,
    pub position: (f32, f32),
    pub country: String,
}

#[derive(Debug, Serialize)]
pub struct TileCounts {
    pub foreground: HashMap<u16, u32>,
    pub background: HashMap<u16, u32>,
}

#[derive(Debug, Serialize)]
pub struct LogsResponse {
    pub logs: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}
