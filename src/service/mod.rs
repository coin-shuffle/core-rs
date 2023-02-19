pub mod storage;
#[cfg(test)]
mod tests;
pub mod types;
pub mod waiter;

use ethers_core::types::{Address, U256};
use rsa::RsaPublicKey;

use self::{
    storage::{participants, rooms},
    types::{participant::Participant, room::Room, Output, ShuffleRound},
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
    ) -> Result<Vec<Room>, CreateRoomsError<W::InternalError>> {
        let rooms = self.waiter.organize(token, amount).await?;

        Ok(rooms)
    }

    /// Return keys that are needed to decrypt and encrypt the message for given room and participant.
    pub async fn participant_keys(
        &self,
        room_id: &uuid::Uuid,
        participant_id: &U256,
    ) -> Result<Vec<RsaPublicKey>, ParticipantKeysError<<S as storage::Storage>::InternalError>>
    {
        let tx = self.storage.transaction().await?;
        let room = tx
            .get_room(room_id)
            .await
            .map_err(GetRoomError::Storage)?
            .ok_or(GetRoomError::NotFound)?;

        let position = room
            .participants
            .iter()
            .position(|p| p == participant_id)
            .ok_or(ParticipantKeysError::ParticipantNotFound)?;

        let mut keys = Vec::with_capacity(room.participants.len() - position);

        for participant_id in room.participants.iter().skip(position) {
            let participant = tx
                .get_participant(participant_id)
                .await
                .map_err(GetParticipantError::Storage)?
                .ok_or(GetParticipantError::NotFound)?;

            keys.push(participant.rsa_pubkey.clone());
        }

        tx.update_participant_round(participant_id, ShuffleRound::Start(keys.clone()))
            .await?;

        tx.commit().await?;

        Ok(keys)
    }

    /// Return outputs that given participant should decrypt.
    pub async fn encoded_outputs(
        &self,
        room_id: &uuid::Uuid,
        participant_id: &U256,
    ) -> Result<Vec<Output>, EncodingOutputsError<<S as storage::Storage>::InternalError>> {
        let tx = self.storage.transaction().await?;
        let room = tx
            .get_room(room_id)
            .await
            .map_err(GetRoomError::Storage)?
            .ok_or(GetRoomError::NotFound)?;

        let position = room
            .participants
            .iter()
            .position(|id| id == participant_id)
            .ok_or(EncodingOutputsError::NoParticipantInRoom)?;

        if position != room.current_round {
            return Err(EncodingOutputsError::InvalidRound);
        }

        // Participant is the first one in the room, so he first adds his encoded outputs
        if position == 0 {
            return Ok(Vec::new());
        }

        let previous_participant_id = room.participants[position - 1];

        let previous_participant = tx
            .get_participant(&previous_participant_id)
            .await
            .map_err(GetParticipantError::Storage)?
            .ok_or(GetParticipantError::NotFound)?;

        // Get decoded outputs from the previous participant
        let ShuffleRound::DecodedOutputs(encoded_outputs) = previous_participant.status else {
            return Err(EncodingOutputsError::InvalidRound);
        };

        tx.commit().await?;

        Ok(encoded_outputs)
    }

    pub async fn pass_decoded_outputs(
        &self,
        room_id: &uuid::Uuid,
        participant_id: &U256,
        decoded_outputs: Vec<Output>,
    ) -> Result<(), DecodedOutputsError<<S as storage::Storage>::InternalError>> {
        let tx = self.storage.transaction().await?;

        let room = tx
            .get_room(room_id)
            .await
            .map_err(GetRoomError::Storage)?
            .ok_or(GetRoomError::NotFound)?;

        let position = room
            .participants
            .iter()
            .position(|id| id == participant_id)
            .ok_or(DecodedOutputsError::NoParticipantInRoom)?;

        if position != room.current_round {
            return Err(DecodedOutputsError::InvalidRound);
        }
        if decoded_outputs.len() != (position + 1) {
            return Err(DecodedOutputsError::InvalidNumberOfOutputs);
        }

        let participant = tx
            .get_participant(participant_id)
            .await
            .map_err(GetParticipantError::Storage)?
            .ok_or(GetParticipantError::NotFound)?;

        // check that previous status is Start
        match participant.status {
            ShuffleRound::Start(_) => {}
            _ => return Err(DecodedOutputsError::InvalidRound),
        };

        tx.update_room_round(room_id, room.current_round + 1)
            .await?;

        tx.update_participant_round(
            participant_id,
            ShuffleRound::DecodedOutputs(decoded_outputs),
        )
        .await?;

        Ok(())
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
pub enum ParticipantKeysError<SE>
where
    SE: std::error::Error,
{
    #[error("internal storage error: {0}")]
    Storage(#[from] SE),
    #[error("failed to get room: {0}")]
    RoomStorage(#[from] GetRoomError<SE>),
    #[error("no participant with given id in the room")]
    ParticipantNotFound,
    #[error("failed to get participant: {0}")]
    GetParticipant(#[from] GetParticipantError<SE>),
    #[error("failed to update participant round: {0}")]
    UpdateParticipantRound(#[from] participants::UpdateError<SE>),
}

#[derive(thiserror::Error, Debug)]
pub enum EncodingOutputsError<SE>
where
    SE: std::error::Error,
{
    #[error("internal storage error: {0}")]
    Storage(#[from] SE),
    #[error("failed to get room: {0}")]
    GetRoom(#[from] GetRoomError<SE>),
    #[error("failed to get participant: {0}")]
    GetParticipant(#[from] GetParticipantError<SE>),
    #[error("no participant in the room")]
    NoParticipantInRoom,
    #[error("invalid round")]
    InvalidRound,
}

#[derive(thiserror::Error, Debug)]
pub enum DecodedOutputsError<SE>
where
    SE: std::error::Error,
{
    #[error("internal storage error: {0}")]
    Storage(#[from] SE),
    #[error("failed to get room: {0}")]
    GetRoom(#[from] GetRoomError<SE>),
    #[error("failed to get participant: {0}")]
    GetParticipant(#[from] GetParticipantError<SE>),
    #[error("invalid round")]
    InvalidRound,
    #[error("no participant in the room")]
    NoParticipantInRoom,
    #[error("invalid number of ouputs")]
    InvalidNumberOfOutputs,
    #[error("failed to update participant round: {0}")]
    UpdateParticipantRound(#[from] participants::UpdateError<SE>),
    #[error("failed to update room round: {0}")]
    UpdateRoomRound(#[from] rooms::UpdateError<SE>),
}

impl<S, W> Service<S, W>
where
    S: storage::Storage,
    W: waiter::Waiter,
{
    pub async fn get_participant(
        &self,
        participant_id: &U256,
    ) -> Result<Participant, GetParticipantError<<S as storage::Storage>::InternalError>> {
        self.storage
            .get_participant(participant_id)
            .await
            .map_err(GetParticipantError::Storage)?
            .ok_or(GetParticipantError::NotFound)
    }

    pub async fn get_room(
        &self,
        room_id: &uuid::Uuid,
    ) -> Result<Room, GetRoomError<<S as storage::Storage>::InternalError>> {
        self.storage
            .get_room(room_id)
            .await
            .map_err(GetRoomError::Storage)?
            .ok_or(GetRoomError::NotFound)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetParticipantError<SE>
where
    SE: std::error::Error,
{
    #[error("Participant not found")]
    NotFound,
    #[error("Participant storage error: {0}")]
    Storage(#[from] SE),
}

#[derive(Debug, thiserror::Error)]
pub enum GetRoomError<SE>
where
    SE: std::error::Error,
{
    #[error("Room not found")]
    NotFound,
    #[error("Room storage error: {0}")]
    Storage(#[from] SE),
}
