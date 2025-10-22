use std::sync::Arc;

use serde::Deserialize;
use tokio::sync::{RwLock, broadcast};

use crate::models::{block::Block, blockchain::BlockChain};

#[derive(Debug, Clone)]
pub struct AppState {
    pub blockchain: Arc<RwLock<BlockChain>>,
    pub broadcaster: broadcast::Sender<Block>,
}

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
}
