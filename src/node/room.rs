use rsa::{RsaPrivateKey, RsaPublicKey};

use crate::types::ShuffleStatus;
use coin_shuffle_contracts_bindings::shared_types::Utxo;

#[derive(Debug, Clone)]
pub struct Room {
    pub utxo: Utxo,
    pub output: Vec<u8>,
    pub public_keys: Vec<RsaPublicKey>,
    pub status: ShuffleStatus,
    pub rsa_private_key: RsaPrivateKey,
    pub participants_number: usize,
}

impl Room {
    pub fn new(utxo: Utxo, rsa_private_key: RsaPrivateKey, output: Vec<u8>) -> Self {
        Self {
            utxo,
            output,
            rsa_private_key,
            status: ShuffleStatus::SearchParticipants,
            public_keys: Vec::new(),
            participants_number: usize::default(),
        }
    }
}
