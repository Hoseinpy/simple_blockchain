use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use hex;
use rmp_serde::{decode, encode};
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;

use crate::models::blockchain::BlockChain;

pub fn generate_sha256_hash(data: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();

    hex::encode(result)
}

pub fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn default_db_path() -> PathBuf {
    let current_dir = env::current_dir().unwrap();
    PathBuf::from(format!("{}/chain.bin", current_dir.display()))
}

pub fn save_chain_atomic(chain: &BlockChain, path: &PathBuf) -> Result<()> {
    let data = encode::to_vec(chain)?;

    let mut tmp = NamedTempFile::new_in(path.parent().unwrap_or_else(|| Path::new(".")))?;
    tmp.write_all(&data)?;
    tmp.flush()?;

    tmp.persist(path)
        .with_context(|| format!("Failed to persist blockchain to {:?}", path))?;

    Ok(())
}

pub fn load_chain(path: &PathBuf, difficulty: usize) -> Result<BlockChain> {
    if !path.exists() {
        return Ok(BlockChain::new(difficulty));
    }
    let bytes = fs::read(path).with_context(|| format!("read {:?}", path))?;
    let mut chain: BlockChain = decode::from_slice(&bytes).context("deserialize chain")?;
    chain.difficulty = difficulty;

    Ok(chain)
}
