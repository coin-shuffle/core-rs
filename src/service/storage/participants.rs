use async_trait::async_trait;
use ethers_core::types::U256;
use rsa::RsaPublicKey;

use crate::service::types::{Participant, ShuffleRound};

#[async_trait]
pub trait Storage {
    type InternalError: std::error::Error;

    async fn insert_participants(
        &self,
        participant: Vec<Participant>,
    ) -> Result<(), InsertError<Self::InternalError>>;

    async fn update_participant_room(
        &self,
        participant: &U256,
        room_id: &uuid::Uuid,
    ) -> Result<(), UpdateError<Self::InternalError>>;

    async fn update_participant_round(
        &self,
        participant: &U256,
        round: ShuffleRound,
    ) -> Result<(), UpdateError<Self::InternalError>>;

    async fn add_participant_key(
        &self,
        participant: &U256,
        key: &RsaPublicKey,
    ) -> Result<(), UpdateError<Self::InternalError>>;

    async fn get_participant(
        &self,
        participant: &U256,
    ) -> Result<Option<Participant>, Self::InternalError>;
}

#[derive(thiserror::Error, Debug)]
pub enum InsertError<IE>
where
    IE: std::error::Error,
{
    #[error("conflict with existing participant: {0}")]
    Conflict(U256),
    #[error("internal error: {0}")]
    Internal(#[from] IE),
}

#[derive(thiserror::Error, Debug)]
pub enum UpdateError<IE>
where
    IE: std::error::Error,
{
    #[error("internal error: {0}")]
    Internal(#[from] IE),
    #[error("participant with utxo={0} not found")]
    NotFound(U256),
}
