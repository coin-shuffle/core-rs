pub mod error;
pub mod storage;
pub mod types;
pub mod waiter;

use coin_shuffle_contracts_bindings::utxo;
use ethers_core::{
    abi::{ethereum_types::Signature, Hash},
    types::{Address, U256},
};
use rsa::RsaPublicKey;

use self::error::Error;
use self::types::{participant::Participant, room::Room, EncodedOutput, ShuffleRound};

pub type Result<T> = std::result::Result<T, Error>;

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
    ) -> Result<()> {
        self.storage
            .insert_participant(participant.clone())
            .await
            .map_err(|e| Error::Storage(e.into()))?;

        self.waiter
            .add_to_queue(token, amount, &participant.utxo_id)
            .await
            .map_err(Error::Waiter)?;

        Ok(())
    }

    /// Remove participants from the queue for given token and amount
    /// and form rooms for all of them. Return id's of newly created rooms.
    pub async fn create_rooms(&self, token: &Address, amount: &U256) -> Result<Vec<Room>> {
        let rooms = self.waiter.organize(token, amount).await?;

        Ok(rooms)
    }

    /// Return keys that are needed to decrypt and encrypt the message for given room and participant.
    pub async fn participant_keys(&self, participant_id: &U256) -> Result<Vec<RsaPublicKey>> {
        let tx = self.storage.transaction().await?;

        let (room, _participant) = Self::room_and_participant(&tx, participant_id).await?;

        let position = Self::participant_position(&room, participant_id)?;

        let mut keys = Vec::with_capacity(room.participants.len() - position);

        for participant_id in room.participants.iter().skip(position) {
            let participant = Self::participant_by_id(&tx, participant_id).await?;

            keys.push(participant.rsa_pubkey.clone());
        }

        tx.update_participant_round(participant_id, ShuffleRound::Start(keys.clone()))
            .await
            .map_err(|e| Error::Storage(e.into()))?;

        tx.commit().await.map_err(|e| Error::Storage(e.into()))?;

        Ok(keys)
    }

    /// Return outputs that given participant should decrypt.
    pub async fn encoded_outputs(&self, participant_id: &U256) -> Result<Vec<EncodedOutput>> {
        let tx = self.storage.transaction().await?;

        let (room, _participant) = Self::room_and_participant(&tx, participant_id).await?;

        let position = Self::participant_position(&room, participant_id)?;

        if position != room.current_round {
            return Err(Error::InvalidRound);
        }

        // Participant is the first one in the room, so he first adds his encoded outputs
        if position == 0 {
            return Ok(Vec::new());
        }

        let previous_participant_id = room.participants[position - 1];

        let previous_participant = Self::participant_by_id(&tx, &previous_participant_id).await?;

        // Get decoded outputs from the previous participant
        let ShuffleRound::DecodedOutputs(encoded_outputs) = previous_participant.status else {
            return Err(Error::InvalidRound);
        };

        tx.commit().await.map_err(|e| Error::Storage(e.into()))?;

        Ok(encoded_outputs)
    }

    pub async fn pass_decoded_outputs(
        &self,
        participant_id: &U256,
        decoded_outputs: Vec<EncodedOutput>,
    ) -> Result<()> {
        let tx = self.storage.transaction().await?;

        let (room, participant) = Self::room_and_participant(&tx, participant_id).await?;

        let position = Self::participant_position(&room, participant_id)?;

        if position != room.current_round {
            return Err(Error::InvalidRound);
        }
        if decoded_outputs.len() != (position + 1) {
            return Err(Error::InvalidNumberOfOutputs);
        }

        // check that previous status is Start
        match participant.status {
            ShuffleRound::Start(_) => {}
            _ => return Err(Error::InvalidRound),
        };

        tx.update_room_round(&room.id, room.current_round + 1)
            .await
            .map_err(|e| Error::Storage(e.into()))?;

        tx.update_participant_round(
            participant_id,
            ShuffleRound::DecodedOutputs(decoded_outputs),
        )
        .await
        .map_err(|e| Error::Storage(e.into()))?;

        tx.commit().await.map_err(|e| Error::Storage(e.into()))?;

        Ok(())
    }

    /// If shuffle is finished, return all outputs that each participant should sign.
    pub async fn decoded_outputs(&self, room_id: &uuid::Uuid) -> Result<Vec<utxo::types::Output>> {
        let tx = self.storage.transaction().await?;

        let decoded_outputs = Self::get_decoded_outputs(&tx, room_id).await?;

        tx.commit().await.map_err(|e| Error::Storage(e.into()))?;

        Ok(decoded_outputs)
    }

    async fn get_decoded_outputs(
        storage: &S,
        room_id: &uuid::Uuid,
    ) -> Result<Vec<utxo::types::Output>> {
        let room = Self::room_by_id(storage, room_id).await?;

        if room.current_round != room.participants.len() {
            return Err(Error::InvalidRound);
        }

        let last_participant = Self::participant_by_id(
            storage,
            room.participants.last().expect("room can't be empty"),
        )
        .await?;

        let ShuffleRound::DecodedOutputs(outputs) = last_participant.status else {
            return Err(Error::InvalidRound);
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
    ) -> Result<()> {
        let tx = self.storage.transaction().await?;

        let (room, participant) = Self::room_and_participant(&tx, participant_id).await?;

        // check that previous status is DecodedOutputs
        match participant.status {
            ShuffleRound::DecodedOutputs(_) => {}
            _ => return Err(Error::InvalidRound),
        };

        if room.current_round != room.participants.len() {
            return Err(Error::InvalidRound);
        }

        tx.update_participant_round(
            participant_id,
            ShuffleRound::SigningOutput(utxo::types::Input {
                signature: signature.as_bytes().to_vec().into(),
                id: *participant_id,
            }),
        )
        .await
        .map_err(|e| Error::Storage(e.into()))?;

        tx.commit().await.map_err(|e| Error::Storage(e.into()))?;

        Ok(())
    }

    pub async fn send_transaction(&self, room_id: &uuid::Uuid) -> Result<Hash> {
        let tx = self.storage.transaction().await?;

        let outputs = Self::get_decoded_outputs(&tx, room_id).await?;

        let room = Self::room_by_id(&tx, room_id).await?;

        if room.current_round != room.participants.len() {
            return Err(Error::InvalidRound);
        }

        let mut inputs = Vec::with_capacity(room.participants.len());

        for participant_id in room.participants.iter() {
            let participant = Self::participant_by_id(&tx, participant_id).await?;

            let ShuffleRound::SigningOutput(input) = participant.status else {
                return Err(Error::InvalidRound);
            };

            inputs.push(input);
        }

        let hash = self
            .utxo_contract
            .transfer(inputs, outputs)
            .await
            .map_err(|e| Error::Transfer(e.to_string()))?;

        for participant_id in room.participants {
            tx.update_participant_round(&participant_id, ShuffleRound::Finish(hash))
                .await
                .map_err(|e| Error::Storage(e.into()))?;
        }

        tx.commit().await.map_err(|e| Error::Storage(e.into()))?;

        Ok(hash)
    }

    async fn participant_by_id(storage: &S, participant_id: &U256) -> Result<Participant> {
        storage
            .get_participant(participant_id)
            .await
            .map_err(|e| Error::Storage(e.into()))?
            .ok_or(Error::ParticipantNotFound)
    }

    pub async fn get_participant(&self, participant_id: &U256) -> Result<Participant> {
        Self::participant_by_id(&self.storage, participant_id).await
    }

    async fn room_by_id(storage: &S, room_id: &uuid::Uuid) -> Result<Room> {
        storage
            .get_room(room_id)
            .await
            .map_err(|e| Error::Storage(e.into()))?
            .ok_or(Error::RoomNotFound)
    }

    pub async fn get_room(&self, room_id: &uuid::Uuid) -> Result<Room> {
        Self::room_by_id(&self.storage, room_id).await
    }

    async fn room_and_participant(
        storage: &S,
        participant_id: &U256,
    ) -> Result<(Room, Participant)> {
        let participant = Self::participant_by_id(storage, participant_id).await?;

        let room_id = participant.room_id.ok_or(Error::ParticipantNotInRoom)?;

        let room = Self::room_by_id(storage, &room_id).await?;

        Ok((room, participant))
    }

    fn participant_position(room: &Room, participant_id: &U256) -> Result<usize> {
        room.participants
            .iter()
            .position(|id| id == participant_id)
            .ok_or(Error::ParticipantNotInRoom)
    }

    /// check that all participants in the room passed signed inputs with outputs
    pub async fn is_signature_passed(&self, room_id: &uuid::Uuid) -> Result<bool> {
        let tx = self.storage.transaction().await?;

        let room = Self::room_by_id(&tx, room_id).await?;

        if room.current_round != room.participants.len() {
            return Ok(false);
        }

        for participant_id in room.participants.iter() {
            let participant = Self::participant_by_id(&tx, participant_id).await?;

            let ShuffleRound::SigningOutput(_) = participant.status else {
                return Ok(false);
            };
        }

        Ok(true)
    }
}
