use ethers_core::types::U256;

use super::{ParticipantState, RoomState};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Participant not found: {0}")]
    ParticipantNotFound(U256),
    #[error("Participant not in the room, participant: {participant}, room: {room}")]
    ParticipantNotInRoom { participant: U256, room: uuid::Uuid },
    #[error("Room not found: {room_id}")]
    RoomNotFound { room_id: uuid::Uuid },
    #[error("Invalid round: {0}")]
    InvalidRound(usize),
    #[error("Room: {room} has invalid state: {state:?}")]
    InvalidRoomState { room: uuid::Uuid, state: RoomState },
    #[error("Participant: {participant} has invalid state: {state:?}")]
    InvalidParticipantState {
        participant: U256,
        state: ParticipantState,
    },
    #[error("Invalid number of outputs")]
    InvalidNumberOfOutputs,
}
