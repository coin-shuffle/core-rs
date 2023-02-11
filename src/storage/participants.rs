use async_trait::async_trait;

use crate::types::Participant;

/// A struct for filters for querying participants
pub struct Selector {
    pub room_id: Option<uuid::Uuid>,
}

#[async_trait]
pub trait Storage {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn get_paritipant(&self, id: &uuid::Uuid) -> Result<Participant, Self::Error>;
    async fn get_participants(&self, selector: &Selector) -> Result<Vec<Participant>, Self::Error>;
    async fn save_participant(&self, participant: &Participant) -> Result<(), Self::Error>;
    async fn delete_participant(&self, id: &uuid::Uuid) -> Result<(), Self::Error>;
    async fn update_participant(&self, participant: &Participant) -> Result<(), Self::Error>;
}
