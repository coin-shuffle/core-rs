pub mod storage;
#[cfg(test)]
mod tests;
pub mod types;
pub mod waiter;

use std::collections::HashMap;

use ethers_core::types::{Address, U256};
use rsa::RsaPublicKey;

use self::{
    storage::participants,
    types::{participant::Participant, room::Room},
};

pub struct Service<S, W>
where
    S: storage::Storage,
    W: waiter::Waiter,
{
    storage: S,
    waiter: W,
}

impl<S, W> Service<S, W>
where
    S: storage::Storage,
    W: waiter::Waiter,
{
    pub fn new(storage: S, waiter: W) -> Self {
        Self { storage, waiter }
    }

    /// Add participant to the queue for given token and amount.
    pub async fn add_participant(
        &self,
        token: &Address,
        amount: &U256,
        participant: &Participant,
    ) -> Result<(), AddParticipantError<W::InternalError, <S as storage::Storage>::InternalError>>
    {
        self.storage
            .insert_participants(vec![participant.clone()]) // TODO:
            .await?;

        self.waiter
            .add_to_queue(token, amount, &participant.utxo_id)
            .await
            .map_err(AddParticipantError::Queue)?;

        Ok(())
    }

    /// Remove participants from the queue for given token and amount
    /// and form rooms for all of them. Return id's of newly created rooms.
    pub async fn create_rooms(
        &self,
        token: &Address,
        amount: &U256,
    ) -> Result<Vec<uuid::Uuid>, CreateRoomsError<W::InternalError>> {
        let rooms = self.waiter.organize(token, amount).await?;

        Ok(rooms)
    }

    /// Start shuffling in the room. Return pairs of participant id and array of
    /// RSA public keys needed to encrypt messages for the participant.
    pub async fn start_shuffle(
        &self,
        room_id: &uuid::Uuid,
    ) -> Result<
        HashMap<U256, Vec<RsaPublicKey>>,
        StartShuffleError<<S as storage::Storage>::InternalError>,
    > {
        let room = self
            .storage
            .get_room(room_id)
            .await
            .map_err(StartShuffleError::RoomStorage)?
            .ok_or(StartShuffleError::RoomNotFound)?;

        let participants_number = room.participants.len();

        let mut pairs = HashMap::with_capacity(participants_number);
        let mut keys = Vec::with_capacity(participants_number);

        for participant in room.participants.iter().rev() {
            let key = self
                .storage
                .get_participant(participant)
                .await
                .map_err(StartShuffleError::ParticipantStorage)?
                .ok_or(StartShuffleError::ParticipantNotFound)?
                .rsa_pubkey;

            dbg!(&keys);

            keys.push(key);

            pairs.insert(*participant, keys.clone());
        }

        Ok(pairs)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AddParticipantError<WE, DE>
where
    WE: std::error::Error,
    DE: std::error::Error,
{
    #[error("failed to insert participant: {0}")]
    InsertParticipant(#[from] participants::InsertError<DE>),
    #[error("failed to add paritipant to queue: {0}")]
    Queue(WE),
}

#[derive(thiserror::Error, Debug)]
pub enum CreateRoomsError<WE>
where
    WE: std::error::Error,
{
    #[error("failed to organize rooms: {0}")]
    Organize(#[from] WE),
}

#[derive(thiserror::Error, Debug)]
pub enum StartShuffleError<SE>
where
    SE: std::error::Error,
{
    #[error("failed to get room: {0}")]
    RoomStorage(SE),
    #[error("room no found")]
    RoomNotFound,
    #[error("failed to get participant: {0}")]
    ParticipantStorage(SE),
    #[error("participant no found")]
    ParticipantNotFound,
}
