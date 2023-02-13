use async_trait::async_trait;
use ethers_core::types::U256;

use crate::service::types::Participant;

#[async_trait]
pub trait Storage {
    type Error: std::error::Error;

    async fn insert_participants(&self, participant: Vec<Participant>) -> Result<(), Self::Error>;

    async fn update_participant_room(
        &self,
        participant: &U256,
        room_id: &uuid::Uuid,
    ) -> Result<(), Self::Error>;
}
