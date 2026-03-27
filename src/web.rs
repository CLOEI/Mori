use axum::{
    Router,
    extract::{Path, Query, State, WebSocketUpgrade},
    extract::ws::{Message, WebSocket},
    http::{StatusCode, HeaderValue, Method},
    response::IntoResponse,
    routing::{delete, get, post},
    Json,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

use crate::bot::Socks5Config;
use crate::bot_manager::{BotInfo, BotManager};
use crate::bot_state::{BotCommand, BotDelays, BotState};
use crate::events::WsTx;
use crate::items::ItemInfo;

pub type SharedManager = Arc<Mutex<BotManager>>;

#[derive(Clone)]
pub struct AppState {
    pub manager: SharedManager,
    pub ws_tx:   WsTx,
}


async fn list_bots(State(s): State<AppState>) -> Json<Vec<BotInfo>> {
    Json(s.manager.lock().unwrap().list())
}

#[derive(Deserialize)]
struct SpawnRequest {
    username:       String,
    password:       String,
    proxy_host:     Option<String>,
    proxy_port:     Option<u16>,
    proxy_username: Option<String>,
    proxy_password: Option<String>,
}

async fn spawn_bot(
    State(s): State<AppState>,
    Json(req): Json<SpawnRequest>,
) -> Json<serde_json::Value> {
    let proxy = match (req.proxy_host, req.proxy_port) {
        (Some(host), Some(port)) => {
            let addr = format!("{}:{}", host, port).parse().ok();
            addr.map(|proxy_addr| Socks5Config {
                proxy_addr,
                username: req.proxy_username,
                password: req.proxy_password,
            })
        }
        _ => None,
    };
    let id = s.manager.lock().unwrap().spawn(req.username, req.password, proxy);
    Json(serde_json::json!({ "id": id }))
}

async fn stop_bot(
    State(s): State<AppState>,
    Path(id): Path<u32>,
) -> StatusCode {
    if s.manager.lock().unwrap().stop(id) {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn bot_state(
    State(s): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<BotState>, StatusCode> {
    s.manager.lock().unwrap()
        .get_state(id)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum CmdRequest {
    Move { x: i32, y: i32 },
    WalkTo { x: u32, y: u32 },
    RunScript { content: String },
    StopScript,
    Wear { item_id: u32 },
    Unwear { item_id: u32 },
    Drop { item_id: u32, count: u32 },
    Trash { item_id: u32, count: u32 },
    SetDelays(BotDelays),
}

#[derive(Deserialize)]
struct ItemsQuery {
    page: Option<usize>,
    q:    Option<String>,
}

#[derive(serde::Serialize)]
struct ItemsResponse {
    items:     Vec<ItemInfo>,
    total:     usize,
    page:      usize,
    page_size: usize,
}

const ITEMS_PAGE_SIZE: usize = 50;

async fn list_items(
    State(s): State<AppState>,
    Query(params): Query<ItemsQuery>,
) -> Json<ItemsResponse> {
    let mgr = s.manager.lock().unwrap();
    let q = params.q.as_deref().unwrap_or("").to_lowercase();
    let page = params.page.unwrap_or(1).max(1);

    let filtered: Vec<&ItemInfo> = mgr.items_dat.items.iter().filter(|i| {
        if q.is_empty() { return true; }
        if let Ok(id) = q.parse::<u32>() { if i.id == id { return true; } }
        i.name.to_lowercase().contains(&q)
    }).collect();

    let total = filtered.len();
    let start = (page - 1) * ITEMS_PAGE_SIZE;
    let items = filtered.into_iter().skip(start).take(ITEMS_PAGE_SIZE).cloned().collect();

    Json(ItemsResponse { items, total, page, page_size: ITEMS_PAGE_SIZE })
}

async fn item_names(State(s): State<AppState>) -> Json<std::collections::HashMap<u32, String>> {
    let mgr = s.manager.lock().unwrap();
    let map = mgr.items_dat.items.iter()
        .map(|i| (i.id, i.name.clone()))
        .collect();
    Json(map)
}

async fn bot_cmd(
    State(s): State<AppState>,
    Path(id): Path<u32>,
    Json(req): Json<CmdRequest>,
) -> StatusCode {
    let cmd = match req {
        CmdRequest::Move { x, y }   => BotCommand::Move { x, y },
        CmdRequest::WalkTo { x, y } => BotCommand::WalkTo { x, y },
        CmdRequest::RunScript { content }   => BotCommand::RunScript { content },
        CmdRequest::StopScript              => BotCommand::StopScript,
        CmdRequest::Wear { item_id }        => BotCommand::Wear { item_id },
        CmdRequest::Unwear { item_id }      => BotCommand::Unwear { item_id },
        CmdRequest::Drop { item_id, count } => BotCommand::Drop { item_id, count },
        CmdRequest::Trash { item_id, count } => BotCommand::Trash { item_id, count },
        CmdRequest::SetDelays(d) => BotCommand::SetDelays(d),
    };
    if s.manager.lock().unwrap().send_cmd(id, cmd) {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

// ── WebSocket handler ─────────────────────────────────────────────────────────

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(s): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, s.ws_tx.subscribe()))
}

async fn handle_socket(
    mut socket: WebSocket,
    mut rx: tokio::sync::broadcast::Receiver<crate::events::WsEvent>,
) {
    loop {
        match rx.recv().await {
            Ok(event) => {
                let msg = match serde_json::to_string(&event) {
                    Ok(s)  => s,
                    Err(_) => continue,
                };
                if socket.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            Err(_) => break,
        }
    }
}

async fn index_html() -> impl IntoResponse {
    let dist = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("dist");

    let html = tokio::fs::read_to_string(dist.join("index.html"))
        .await
        .unwrap_or_default();

    let injected = html.replace(
        "</body>",
        r#"<div style="position:fixed;bottom:8px;right:12px;font-size:10px;opacity:0.35;color:#fff;pointer-events:none;z-index:9999;font-family:sans-serif;">Mori created with &#x2764;&#xfe0e; by Cendy</div></body>"#,
    );

    axum::response::Html(injected)
}

pub async fn serve(manager: SharedManager, ws_tx: WsTx) {
    let state = AppState { manager, ws_tx };

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers(Any);

    let dist = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("dist");

    let app = Router::new()
        .route("/", get(index_html))

        .route("/bots", get(list_bots).post(spawn_bot))
        .route("/bots/{id}", delete(stop_bot))
        .route("/bots/{id}/state", get(bot_state))
        .route("/bots/{id}/cmd", post(bot_cmd))
        .route("/items", get(list_items))
        .route("/items/names", get(item_names))
        .route("/ws", get(ws_handler))
        .layer(cors)
        .with_state(state)
        .fallback_service(ServeDir::new(&dist).fallback(ServeFile::new(dist.join("index.html"))));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Dashboard  http://localhost:3000");
    println!("WebSocket  ws://localhost:3000/ws");
    println!("API        http://localhost:3000/bots");
    axum::serve(listener, app).await.unwrap();
}
