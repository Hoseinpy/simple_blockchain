use serde::{Deserialize, Serialize};
use std::env;

use crate::{
    models::transaction::Transaction,
    utils::{generate_sha256_hash, get_timestamp},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub index: u64,
    pub version: String,
    pub previous_block_hash: Option<String>,
    pub block_hash: String,
    pub merkle_root_hash: String,
    pub timestamp: u64,
    pub nonce: u64,
    pub difficulty: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(
        transactions: Vec<Transaction>,
        index: u64,
        previous_block_hash: Option<String>,
        difficulty: usize,
    ) -> Self {
        let timestamp = get_timestamp();
        let merkle_root_hash = generate_sha256_hash(
            transactions
                .iter()
                .map(|tx| {
                    format!(
                        "{}{}{}{}{}",
                        tx.txid, tx.amount, tx.from, tx.to, tx.timestamp
                    )
                })
                .collect::<Vec<String>>()
                .join(""),
        );

        let nonce: u64 = 0;

        let hash = generate_sha256_hash(format!(
            "{}{}{}{}",
            index, timestamp, merkle_root_hash, nonce
        ));

        Self {
            header: BlockHeader {
                index: index,
                version: env!("CARGO_PKG_VERSION").to_string(),
                previous_block_hash: previous_block_hash,
                block_hash: hash,
                merkle_root_hash: merkle_root_hash,
                timestamp: timestamp,
                nonce: nonce,
                difficulty: difficulty,
            },
            transactions: transactions,
        }
    }

    pub fn mine(&mut self, difficulty: usize) {
        let target_prefix = "0".repeat(difficulty);
        loop {
            let h = generate_sha256_hash(format!(
                "{}{}{}{}",
                self.header.index,
                self.header.timestamp,
                self.header.merkle_root_hash,
                self.header.nonce
            ));
            if h.starts_with(&target_prefix) {
                self.header.block_hash = h;
                break;
            } else {
                self.header.nonce += 1;
            }
        }
    }
}
