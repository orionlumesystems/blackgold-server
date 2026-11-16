mod handlers;
mod models;
mod services;
mod state;

use axum::{
    routing::{get, post},
    Router,
};
use solana_client::rpc_client::RpcClient;
use sqlx::postgres::PgPoolOptions;
use std::{sync::Arc, time::Duration};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

use crate::state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/blkg_mining".to_string());

    let db = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL container");

    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let solana_rpc = Arc::new(RpcClient::new(rpc_url));

    let (ws_tx, _rx) = broadcast::channel::<String>(512);

    let state = Arc::new(AppState {
        db,
        solana_rpc,
        ws_tx,
    });

    // Spawn background indexer worker
    let indexer_state = Arc::clone(&state);
    tokio::spawn(async move {
        services::indexer::run_indexer(indexer_state).await;
    });

    let app = Router::new()
        .route(
            "/health",
            get(|| async { "BlackGold Protocol Server Healthy" }),
        )
        // Chat (GET history only, POST removed)
        .route("/api/chat", get(handlers::chat::get_history))
        .route("/ws", get(handlers::chat::ws_handler))
        // Protocol Endpoints
        .route("/api/protocol/rounds", get(handlers::protocol::get_rounds))
        .route("/api/mint", post(handlers::mint::mint_tokens))
        .route("/api/entropy", get(handlers::entropy::fetch_entropy))
        .route("/api/stake", post(handlers::stake::stake_tokens))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("Failed to bind port");

    println!("BlackGold Server listening on http://0.0.0.0:{port}");
    axum::serve(listener, app).await.unwrap();
}
