use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    Json,
};
use futures_util::{SinkExt, StreamExt};

use crate::{
    models::{ChatMessage, NewChatPayload},
    state::SharedState,
};

pub async fn get_history(State(state): State<SharedState>) -> Json<Vec<ChatMessage>> {
    let messages = sqlx::query_as::<_, ChatMessage>(
        r#"
        SELECT m.id, m.wallet_address, u.username, m.content, m.created_at
        FROM chat_messages m
        LEFT JOIN users u ON m.wallet_address = u.wallet_address
        WHERE m.deleted_at IS NULL
        ORDER BY m.created_at DESC
        LIMIT 50
        "#,
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(messages)
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.ws_tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            if let Ok(payload) = serde_json::from_str::<NewChatPayload>(&text) {
                let _ = sqlx::query(
                    "INSERT INTO users (wallet_address) VALUES ($1) ON CONFLICT DO NOTHING",
                )
                .bind(&payload.wallet_address)
                .execute(&state_clone.db)
                .await;

                if let Ok(saved) = sqlx::query_as::<_, ChatMessage>(
                    r#"
                    INSERT INTO chat_messages (wallet_address, content)
                    VALUES ($1, $2)
                    RETURNING id, wallet_address, NULL::TEXT as username, content, created_at
                    "#,
                )
                .bind(&payload.wallet_address)
                .bind(&payload.content)
                .fetch_one(&state_clone.db)
                .await
                {
                    if let Ok(json_out) = serde_json::to_string(&saved) {
                        let _ = state_clone.ws_tx.send(json_out);
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}