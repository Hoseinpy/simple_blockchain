use std::sync::Arc;

use crate::{
    models::{blockchain::BlockChain, utils::AppState},
    route_handlers::{
        get_current_chain, get_current_chain_ws, handle_health, handle_new_transaction,
    },
    utils::{default_db_path, load_chain},
};
use anyhow::Result;
use axum::{
    Router,
    routing::{any, get, post},
};
use tokio::sync::{RwLock, broadcast};

mod models;
mod route_handlers;
mod utils;

const SERVER_ADDRESS: &str = "0.0.0.0";
const SERVER_PORT: &str = "3000";

#[tokio::main]
async fn main() -> Result<()> {
    // init db_path and blockchain and broadcaster
    let db_path = default_db_path();
    let blockchain = Arc::new(RwLock::new(load_chain(&db_path, 5)?));
    let (tx, _rx) = broadcast::channel(100);

    // init AppState
    let app_state = AppState {
        blockchain: Arc::clone(&blockchain),
        broadcaster: tx.clone(),
    };

    // axum router app config (route, etc..)
    let app = Router::new()
        .route("/api/health", get(handle_health))
        .route(
            "/api/chain",
            get(get_current_chain).with_state(app_state.clone()),
        )
        .route(
            "/api/new_transaction",
            post(handle_new_transaction).with_state(app_state.clone()),
        )
        .route(
            "/ws/chain",
            any(get_current_chain_ws).with_state(app_state.clone()),
        );

    // run mine memory pool task inside block
    {
        BlockChain::mine_memory_pool(app_state, db_path);
    }

    // run axum server on (SERVER_ADDRESS, SERVER_PORT)
    println!("running server at {}:{}", SERVER_ADDRESS, SERVER_PORT);
    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", SERVER_ADDRESS, SERVER_PORT)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
