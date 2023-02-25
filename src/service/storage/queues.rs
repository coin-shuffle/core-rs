use async_trait::async_trait;
use ethers_core::types::{Address, U256};

#[async_trait]
pub trait Storage {
    /// Push new participant to queue. Create one if haven't been created before.
    ///
    /// [`participant`] - id of UTXO of the participant.
    async fn push_to_queue(
        &self,
        token: &Address,
        amount: &U256,
        paricipant: &U256,
    ) -> Result<(), Error>;

    /// Remove and return from queue first [`number`] participants
    async fn pop_from_queue(
        &self,
        token: &Address,
        amount: &U256,
        number: usize,
    ) -> Result<Vec<U256>, Error>;

    /// Return queue length
    async fn queue_len(&self, token: &Address, amount: &U256) -> Result<usize, Error>;
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("queue not found")]
    QueueNotFound,
    #[error("internal error: {0}")]
    Internal(String),
}
