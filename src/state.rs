use solana_client::rpc_client::RpcClient;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct AppState {
    pub db: PgPool,
    pub solana_rpc: Arc<RpcClient>,
    pub ws_tx: broadcast::Sender<String>,
}

pub type SharedState = Arc<AppState>;