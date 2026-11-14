// src/handlers/entropy.rs
use axum::{extract::State, Json};
use serde_json::{json, Value};
use crate::state::SharedState;

pub async fn fetch_entropy(State(_state): State<SharedState>) -> Json<Value> {
    Json(json!({ "status": "entropy_fetched" }))
}