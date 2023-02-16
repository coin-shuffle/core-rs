pub mod simple;

use ethers_core::types::{Address, U256};

use super::types::Room;

/// `Waiter` a generic trait for something that will organize participants into rooms
/// using some algorithm
#[async_trait::async_trait]
pub trait Waiter {
    type InternalError: std::error::Error;

    /// Add participant to queue that will be organized into rooms later
    async fn add_to_queue(
        &self,
        token: &Address,
        amount: &U256,
        participant: &U256,
    ) -> Result<(), Self::InternalError>;

    /// Organize participants inside queue into rooms and return it's IDs.
    async fn organize(
        &self,
        token: &Address,
        amount: &U256,
    ) -> Result<Vec<Room>, Self::InternalError>;
}
