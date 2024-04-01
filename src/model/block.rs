use super::blockchain::mine_block;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub id: u64,
    pub data: String,
    pub curr_hash: String,
    pub prev_hash: String,
    pub timestamp: i64,
    // signature: u64,
    pub nonce: u64,
}

impl Block {
    pub fn new(
        id: u64,
        data: String,
        // curr_hash: String,
        prev_hash: String,
        // signature: u64,
    ) -> Self {
        let now = Utc::now();
        let (nonce, curr_hash) = mine_block(id, now.timestamp(), &prev_hash, &data);
        Self {
            id,
            data,
            curr_hash,
            prev_hash,
            timestamp: now.timestamp(),
            // signature,
            nonce,
        }
    }
}
