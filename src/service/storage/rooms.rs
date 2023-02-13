use async_trait::async_trait;
use ethers_core::types::{Address, U256};

use crate::service::Room;

#[async_trait]
pub trait Storage {
    type Error: std::error::Error;

    async fn get_room(&self, amount: &U256, token: &Address) -> Result<Room, Self::Error>;

    async fn insert_room(&self, room: &Room) -> Result<(), Self::Error>;
}
