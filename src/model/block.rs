use crate::{calculate_hash, hash_to_binary_representation, DIFFICULTY_PREFIX};
use chrono::prelude::*;
use log::info;
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
        let (nonce, curr_hash) = Self::mine_block(id, now.timestamp(), &prev_hash, &data);
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

    pub fn mine_block(id: u64, timestamp: i64, previous_hash: &str, data: &str) -> (u64, String) {
        println!("Mining block...");
        let mut nonce = 0;

        loop {
            if nonce % 100000 == 0 {
                info!("nonce: {}", nonce);
            }
            let hash = calculate_hash(id, timestamp, previous_hash, data, nonce);
            let binary_hash = hash_to_binary_representation(&hash);
            if binary_hash.starts_with(DIFFICULTY_PREFIX) {
                println!(
                    "mined! nonce: {}, hash: {}, binary hash: {}",
                    nonce,
                    hex::encode(&hash),
                    binary_hash
                );
                return (nonce, hex::encode(hash));
            }
            nonce += 1;
        }
    }
}
