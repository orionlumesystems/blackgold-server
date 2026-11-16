// src/handlers/entropy.rs
use crate::state::SharedState;
use axum::{extract::State, Json};
use serde_json::{json, Value};

pub async fn fetch_entropy(State(_state): State<SharedState>) -> Json<Value> {
    Json(json!({ "status": "entropy_fetched" }))
}
