use async_trait::async_trait;
use ethers_core::types::{Address, U256};

#[async_trait]
pub trait Storage {
    type Error: std::error::Error;

    /// Push new participant to queue. Create one if haven't been created before.
    ///
    /// [`participant`] - id of UTXO of the participant.
    async fn push_to_queue(
        &self,
        token: &Address,
        amount: &U256,
        paricipant: &U256,
    ) -> Result<(), Self::Error>;

    /// Remove and return from queue first [`number`] participants
    async fn pop_from_queue(
        &self,
        token: &Address,
        amount: &U256,
        number: u64,
    ) -> Result<Vec<U256>, Self::Error>;

    /// Return number of participants inside the queue
    async fn queue_length(&self, token: &Address, amount: &U256) -> Result<u64, Self::Error>;

    async fn clean_queue(&self, token: Address, amount: U256) -> Result<(), Self::Error>;
}
