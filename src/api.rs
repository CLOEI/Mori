use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use gt_core::types::bot::LoginVia;
use gt_core::Socks5Config;
use std::net::ToSocketAddrs;
use std::sync::{atomic::Ordering, Arc};
use uuid::Uuid;

use crate::bot_manager::BotManager;
use crate::dto::*;

// Bot Management Endpoints

pub async fn create_bot(
    State(manager): State<Arc<BotManager>>,
    Json(request): Json<CreateBotRequest>,
) -> Result<Json<ApiResponse<BotSummary>>, StatusCode> {
    // Parse login method
    let login_via = match request.login_method.to_lowercase().as_str() {
        "google" => LoginVia::GOOGLE,
        "apple" => LoginVia::APPLE,
        "ltoken" => {
            if let Some(BotCredentials::LTOKEN { token }) = request.credentials {
                let parts: Vec<String> = token.split(':').map(|s| s.to_string()).collect();
                if parts.len() != 4 {
                    return Ok(Json(ApiResponse::error(
                        "LTOKEN must have 4 values separated by ':'".to_string(),
                    )));
                }
                LoginVia::LTOKEN([
                    parts[0].clone(),
                    parts[1].clone(),
                    parts[2].clone(),
                    parts[3].clone(),
                ])
            } else {
                return Ok(Json(ApiResponse::error(
                    "LTOKEN credentials required".to_string(),
                )));
            }
        }
        "legacy" => {
            if let Some(BotCredentials::Legacy { growid, password }) = request.credentials {
                LoginVia::LEGACY([growid, password])
            } else {
                return Ok(Json(ApiResponse::error(
                    "Legacy credentials (growid, password) required".to_string(),
                )));
            }
        }
        _ => {
            return Ok(Json(ApiResponse::error(format!(
                "Invalid login method: {}",
                request.login_method
            ))))
        }
    };

    // Parse SOCKS5 config if provided
    let socks5_config = if let Some(proxy_str) = request.socks5 {
        parse_socks5_config(&proxy_str)?
    } else {
        None
    };

    // Token fetcher for Google/Apple
    // Note: For now, Google/Apple login requires external token fetching
    // You can implement a token fetcher function here or use None
    let token_fetcher = None;

    // Create bot
    let bot_id = manager
        .create_bot(login_via, token_fetcher, socks5_config)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get bot summary
    if let Some(bot) = manager.get_bot(&bot_id) {
        let name = if let Ok(login_info_lock) = bot.info.login_info.try_lock() {
            login_info_lock
                .as_ref()
                .map(|info| info.tank_id_name.clone())
                .unwrap_or_else(|| "Unknown".to_string())
        } else {
            "Unknown".to_string()
        };

        let summary = BotSummary {
            id: bot_id,
            name,
            login_method: request.login_method,
            status: "connecting".to_string(),
            gems: bot.inventory.gems(),
            ping: bot.ping.load(Ordering::Relaxed),
            world: None,
        };

        Ok(Json(ApiResponse::success(summary)))
    } else {
        Ok(Json(ApiResponse::error("Failed to create bot".to_string())))
    }
}

pub async fn list_bots(
    State(manager): State<Arc<BotManager>>,
) -> Json<ApiResponse<BotListResponse>> {
    let bots = manager.list_bots();

    let summaries: Vec<BotSummary> = bots
        .iter()
        .map(|(id, login_method, bot)| {
            let name = if let Ok(login_info_lock) = bot.info.login_info.try_lock() {
                login_info_lock
                    .as_ref()
                    .map(|info| info.tank_id_name.clone())
                    .unwrap_or_else(|| "Unknown".to_string())
            } else {
                "Unknown".to_string()
            };

            let world = if let Ok(world_lock) = bot.world.data.try_lock() {
                Some(world_lock.name.clone())
            } else {
                None
            };

            BotSummary {
                id: *id,
                name,
                login_method: login_method.clone(),
                status: "connected".to_string(),
                gems: bot.inventory.gems(),
                ping: bot.ping.load(Ordering::Relaxed),
                world,
            }
        })
        .collect();

    Json(ApiResponse::success(BotListResponse { bots: summaries }))
}

