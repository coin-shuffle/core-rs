use ethers_core::types::U256;

use crate::node::storage;
use crate::rsa::RSAError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("utxo doesn't exist id: {0}")]
    UtxoDoesntExist(U256),
    #[error("Storage error: {0}")]
    Storage(#[from] storage::memory::Error),
    #[error("invalid owner: {0}")]
    InvalidOwner(String),
    #[error("room with specified UTXO doesn't exist utxo_id: {0}")]
    RoomDoesntExist(U256),
    #[error("failed to decode by chunks: {0}")]
    DecodeByChunks(RSAError),
    #[error("failed to encode by chunks: {0}")]
    EncodeByChunks(RSAError),
}
