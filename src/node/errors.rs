use ethers_core::types::U256;

use crate::node::storage;
use crate::rsa;

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
    #[error("failed to decrypt by chunks: {0}")]
    DecryptByChunks(rsa::Error),
    #[error("failed to encrypt by chunks: {0}")]
    EncryptByChunks(rsa::Error),
}
