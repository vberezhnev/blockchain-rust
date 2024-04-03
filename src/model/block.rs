use crate::{calculate_hash, hash_to_binary_representation, DIFFICULTY_PREFIX};
use chrono::prelude::*;
use log::info;
use rsa::RsaPrivateKey;
use serde::{Deserialize, Serialize};

pub const RAND_BYTES: i32 = 32;
pub const START_PERCENT: i32 = 10;
pub const STORAGE_REWARD: i32 = 1; // coin

// FIXME: Resolve conflict with Blockchain structure from... We didn't have blockchain structure before?
pub struct BlockChain {
    // database: ,
    index: u64
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub id: u64,
    pub data: String,
    pub curr_hash: String,
    pub prev_hash: String,
    pub timestamp: i64,
    pub transaction: Vec<Transaction>,
    pub nonce: u64,
}

pub struct Transaction {
    rand_bytes: i32,
    prev_block: i32,
    sender: String,
    receiver: String,
    value: u64,
    to_storage: u64,
    curr_hash: i32,
    signature: u64,
}

pub struct User {
    private_key: RsaPrivateKey,
}

// TODO: Present it as implementation of Transaction structure
pub fn new_transaction(user: User, last_hash: String, to: String, value: u64) -> Transaction {
    let tx = Transaction {
        rand_bytes: generate_random_bytes(RAND_BYTES),
        prev_block: last_hash,
        sender: user.Address(),
        receiver: to,
        value: value,
    };

    if value > START_PERCENT {
        tx.to_storage = STORAGE_REWARD
    }

    tx.curr_hash = tx.hash(); // FIXME: .hash() is invalid function
    tx.signature = tx.sign(user.Private()); // FIXME: .Private() is invalid function
    tx
}

pub fn add_transaction(chain: )

/* ------------------------- */

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
