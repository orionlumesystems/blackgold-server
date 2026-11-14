use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct ChatMessage {
    pub id: i64,
    pub wallet_address: String,
    pub username: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NewChatPayload {
    pub wallet_address: String,
    pub content: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct RoundSummary {
    pub round_id: i64,
    pub status: String,
    pub winning_tile: Option<i16>,
    pub total_deployed: i64,
    pub total_payout: i64,
}