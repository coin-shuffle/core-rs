pub mod error;
mod storage;
pub mod types;

use std::collections::{BTreeSet, HashMap};

use coin_shuffle_contracts_bindings::shared_types::{Input, Output};
use ethers_core::abi::ethereum_types::Signature;
use ethers_core::types::{Address, Bytes, U256};
use rsa::RsaPublicKey;

use self::types::RoomState;
use self::types::{Participant, ParticipantState, Room};
use self::{error::Error, storage::memory};
use crate::types::EncodedOutput;

pub type CoordinatorResult<T> = std::result::Result<T, Error>;

/// Implementation of the coordinator that stores all the data in memory.
#[derive(Clone)]
pub struct Coordinator {
    storage: memory::CoordinatorStorage,
}

impl Default for Coordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl Coordinator {
    pub fn new() -> Self {
        Self {
            storage: memory::CoordinatorStorage::new(),
        }
    }

    /// Create room with given participants, where each participant is
    /// represented by his UTXO id, and return room.
    pub async fn create_room(&self, token: Address, amount: U256, participants: Vec<U256>) -> Room {
        let room = Room::new(token, amount, participants);

        self.storage.rooms().insert(room.clone()).await;

        for participant in room.participants.iter() {
            self.storage
                .participants()
                .insert(Participant::new(*participant, room.id))
                .await;
        }

        room
    }

    /// Connect participant to the room with passed RSA public key. If all
    /// participants are connected, then start the shuffling process and return
    /// the keys that are needed to decrypt and encrypt the message for given
    /// room and participant.
    pub async fn connect_participant(
        &self,
        participant_id: &U256,
        rsa_pubkey: RsaPublicKey,
    ) -> CoordinatorResult<Option<HashMap<U256, Vec<RsaPublicKey>>>> {
        let participant = self.participant_by_id(participant_id).await?;

        let room = self.room_by_id(&participant.room_id).await?;
        if !room.participants.contains(participant_id) {
            return Err(Error::ParticipantNotInRoom {
                participant: *participant_id,
                room: room.id,
            });
        }

        self.update_participant_state(participant_id, ParticipantState::Start(rsa_pubkey))
            .await;

        let connected = match room.state {
            RoomState::Waiting => {
                let mut connected = BTreeSet::new();
                connected.insert(*participant_id);
                connected
            }
            RoomState::Connecting(mut keys) => {
                keys.insert(*participant_id);
                keys
            }
            _ => {
                return Err(Error::InvalidRoomState {
                    room: room.id,
                    state: room.state,
                })
            }
        };

        if connected.len() == room.participants.len() {
            let keys = self.distribute_keys(room.participants).await?;

            self.update_room_state(&room.id, RoomState::Shuffle(0))
                .await;
            return Ok(Some(keys));
        }

        self.update_room_state(&room.id, RoomState::Connecting(connected))
            .await;

        Ok(None)
    }

    async fn update_room_state(&self, room_id: &uuid::Uuid, state: RoomState) {
        self.storage.rooms().update_state(*room_id, state).await;
    }

    async fn room_by_id(&self, room_id: &uuid::Uuid) -> CoordinatorResult<Room> {
        self.storage
            .rooms()
            .get(*room_id)
            .await
            .ok_or_else(|| Error::RoomNotFound { room_id: *room_id })
    }

    async fn participant_by_id(&self, participant_id: &U256) -> CoordinatorResult<Participant> {
        self.storage
            .participants()
            .get(*participant_id)
            .await
            .ok_or(Error::ParticipantNotFound(*participant_id))
    }

