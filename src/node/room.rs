use rsa::{RsaPrivateKey, RsaPublicKey};

use super::Signer;
use crate::types::ShuffleStatus;
use coin_shuffle_contracts_bindings::utxo::types::Utxo;

// todo #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Room<S: Signer + Clone + Send + Sync + Send> {
    pub utxo: Utxo,
    pub output: Vec<u8>,
    pub public_keys: Vec<RsaPublicKey>,
    pub status: ShuffleStatus,
    pub rsa_private_key: RsaPrivateKey,
    pub signer: S,
    pub participants_number: usize,
}

impl<S: Signer + Clone + Send + Sync> Room<S> {
    pub fn new(utxo: Utxo, rsa_private_key: RsaPrivateKey, signer: S, output: Vec<u8>) -> Self {
        Self {
            utxo,
            output,
            status: ShuffleStatus::SearchParticipants,
            rsa_private_key,
            signer,
            public_keys: Vec::new(),
            participants_number: usize::default(),
        }
    }
}
