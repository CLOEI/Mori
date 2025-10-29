mod api;
mod bot_manager;
mod dto;
mod templates;

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
};

use crate::bot_manager::BotManager;

#[tokio::main]
async fn main() {
    let bot_manager = Arc::new(BotManager::new());
    let app = Router::new()
        // Web UI routes
        .route("/", get(templates::index))

        // API routes - Bot management
        .route("/api/bots", get(api::list_bots).post(api::create_bot))
        .route("/api/bots/{id}", get(api::get_bot).delete(api::remove_bot))

        // API routes - Bot actions
        .route("/api/bots/{id}/connect", post(api::connect_bot))
        .route("/api/bots/{id}/disconnect", post(api::disconnect_bot))
        .route("/api/bots/{id}/warp", post(api::warp_bot))
        .route("/api/bots/{id}/say", post(api::say))
        .route("/api/bots/{id}/walk", post(api::walk))
        .route("/api/bots/{id}/collect", post(api::collect))
        .route("/api/bots/{id}/punch", post(api::punch))
        .route("/api/bots/{id}/place", post(api::place))

        // API routes - Bot data
        .route("/api/bots/{id}/inventory", get(api::get_inventory))
        .route("/api/bots/{id}/world", get(api::get_world))
        .route("/api/bots/{id}/logs", get(api::get_logs))
        .route("/api/bots/{id}/config", post(api::update_config))

        // Static files
        .nest_service("/static", ServeDir::new("static"))
        .with_state(bot_manager)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("🚀 Mori web server starting on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}