    /// Return a map of RSA public keys for each participant in the room.
    async fn distribute_keys(
        &self,
        participants: Vec<U256>,
    ) -> CoordinatorResult<HashMap<U256, Vec<RsaPublicKey>>> {
        let participants_keys = self
            .storage
            .participants()
            .get_many(&participants)
            .await
            .into_iter()
            .map(|p| {
                let ParticipantState::Start(key) = p.state else {
                    return Err(Error::InvalidParticipantState {
                        participant: p.utxo_id,
                        state: p.state,
                    });
                };
                Ok((p.utxo_id, key))
            })
            .collect::<CoordinatorResult<HashMap<U256, RsaPublicKey>>>()?;

        let mut keys = HashMap::new();

        for (position, utxo_id) in participants.iter().enumerate() {
            let keys_for_participant = participants
                .iter()
                // Skip participants that already have the key, `+1`
                // because we don't want to include the current participant
                .skip(position + 1)
                // Reverse to get the keys in order the participant should decrypt
                .rev()
                // Get the key for the participant
                .map(|utxo_id| {
                    participants_keys
                        .get(utxo_id)
                        .cloned()
                        .ok_or(Error::ParticipantNotFound(*utxo_id))
                })
                .collect::<CoordinatorResult<Vec<RsaPublicKey>>>()?;

            keys.insert(*utxo_id, keys_for_participant);
        }

        Ok(keys)
    }

    /// Return outputs that given participant should decrypt.
    pub async fn encoded_outputs(
        &self,
        participant_id: &U256,
    ) -> CoordinatorResult<Vec<EncodedOutput>> {
        let participant = self.participant_by_id(participant_id).await?;
        let room = self.room_by_id(&participant.room_id).await?;

        let position = Self::participant_position(&room, participant_id)?;
        // Participant is the first one in the room, so he first adds his encoded outputs
        if position == 0 {
            return Ok(Vec::new());
        }

        let previous_participant_id = room
            .participants
            .get(position - 1)
            .ok_or_else(|| Error::InvalidRound(position))?;
        let previous_participant = self.participant_by_id(previous_participant_id).await?;

        // Get decoded outputs of previous participant and send them to the current one
        let ParticipantState::DecodedOutputs(encoded_outputs) = previous_participant.state else {
            return Err(Error::InvalidParticipantState {
                participant: previous_participant.utxo_id,
                state: previous_participant.state,
            });
        };

        Ok(encoded_outputs)
    }

    /// Return position of the participant in the room.
    fn participant_position(room: &Room, participant_id: &U256) -> CoordinatorResult<usize> {
        room.participants
            .iter()
            .position(|id| id == participant_id)
            .ok_or(Error::ParticipantNotFound(*participant_id))
    }

    /// Path decoded by participant outputs and store them in the storage.
    ///
    /// If participant is the last one in the room, then return
    /// [`PassDecodedOutputsResult::Finished`]. Otherwise, return
    /// [`PassDecodedOutputsResult::Round`] with position of the next
    /// participant in the room.
    pub async fn pass_decoded_outputs(
        &self,
        participant_id: &U256,
        decoded_outputs: Vec<EncodedOutput>,
    ) -> CoordinatorResult<PassDecodedOutputsResult> {
        let participant = self.participant_by_id(participant_id).await?;
        let room = self.room_by_id(&participant.room_id).await?;

        let position = Self::participant_position(&room, participant_id)?;
        let RoomState::Shuffle(current_round) = room.state else {
            return Err(Error::InvalidRoomState {
                room: room.id,
                state: room.state,
            });
        };
        if position != current_round {
            return Err(Error::InvalidRound(position));
        }
        if decoded_outputs.len() != (position + 1) {
            return Err(Error::InvalidNumberOfOutputs);
        }

        // check that previous status is Start
        let ParticipantState::Start(_) = participant.state else {
            return Err(Error::InvalidParticipantState {
                participant: *participant_id,
                state: participant.state,
            });
        };

        // If participant is the last one in the room, then his outputs are output addresses
        let outputs = if position == room.participants.len() - 1 {
            let outputs = decoded_outputs
                .clone()
                .into_iter()
                .map(|o| Output {
                    amount: room.amount,
                    owner: Address::from_slice(&o),
                })
                .collect::<Vec<Output>>();
            self.update_room_state(
                &room.id,
                RoomState::Signatures((outputs.clone(), Vec::new())),
            )
            .await;
            PassDecodedOutputsResult::Finished(outputs)
        } else {
            let current_round = current_round + 1;
            self.update_room_state(&room.id, RoomState::Shuffle(current_round))
                .await;
            PassDecodedOutputsResult::Round(current_round)
        };

        self.update_participant_state(
            participant_id,
            ParticipantState::DecodedOutputs(decoded_outputs),
        )
        .await;

        Ok(outputs)
    }

