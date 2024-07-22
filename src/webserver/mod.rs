use std::{fs, net::SocketAddr, sync::Arc};

use axum::{
    extract::{
        ws::{self, Message, WebSocket},
        WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use spdlog::info;

use crate::{manager::Manager, types::e_login_method::ELoginMethod};

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub game_version: String,
    pub protocol: String,
    pub bots: Vec<Bot>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Bot {
    pub username: String,
    pub password: String,
    pub token: String,
    pub login_method: ELoginMethod,
}

#[derive(Serialize, Deserialize)]
struct AddBotMessage {
    _type: String,
    data: Bot,
}

#[tokio::main]
pub async fn start(manager: Arc<Manager>) {
    let _ = match fs::read_to_string("data.json") {
        Ok(data) => data,
        Err(_) => {
            let data = Data {
                game_version: "4.61".to_string(),
                protocol: "209".to_string(),
                bots: vec![],
            };
            let serialized = serde_json::to_string_pretty(&data).expect("Failed to serialize data");
            fs::write("data.json", &serialized).unwrap();
            serialized
        }
    };

    let app = Router::new().route("/ws", get(|ws: WebSocketUpgrade| handler(ws, manager)));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Socket started on localhost with port 3000");
    axum::serve(listener, app).await.unwrap();
}

async fn handler(ws: WebSocketUpgrade, manager: Arc<Manager>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, manager))
}

async fn handle_socket(mut socket: WebSocket, manager: Arc<Manager>) {
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                let msg = serde_json::from_str::<Value>(&text).unwrap();
                if msg["_type"] == "get_data" {
                    let data = fs::read_to_string("data.json").unwrap();
                    let message = json::stringify(json::object! {
                        "_type" => "data",
                        "data" => data
                    });
                    socket.send(Message::Text(message)).await.unwrap();
                }
                if msg["_type"] == "get_item_database" {
                    let item_database = &manager.items_database;
                    let message = serde_json::to_string(&serde_json::json!({
                        "_type": "item_database",
                        "data": {
                            "version": item_database.version,
                            "item_count": item_database.item_count,
                            "items": item_database.items
                        }
                    }))
                    .unwrap();
                    socket.send(Message::Text(message)).await.unwrap();
                }
                if msg["_type"] == "add_bot" {
                    let messsage: AddBotMessage = serde_json::from_str(&msg.to_string())
                        .expect("Wrong format to use add_bot");
                    let data = fs::read_to_string("data.json").unwrap();
                    let mut data: Data = serde_json::from_str(&data).unwrap();
                    let bot = Bot {
                        username: messsage.data.username,
                        password: messsage.data.password,
                        token: messsage.data.token,
                        login_method: messsage.data.login_method,
                    };
                    data.bots.push(bot.clone());
                    let serialized =
                        serde_json::to_string(&data).expect("Failed to serialize data");
                    fs::write("data.json", &serialized).unwrap();
                    let data = fs::read_to_string("data.json").unwrap();
                    let message = serde_json::to_string_pretty(&serde_json::json!({
                        "_type": "data",
                        "data": data
                    }))
                    .unwrap();
                    socket.send(Message::Text(message)).await.unwrap();
                }
                if msg["_type"] == "get_world" {}
                if msg["_type"] == "remove_bot" {}
                if msg["_type"] == "update_bot" {}
            }
            Ok(Message::Binary(bin)) => {
                println!("Received: {:?}", bin);
                socket.send(Message::Binary(bin)).await.unwrap();
            }
            Ok(Message::Ping(ping)) => {
                println!("Received: Ping");
                socket.send(Message::Pong(ping)).await.unwrap();
            }
            Ok(Message::Pong(_)) => {
                println!("Received: Pong");
            }
            Ok(Message::Close(_)) => {
                println!("Received: Close");
                break;
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }
}
