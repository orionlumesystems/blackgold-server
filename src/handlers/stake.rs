// src/handlers/stake.rs
use crate::state::SharedState;
use axum::{extract::State, Json};
use serde_json::{json, Value};

pub async fn stake_tokens(
    State(_state): State<SharedState>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    Json(json!({ "status": "stake_received", "data": payload }))
}
