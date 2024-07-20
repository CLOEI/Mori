use axum::{
    extract::{ws::WebSocket, WebSocketUpgrade},
    routing::get,
    Router,
};
use spdlog::info;

#[tokio::main]
pub async fn start() {
    let app = Router::new().route("/", get(handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Socket started on localhost with port 3000");
    axum::serve(listener, app).await.unwrap();
}

async fn handler(ws: WebSocketUpgrade) {
    let _ = ws.on_upgrade(handle_socket);
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            return;
        };

        if socket.send(msg).await.is_err() {
            return;
        }
    }
}
