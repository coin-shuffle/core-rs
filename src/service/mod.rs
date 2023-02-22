pub mod storage;
#[cfg(test)]
mod tests;
pub mod types;
pub mod waiter;

use coin_shuffle_contracts_bindings::utxo;
use ethers_core::{
    abi::{ethereum_types::Signature, Hash},
    types::{Address, U256},
};

use rsa::RsaPublicKey;

use self::{
    storage::{participants, rooms},
    types::{participant::Participant, room::Room, EncodedOuput, ShuffleRound},
};

pub struct Service<S, W, C>
where
    S: storage::Storage,
    W: waiter::Waiter,
    C: utxo::Contract,
{
    storage: S,
    waiter: W,
    utxo_contract: C,
}

impl<S, W, C> Service<S, W, C>
where
    S: storage::Storage,
    W: waiter::Waiter,
    C: utxo::Contract,
{
    pub fn new(storage: S, waiter: W, contract: C) -> Self {
        Self {
            storage,
            waiter,
            utxo_contract: contract,
        }
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
        participant_id: &U256,
    ) -> Result<Vec<RsaPublicKey>, ParticipantKeysError<<S as storage::Storage>::InternalError>>
    {
        let participant = self
            .storage
            .get_participant(participant_id)
            .await?
            .ok_or(ParticipantKeysError::ParticipantNotFound)?;

        let tx = self.storage.transaction().await?;
        let room = tx
            .get_room(&participant.room_id.unwrap()) // FIXME:
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
        participant_id: &U256,
    ) -> Result<Vec<EncodedOuput>, EncodingOutputsError<<S as storage::Storage>::InternalError>>
    {
        let tx = self.storage.transaction().await?;

        let participant = tx.get_participant(participant_id).await?.ok_or(
            EncodingOutputsError::GetParticipant(GetParticipantError::NotFound),
        )?;

        let room = tx
            .get_room(participant.room_id.as_ref().unwrap()) // FIXME:
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
        participant_id: &U256,
        decoded_outputs: Vec<EncodedOuput>,
    ) -> Result<(), DecodedOutputsError<<S as storage::Storage>::InternalError>> {
        let tx = self.storage.transaction().await?;

        let participant = tx
            .get_participant(participant_id)
            .await
            .map_err(GetParticipantError::Storage)?
            .ok_or(GetParticipantError::NotFound)?;

        let room = tx
            .get_room(participant.room_id.as_ref().unwrap()) // FIXME:
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

        // check that previous status is Start
        match participant.status {
            ShuffleRound::Start(_) => {}
            _ => return Err(DecodedOutputsError::InvalidRound),
        };

        tx.update_room_round(&room.id, room.current_round + 1)
            .await?;

        tx.update_participant_round(
            participant_id,
            ShuffleRound::DecodedOutputs(decoded_outputs),
        )
        .await?;

        Ok(())
    }

    /// If shuffle is finished, return all outputs that each participant should sign.
    pub async fn decoded_outputs(
        &self,
        room_id: &uuid::Uuid,
    ) -> Result<Vec<utxo::types::Output>, DecodedOutputsError<<S as storage::Storage>::InternalError>>
    {
        let tx = self.storage.transaction().await?;

        let room = tx
            .get_room(room_id)
            .await
            .map_err(GetRoomError::Storage)?
            .ok_or(GetRoomError::NotFound)?;

        if room.current_round != room.participants.len() {
            return Err(DecodedOutputsError::InvalidRound);
        }

        let last_participant = tx
            .get_participant(room.participants.last().unwrap())
            .await
            .map_err(GetParticipantError::Storage)?
            .ok_or(GetParticipantError::NotFound)?;

        let ShuffleRound::DecodedOutputs(outputs) = last_participant.status else {
            return Err(DecodedOutputsError::InvalidRound);
        };

        let decoded_outputs = outputs
            .into_iter()
            .map(|output| utxo::types::Output {
                amount: room.amount,
                owner: Address::from_slice(&output),
            })
            .collect::<Vec<utxo::types::Output>>();

        Ok(decoded_outputs)
    }

    pub async fn pass_outputs_signature(
        &self,
        participant_id: &U256,
        signature: Signature,
    ) -> Result<(), SignatureError<<S as storage::Storage>::InternalError>> {
        let tx = self.storage.transaction().await?;

        let participant = tx
            .get_participant(participant_id)
            .await
            .map_err(GetParticipantError::Storage)?
            .ok_or(GetParticipantError::NotFound)?;

        // check that previous status is DecodedOutputs
        match participant.status {
            ShuffleRound::DecodedOutputs(_) => {}
            _ => return Err(SignatureError::InvalidRound),
        };

        let room = tx
            .get_room(participant.room_id.as_ref().unwrap()) // FIXME:
            .await
            .map_err(GetRoomError::Storage)?
            .ok_or(GetRoomError::NotFound)?;

        if room.current_round != room.participants.len() {
            return Err(SignatureError::InvalidRound); // TODO
        }

        tx.update_participant_round(
            participant_id,
            ShuffleRound::SigningOutput(utxo::types::Input {
                signature: signature.as_bytes().to_vec().into(),
                id: *participant_id,
            }),
        )
        .await?;

        tx.commit().await?;

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
    #[error("one or more outputs are invalid")]
    InvalidOutputs,
}

#[derive(thiserror::Error, Debug)]
pub enum SignatureError<SE>
where
    SE: std::error::Error,
{
    #[error("failed to get encoded outputs: {0}")]
    GetEncodedOutputs(#[from] EncodingOutputsError<SE>),
    #[error("internal storage error: {0}")]
    Storage(#[from] SE),
    #[error("failed to get room: {0}")]
    GetRoom(#[from] GetRoomError<SE>),
    #[error("failed to get participant: {0}")]
    GetParticipant(#[from] GetParticipantError<SE>),
    #[error("no participant with given id in the room")]
    NoParticipantInRoom,
    #[error("invalid round")]
    InvalidRound,
    #[error("failed to update participant round: {0}")]
    UpdateParticipantRound(#[from] participants::UpdateError<SE>),
}

impl<S, W, C> Service<S, W, C>
where
    S: storage::Storage,
    W: waiter::Waiter,
    C: utxo::Contract,
{
    pub async fn send_transaction(
        &self,
        room_id: &uuid::Uuid,
    ) -> Result<Hash, SendTransactionError<<S as storage::Storage>::InternalError, C::Error>> {
        let outputs = self.decoded_outputs(room_id).await?;

        let tx = self.storage.transaction().await?;

        let room = tx
            .get_room(room_id)
            .await
            .map_err(GetRoomError::Storage)?
            .ok_or(GetRoomError::NotFound)?;

        if room.current_round != room.participants.len() {
            return Err(SendTransactionError::InvalidRound);
        }

        let mut inputs = Vec::with_capacity(room.participants.len());

        for participant_id in room.participants.iter() {
            let participant = tx
                .get_participant(participant_id)
                .await
                .map_err(GetParticipantError::Storage)?
                .ok_or(GetParticipantError::NotFound)?;

            let ShuffleRound::SigningOutput(input) = participant.status else {
                return Err(SendTransactionError::InvalidRound);
            };

            inputs.push(input);
        }

        let hash = self
            .utxo_contract
            .transfer(inputs, outputs)
            .await
            .map_err(SendTransactionError::Transfer)?;

        for participant_id in room.participants {
            tx.update_participant_round(&participant_id, ShuffleRound::Finish(hash))
                .await?;
        }

        tx.commit().await?;

        Ok(hash)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SendTransactionError<SE, CE>
where
    SE: std::error::Error,
    CE: std::error::Error,
{
    #[error("failed to get decoded outputs: {0}")]
    GetDecodedOutputs(#[from] DecodedOutputsError<SE>),
    #[error("internal storage error: {0}")]
    Storage(#[from] SE),
    #[error("failed to get room: {0}")]
    GetRoom(#[from] GetRoomError<SE>),
    #[error("failed to get participant: {0}")]
    GetParticipant(#[from] GetParticipantError<SE>),
    #[error("invalid round")]
    InvalidRound,
    #[error("failed to send transaction: {0}")]
    Transfer(CE),
    #[error("failed to update participant round: {0}")]
    UpdateParticipantRound(#[from] participants::UpdateError<SE>),
}

impl<S, W, C> Service<S, W, C>
where
    S: storage::Storage,
    W: waiter::Waiter,
    C: utxo::Contract,
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

    /// check that all participants in the room passed signed inputs with outputs
    pub async fn is_signature_passed(
        &self,
        room_id: &uuid::Uuid,
    ) -> Result<bool, SignatureError<<S as storage::Storage>::InternalError>> {
        let tx = self.storage.transaction().await?;

        let room = tx
            .get_room(room_id)
            .await
            .map_err(GetRoomError::Storage)?
            .ok_or(GetRoomError::NotFound)?;

        if room.current_round != room.participants.len() {
            return Ok(false);
        }

        for participant_id in room.participants.iter() {
            let participant = tx
                .get_participant(participant_id)
                .await
                .map_err(GetParticipantError::Storage)?
                .ok_or(GetParticipantError::NotFound)?;

            let ShuffleRound::SigningOutput(_) = participant.status else {
                return Ok(false);
            };
        }

        Ok(true)
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
