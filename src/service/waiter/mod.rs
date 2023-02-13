use ethers_core::types::{Address, U256};

use super::types::{Participant, Room};

pub mod simple;

/// `Waiter` organizes the waited participants into rooms
#[async_trait::async_trait]
pub trait Waiter {
    type Error: std::error::Error;

    /// Add participant to queue that will be organized later
    async fn add_to_queue(
        &self,
        token: &Address,
        amount: &U256,
        participant: &U256,
    ) -> Result<(), Self::Error>;

    async fn organize(&self, token: &Address, amount: &U256) -> Result<Vec<Room>, Self::Error>;
}
