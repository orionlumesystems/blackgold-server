use axum::{extract::State, Json};
use crate::{models::RoundSummary, state::SharedState};

pub async fn get_rounds(State(state): State<SharedState>) -> Json<Vec<RoundSummary>> {
    let rounds = sqlx::query_as::<_, RoundSummary>(
        "SELECT round_id, status, winning_tile, total_deployed, total_payout FROM rounds ORDER BY round_id DESC LIMIT 20"
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(rounds)
}