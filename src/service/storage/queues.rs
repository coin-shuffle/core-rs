use async_trait::async_trait;
use ethers_core::types::{Address, U256};

#[async_trait]
pub trait Storage {
    type InternalError: std::error::Error;

    /// Push new participant to queue. Create one if haven't been created before.
    ///
    /// [`participant`] - id of UTXO of the participant.
    async fn push_to_queue(
        &self,
        token: &Address,
        amount: &U256,
        paricipant: &U256,
    ) -> Result<(), Self::InternalError>;

    /// Remove and return from queue first [`number`] participants
    async fn pop_from_queue(
        &self,
        token: &Address,
        amount: &U256,
        number: usize,
    ) -> Result<Vec<U256>, Error<Self::InternalError>>;
}

#[derive(thiserror::Error, Debug)]
pub enum Error<IE>
where
    IE: std::error::Error,
{
    #[error("queue not found")]
    QueueNotFound,
    #[error("internal error: {0}")]
    Internal(IE),
}
