use coin_shuffle_contracts_bindings::utxo::types::Input;
use ethers_core::{abi::Hash, types::U256};
use rsa::RsaPublicKey;

use super::EncodedOutput;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum ShuffleRound {
    /// Participant added to queue before shuffle started.
    Wait,
    /// Shuffle started, the participant receiving RSA public
    /// keys, that are required for shuffle process.
    Start(Vec<RsaPublicKey>),
    /// Decoded by participant outputs.
    DecodedOutputs(Vec<EncodedOutput>),
    /// Participant signs the decoded outputs and his input
    SigningOutput(Input),
    /// Participant received the transaction hash
    Finish(Hash),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Participant {
    pub room_id: Option<uuid::Uuid>,
    pub utxo_id: U256,
    pub rsa_pubkey: RsaPublicKey,
    pub status: ShuffleRound,
}

impl Participant {
    pub fn new(utxo_id: U256, pubkey: RsaPublicKey) -> Self {
        Self {
            room_id: None, // because participant haven't entered room yet
            utxo_id,
            rsa_pubkey: pubkey,
            status: ShuffleRound::Wait,
        }
    }
}
