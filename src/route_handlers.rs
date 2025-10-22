use axum::{
    Json,
    extract::{Query, State, WebSocketUpgrade, ws::Message},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use serde_json::{Value, json};
use tokio::sync::broadcast;

use crate::{
    AppState,
    models::{
        transaction::{Transaction, TransactionPayload},
        utils::Pagination,
    },
};

pub async fn handle_health() -> Json<Value> {
    Json(json!({ "health": "OK" }))
}

pub async fn get_current_chain(
    State(app_state): State<AppState>,
    Query(pagination): Query<Pagination>,
) -> Json<Value> {
    let page = pagination.page.unwrap_or(1);
    let page_size = pagination.page_size.unwrap_or(50);
    let offset = (page - 1) * page_size;

    let chain = {
        let bc = app_state.blockchain.read().await;
        bc.chain.clone()
    };

    let paged_chain = chain
        .clone()
        .into_iter()
        .skip(offset)
        .take(page_size)
        .collect::<Vec<_>>();

    Json(
        json!({ "success": true, "count": chain.len(), "page": page, "page_size": page_size, "chain": paged_chain }),
    )
}

pub async fn get_current_chain_ws(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
) -> Response {
    ws.on_upgrade(|mut socket| async move {
        let mut rx = app_state.broadcaster.subscribe();

        loop { // TODO: FIX BUG. WHEN CLIENT DISCONNECTED DONT SEND MESSAGE
            match rx.recv().await {
                Ok(new_block) => {
                    if socket
                        .send(Message::text(
                            serde_json::to_string_pretty(&new_block).unwrap(),
                        ))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    eprintln!("WebSocket skipped {} blocks", skipped);
                }
                Err(_) => break,
            }
        }
    })
}

pub async fn handle_new_transaction(
    State(app_state): State<AppState>,
    Json(payload): Json<TransactionPayload>,
) -> impl IntoResponse {
    if payload.amount.is_none() || payload.from.is_none() || payload.to.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "status": "NOK", "message": "bad payload" })),
        );
    }
    {
        let mut bc = app_state.blockchain.write().await;
        let tx = Transaction::new(
            Decimal::from_f64(payload.amount.unwrap()).unwrap(),
            payload.from.unwrap(),
            payload.to.unwrap(),
        );
        bc.memory_pool.push(tx);
    }

    return (
        StatusCode::OK,
        Json(json!({ "status": "OK", "message": "transaction successfully added to memory pool" })),
    );
}
