use ethers_core::abi::AbiError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
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
    #[error("Invalid status")]
    InvalidStatus,
    #[error("Invalid number of outputs")]
    InvalidNumberOfOutputs,
    #[error("Invalid number of participants")]
    InvalidNumberOfParticipants,
    #[error("Failed to create transfer")]
    Transfer(String),
    #[error("No RSA pub key")]
    NoRSAPubKey,
    #[error("failed to get decoded outputs: {0}")]
    GetDecodedOutputs(String),
    #[error("invalid outputs: {0}")]
    InvalidOutputs(AbiError),
}
