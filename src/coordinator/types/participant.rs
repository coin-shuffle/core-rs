use coin_shuffle_contracts_bindings::shared_types::Input;
use ethers_core::types::U256;
use rsa::RsaPublicKey;
use uuid::Uuid;

use crate::types::EncryptedOutput;

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    /// Participant havn't started the process of shuffle, but room is created.
    Wait,
    /// Shuffle started, the participant receiving RSA public
    /// keys, that are required for shuffle process.
    Start(RsaPublicKey),
    /// Decoded by participant outputs.
    DecryptedOutputs(Vec<EncryptedOutput>),
    /// Participant signs the decoded outputs and his input
    SigningOutput(Input),
    /// Participant finished the process of shuffle
    Finish,
}

#[derive(Debug, Clone)]
pub struct Participant {
    pub room_id: uuid::Uuid,
    pub utxo_id: U256,
    pub state: State,
}

impl Participant {
    pub fn new(utxo_id: U256, room_id: Uuid) -> Self {
        Self {
            room_id,
            utxo_id,
            state: State::Wait,
        }
    }
}
