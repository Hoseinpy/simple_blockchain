use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    models::{block::Block, transaction::Transaction, utils::AppState},
    utils::save_chain_atomic,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockChain {
    pub chain: Vec<Block>,
    pub difficulty: usize,
    pub memory_pool: Vec<Transaction>,
}

impl BlockChain {
    pub fn new(difficulty: usize) -> Self {
        // create genesis block
        let genesis_block = Block::new(Vec::new(), 0, None, difficulty);

        Self {
            chain: vec![genesis_block],
            difficulty: difficulty,
            memory_pool: Vec::new(),
        }
    }

    pub fn latest_block(&self) -> Block {
        self.chain.last().cloned().expect("chain is empty")
    }

    pub fn mine_memory_pool(app_state: AppState, db_path: PathBuf) {
        tokio::spawn(async move {
            loop {
                let (transactions, difficulty, index, previous_block_hash) = {
                    let mut bc = app_state.blockchain.write().await;

                    let transactions = bc.memory_pool.clone();
                    bc.memory_pool.clear();

                    let difficulty = bc.difficulty;
                    let latest_block_in_chain = bc.latest_block();
                    let index = latest_block_in_chain.header.index + 1;
                    let previous_block_hash = latest_block_in_chain.header.block_hash;

                    (transactions, difficulty, index, previous_block_hash)
                };

                let new_block = tokio::task::spawn_blocking(move || {
                    let mut block =
                        Block::new(transactions, index, Some(previous_block_hash), difficulty);
                    block.mine(difficulty);
                    block
                })
                .await
                .expect("mining thread crashed");

                // write to file
                {
                    let bc = app_state.blockchain.read().await;
                    save_chain_atomic(&bc, &db_path).unwrap();
                }

                // push block to chain and clear memory pool
                {
                    let mut bc = app_state.blockchain.write().await;
                    bc.chain.push(new_block.clone());
                }

                // broadcast new block to websockets
                let _ = app_state.broadcaster.send(new_block.clone());
            }
        });
    }
}
