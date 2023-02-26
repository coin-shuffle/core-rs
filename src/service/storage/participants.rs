use async_trait::async_trait;
use ethers_core::types::U256;
use rsa::RsaPublicKey;

use crate::service::types::{Participant, ShuffleRound};

#[async_trait]
pub trait Storage {
    async fn insert_participant(&self, participant: Participant) -> Result<(), Error>;

    async fn update_participant_room(
        &self,
        participant: &U256,
        room_id: &uuid::Uuid,
    ) -> Result<(), Error>;

    async fn update_participant_round(
        &self,
        participant: &U256,
        round: ShuffleRound,
    ) -> Result<(), Error>;

    async fn update_participant_key(
        &self,
        participant: &U256,
        key: RsaPublicKey,
    ) -> Result<(), Error>;

    async fn get_participant(&self, participant: &U256) -> Result<Option<Participant>, Error>;
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("internal error: {0}")]
    Internal(String),
}
