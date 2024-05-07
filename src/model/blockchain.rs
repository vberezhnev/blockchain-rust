use libp2p::Transport;
use log::info;
use sha2::{Digest, Sha256};

pub const DIFFICULTY_PREFIX: &str = "10";

pub fn calculate_hash(id: u64, timestamp: i64, prev_hash: &str, data: &str, nonce: u64) -> Vec<u8> {
    let data = serde_json::json!({
    "id": id,
    "timestamp": timestamp,
    "prev_hash": prev_hash,
    "data": data,
    "nonce": nonce,
    });

    let mut hasher = Sha256::new();
    hasher.update((data.to_string()).as_bytes());
    hasher.finalize().as_slice().to_owned()
}

pub fn hash_to_binary_representation(hash: &[u8]) -> String {
    let mut res: String = String::default();

    for i in hash {
        res.push_str(&format!("{:b}", i))
    }

    res
}
