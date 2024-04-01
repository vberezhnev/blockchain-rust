mod block;
mod blockchain;

pub use block::Block;
pub use blockchain::{
    calculate_hash, hash_to_binary_representation, mine_block, DIFFICULTY_PREFIX,
};
