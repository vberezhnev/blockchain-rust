mod block;
mod blockchain;

pub use block::Block;
pub use blockchain::{calculate_hash, hash_to_binary_representation, DIFFICULTY_PREFIX};
