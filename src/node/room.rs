use rsa::{RsaPrivateKey, RsaPublicKey};

use coin_shuffle_contracts_bindings::shared_types::Utxo;

#[derive(Debug, Clone)]
pub struct Room {
    pub utxo: Utxo,
    pub output: Vec<u8>,
    pub public_keys: Vec<RsaPublicKey>,
    pub rsa_private_key: RsaPrivateKey,
}

impl Room {
    pub fn new(utxo: Utxo, rsa_private_key: RsaPrivateKey, output: Vec<u8>) -> Self {
        Self {
            utxo,
            output,
            rsa_private_key,
            public_keys: Vec::new(),
        }
    }
}