    async fn update_participant_state(&self, participant_id: &U256, state: ParticipantState) {
        self.storage
            .participants()
            .update_state(*participant_id, state)
            .await;
    }

    /// Return outputs that given room should sign.
    pub async fn outputs_to_sign(&self, room_id: &uuid::Uuid) -> CoordinatorResult<Vec<Output>> {
        let room = self.room_by_id(room_id).await?;
        let RoomState::Signatures((outputs, _)) = room.state else {
            return Err(Error::InvalidRoomState {
                room: *room_id,
                state: room.state,
            });
        };
        Ok(outputs)
    }

    /// Pass signature of the output and store it in the storage.
    ///
    /// If all participants passed their signatures, return all inputs and outputs.
    pub async fn pass_signature(
        &self,
        room_id: &uuid::Uuid,
        participant_id: &U256,
        signature: Signature,
    ) -> CoordinatorResult<Option<(Vec<Output>, Vec<Input>)>> {
        let room = self.room_by_id(room_id).await?;
        let _position = Self::participant_position(&room, participant_id)?;

        let RoomState::Signatures((outputs, mut passed)) = room.state else {
            return Err(Error::InvalidRoomState {
                room: *room_id,
                state: room.state,
            });
        };

        // TODO(Velnbur): validate signature here

        let participant = self.participant_by_id(participant_id).await?;
        // check that previous status is Start
        let ParticipantState::DecodedOutputs(_) = participant.state else {
            return Err(Error::InvalidParticipantState {
                participant: *participant_id,
                state: participant.state,
            });
        };
        passed.push(*participant_id);

        let input = Input {
            id: participant.utxo_id,
            signature: Bytes::from(signature.as_bytes().to_vec()),
        };

        self.update_participant_state(participant_id, ParticipantState::SigningOutput(input))
            .await;

        let participants_passed = passed.len();

        self.update_room_state(&room.id, RoomState::Signatures((outputs.clone(), passed)))
            .await;

        if participants_passed != room.participants.len() {
            return Ok(None);
        }

        let mut inputs = Vec::new();
        for participant_id in room.participants {
            let participant = self.participant_by_id(&participant_id).await?;
            let ParticipantState::SigningOutput(input) = participant.state else {
                return Err(Error::InvalidParticipantState {
                    participant: participant_id,
                    state: participant.state,
                });
            };

            inputs.push(input);
        }

        Ok(Some((outputs, inputs)))
    }

    /// Get participant by id.
    pub async fn get_participant(&self, participant_id: &U256) -> Option<Participant> {
        self.storage.participants().get(*participant_id).await
    }

    /// Get room by id.
    pub async fn get_room(&self, room_id: &uuid::Uuid) -> Option<Room> {
        self.storage.rooms().get(*room_id).await
    }

    /// Clear room and participants from the storage.
    pub async fn clear_room(&self, room_id: &uuid::Uuid) {
        self.storage.clear_room(room_id).await;
    }
}

/// Result of the `pass_decoded_outputs` method.
pub enum PassDecodedOutputsResult {
    /// All participants decoded their outputs, so the next step is to sign them.
    Finished(Vec<Output>),
    /// Not all participants decoded their outputs, so the next step is to shuffle outputs.
    Round(usize),
}
