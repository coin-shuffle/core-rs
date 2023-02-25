use super::storage;
use super::waiter;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Storage error: {0}")]
    Storage(#[from] storage::Error),
    #[error("Participant not found")]
    ParticipantNotFound,
    #[error("Participant already in room")]
    ParticipantAlreadyInRoom,
    #[error("Participant not in room")]
    ParticipantNotInRoom,
    #[error("Room not found")]
    RoomNotFound,
    #[error("Invalid round")]
    InvalidRound,
    #[error("Invalid number of outputs")]
    InvalidNumberOfOutputs,
    #[error("Invalid number of participants")]
    InvalidNumberOfParticipants,
    #[error("Failed to create transfer")]
    Transfer(String),
    #[error("Waiter error: {0}")]
    Waiter(#[from] waiter::Error),
}