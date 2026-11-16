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

// HTTP GET: Fetch recent chat history
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

// WebSocket Upgrade Handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

// WebSocket Connection Loop (Handles incoming messages, saves to DB, & broadcasts)
async fn handle_socket(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.ws_tx.subscribe();

    // Broadcast receiver task (sends messages out to this client)
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Receiver task (listens for incoming messages from this client)
    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    if let Ok(payload) = serde_json::from_str::<NewChatPayload>(&text) {
                        let trimmed_content = payload.content.trim();
                        if trimmed_content.is_empty() || trimmed_content.len() > 500 {
                            continue; // Skip empty or overly long spam messages
                        }

                        // Ensure user exists
                        let _ = sqlx::query(
                            "INSERT INTO users (wallet_address) VALUES ($1) ON CONFLICT DO NOTHING",
                        )
                        .bind(&payload.wallet_address)
                        .execute(&state_clone.db)
                        .await;

                        // Save message to Postgres
                        if let Ok(saved) = sqlx::query_as::<_, ChatMessage>(
                            r#"
                            INSERT INTO chat_messages (wallet_address, content)
                            VALUES ($1, $2)
                            RETURNING id, wallet_address, NULL::TEXT as username, content, created_at
                            "#,
                        )
                        .bind(&payload.wallet_address)
                        .bind(&trimmed_content)
                        .fetch_one(&state_clone.db)
                        .await
                        {
                            if let Ok(json_out) = serde_json::to_string(&saved) {
                                let _ = state_clone.ws_tx.send(json_out);
                            }
                        }
                    }
                }
                Ok(Message::Ping(p)) => {
                    if sender.send(Message::Pong(p)).await.is_err() {
                        break;
                    }
                }
                _ => break, // Disconnect on close or protocol error
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}