pub async fn get_bot(
    State(manager): State<Arc<BotManager>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<BotDetailResponse>>, StatusCode> {
    let bot_entry = manager
        .get_bot_entry(&id)
        .ok_or(StatusCode::NOT_FOUND)?;

    let bot = bot_entry.bot;

    let name = if let Ok(login_info_lock) = bot.info.login_info.try_lock() {
        login_info_lock
            .as_ref()
            .map(|info| info.tank_id_name.clone())
            .unwrap_or_else(|| "Unknown".to_string())
    } else {
        "Unknown".to_string()
    };

    let position = *bot.position.read().unwrap();

    let world = if let Ok(world_lock) = bot.world.data.try_lock() {
        let player_count = bot.world.players.try_lock().map(|p| p.len()).unwrap_or(0);
        Some(WorldInfo {
            name: world_lock.name.clone(),
            width: world_lock.width,
            height: world_lock.height,
            player_count,
        })
    } else {
        None
    };

    let (automation, delay_config) = bot.config.get_all();
    let config = BotConfig {
        auto_collect: automation.auto_collect,
        auto_reconnect: automation.auto_reconnect,
        findpath_delay: delay_config.findpath_delay,
        punch_delay: delay_config.punch_delay,
        place_delay: delay_config.place_delay,
    };

    let detail = BotDetailResponse {
        id,
        name,
        login_method: bot_entry.login_method,
        status: "connected".to_string(),
        gems: bot.inventory.gems(),
        ping: bot.ping.load(Ordering::Relaxed),
        position,
        world,
        config,
    };

    Ok(Json(ApiResponse::success(detail)))
}

pub async fn remove_bot(
    State(manager): State<Arc<BotManager>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<MessageResponse>>, StatusCode> {
    manager
        .remove_bot(&id)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(ApiResponse::success(MessageResponse {
        message: "Bot removed successfully".to_string(),
    })))
}

// Bot Action Endpoints

pub async fn connect_bot(
    State(manager): State<Arc<BotManager>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<MessageResponse>>, StatusCode> {
    let bot = manager.get_bot(&id).ok_or(StatusCode::NOT_FOUND)?;

    bot.connect_to_server();

    Ok(Json(ApiResponse::success(MessageResponse {
        message: "Bot connecting...".to_string(),
    })))
}

pub async fn disconnect_bot(
    State(manager): State<Arc<BotManager>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<MessageResponse>>, StatusCode> {
    let _bot = manager.get_bot(&id).ok_or(StatusCode::NOT_FOUND)?;

    // TODO: Disconnect bot

    Ok(Json(ApiResponse::success(MessageResponse {
        message: "Bot disconnect requested (not implemented)".to_string(),
    })))
}

pub async fn warp_bot(
    State(manager): State<Arc<BotManager>>,
    Path(id): Path<Uuid>,
    Json(request): Json<WarpRequest>,
) -> Result<Json<ApiResponse<MessageResponse>>, StatusCode> {
    let bot = manager.get_bot(&id).ok_or(StatusCode::NOT_FOUND)?;

    bot.warp(request.world_name.clone());

    Ok(Json(ApiResponse::success(MessageResponse {
        message: format!("Warping to {}", request.world_name),
    })))
}

pub async fn say(
    State(manager): State<Arc<BotManager>>,
    Path(id): Path<Uuid>,
    Json(request): Json<SayRequest>,
) -> Result<Json<ApiResponse<MessageResponse>>, StatusCode> {
    let bot = manager.get_bot(&id).ok_or(StatusCode::NOT_FOUND)?;

    bot.say(request.message.clone());

    Ok(Json(ApiResponse::success(MessageResponse {
        message: "Message sent".to_string(),
    })))
}

pub async fn walk(
    State(manager): State<Arc<BotManager>>,
    Path(id): Path<Uuid>,
    Json(request): Json<WalkRequest>,
) -> Result<Json<ApiResponse<MessageResponse>>, StatusCode> {
    let bot = manager.get_bot(&id).ok_or(StatusCode::NOT_FOUND)?;

    bot.walk(request.x, request.y, request.autopilot.unwrap_or(false));

    Ok(Json(ApiResponse::success(MessageResponse {
        message: format!("Walking to ({}, {})", request.x, request.y),
    })))
}

pub async fn collect(
    State(manager): State<Arc<BotManager>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<MessageResponse>>, StatusCode> {
    let bot = manager.get_bot(&id).ok_or(StatusCode::NOT_FOUND)?;

    let count = bot.collect();

    Ok(Json(ApiResponse::success(MessageResponse {
        message: format!("Collected {} items", count),
    })))
}

pub async fn punch(
    State(manager): State<Arc<BotManager>>,
    Path(id): Path<Uuid>,
    Json(request): Json<PunchRequest>,
) -> Result<Json<ApiResponse<MessageResponse>>, StatusCode> {
    let bot = manager.get_bot(&id).ok_or(StatusCode::NOT_FOUND)?;

    bot.punch(request.offset_x, request.offset_y);

    Ok(Json(ApiResponse::success(MessageResponse {
        message: "Punching tile".to_string(),
    })))
}

pub async fn place(
    State(manager): State<Arc<BotManager>>,
    Path(id): Path<Uuid>,
    Json(request): Json<PlaceRequest>,
) -> Result<Json<ApiResponse<MessageResponse>>, StatusCode> {
    let bot = manager.get_bot(&id).ok_or(StatusCode::NOT_FOUND)?;

    bot.place(request.offset_x, request.offset_y, request.item_id);

    Ok(Json(ApiResponse::success(MessageResponse {
        message: "Placing item".to_string(),
    })))
}

// Bot Data Endpoints

pub async fn get_inventory(
    State(manager): State<Arc<BotManager>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<InventoryResponse>>, StatusCode> {
    let bot = manager.get_bot(&id).ok_or(StatusCode::NOT_FOUND)?;
    let item_db = manager.get_item_database();

    let all_items = bot.inventory.get_all_items();
    let (size, item_count) = bot.inventory.size_and_count();

    let item_db_lock = item_db.read().unwrap();

    let items: Vec<InventoryItemDto> = all_items
        .iter()
        .map(|(id, item)| {
            let name = item_db_lock
                .items
                .get(&(*id as u32))
                .map(|i| i.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            InventoryItemDto {
                id: *id,
                name,
                amount: item.amount,
                flag: item.flag,
            }
        })
        .collect();

    let response = InventoryResponse {
        size,
        item_count,
        items,
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn get_world(
    State(manager): State<Arc<BotManager>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<WorldResponse>>, StatusCode> {
    let bot = manager.get_bot(&id).ok_or(StatusCode::NOT_FOUND)?;

    let world_data = bot
        .world
        .data
        .try_lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let players_data = bot
        .world
        .players
        .try_lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let players: Vec<PlayerDto> = players_data
        .iter()
        .map(|(_, player)| PlayerDto {
            net_id: player.net_id,
            name: player.name.clone(),
            position: player.position,
            country: player.country.clone(),
        })
        .collect();

    // Count tiles
    let mut foreground_counts = std::collections::HashMap::new();
    let mut background_counts = std::collections::HashMap::new();

    for tile in &world_data.tiles {
        if tile.foreground_item_id != 0 {
            *foreground_counts
                .entry(tile.foreground_item_id)
                .or_insert(0u32) += 1;
        }
        if tile.background_item_id != 0 {
            *background_counts
                .entry(tile.background_item_id)
                .or_insert(0u32) += 1;
        }
    }

    let response = WorldResponse {
        name: world_data.name.clone(),
        width: world_data.width,
        height: world_data.height,
        players,
        tile_counts: TileCounts {
            foreground: foreground_counts,
            background: background_counts,
        },
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn get_logs(
    State(manager): State<Arc<BotManager>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<LogsResponse>>, StatusCode> {
    let bot = manager.get_bot(&id).ok_or(StatusCode::NOT_FOUND)?;

    let logs = bot.logs.read().unwrap().clone();

    Ok(Json(ApiResponse::success(LogsResponse { logs })))
}

pub async fn update_config(
    State(manager): State<Arc<BotManager>>,
    Path(id): Path<Uuid>,
    Json(request): Json<ConfigUpdate>,
) -> Result<Json<ApiResponse<MessageResponse>>, StatusCode> {
    let bot = manager.get_bot(&id).ok_or(StatusCode::NOT_FOUND)?;

    if let Some(auto_collect) = request.auto_collect {
        bot.set_auto_collect(auto_collect);
    }

    if let Some(auto_reconnect) = request.auto_reconnect {
        bot.set_auto_reconnect(auto_reconnect);
    }

    if let Some(delay) = request.findpath_delay {
        bot.set_findpath_delay(delay);
    }

    if let Some(delay) = request.punch_delay {
        bot.set_punch_delay(delay);
    }

    if let Some(delay) = request.place_delay {
        bot.set_place_delay(delay);
    }

    Ok(Json(ApiResponse::success(MessageResponse {
        message: "Configuration updated".to_string(),
    })))
}

// Helper functions

fn parse_socks5_config(proxy_str: &str) -> Result<Option<Socks5Config>, StatusCode> {
    let parts: Vec<&str> = proxy_str.split(':').collect();

    match parts.len() {
        2 => {
            let host = parts[0];
            let port = parts[1];

            match format!("{}:{}", host, port).to_socket_addrs() {
                Ok(mut addrs) => {
                    if let Some(proxy_addr) = addrs.next() {
                        Ok(Some(Socks5Config {
                            proxy_addr,
                            username: None,
                            password: None,
                        }))
                    } else {
                        Err(StatusCode::BAD_REQUEST)
                    }
                }
                Err(_) => Err(StatusCode::BAD_REQUEST),
            }
        }
        4 => {
            let host = parts[0];
            let port = parts[1];

            match format!("{}:{}", host, port).to_socket_addrs() {
                Ok(mut addrs) => {
                    if let Some(proxy_addr) = addrs.next() {
                        Ok(Some(Socks5Config {
                            proxy_addr,
                            username: Some(parts[2].to_string()),
                            password: Some(parts[3].to_string()),
                        }))
                    } else {
                        Err(StatusCode::BAD_REQUEST)
                    }
                }
                Err(_) => Err(StatusCode::BAD_REQUEST),
            }
        }
        _ => Err(StatusCode::BAD_REQUEST),
    }
}
