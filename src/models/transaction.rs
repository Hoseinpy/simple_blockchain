use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::utils::{generate_sha256_hash, get_timestamp};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub txid: String,
    pub amount: Decimal,
    pub from: String,
    pub to: String,
    pub timestamp: u64,
}

impl Transaction {
    pub fn new(amount: Decimal, from: String, to: String) -> Self {
        let timestamp = get_timestamp();
        let txid = generate_sha256_hash(format!("{}{}{}{}", amount, from, to, timestamp));
        Self {
            txid,
            amount,
            from,
            to,
            timestamp,
        }
    }
}

#[derive(Deserialize)]
pub struct TransactionPayload {
    pub amount: Option<f64>,
    pub from: Option<String>,
    pub to: Option<String>,
}
