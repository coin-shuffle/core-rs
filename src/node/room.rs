use ethers_signers::LocalWallet;
use rsa::{RsaPrivateKey, RsaPublicKey};

use crate::types::ShuffleStatus;
use crate::utxo_connector::types::Utxo;

// todo #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Room {
    pub utxo: Utxo,
    pub output: String,
    pub serial_number: usize,
    pub public_keys: Vec<RsaPublicKey>,
    pub status: ShuffleStatus,
    pub rsa_private_key: RsaPrivateKey,
    pub ecdsa_private_key: LocalWallet,
}

impl Room {
    pub fn new(
        utxo: Utxo,
        rsa_private_key: RsaPrivateKey,
        ecdsa_private_key: LocalWallet,
        output: String,
    ) -> Self {
        Self {
            utxo,
            output,
            status: ShuffleStatus::SearchParticipants,
            rsa_private_key,
            ecdsa_private_key,
            public_keys: Vec::new(),
            serial_number: usize::default(),
        }
    }
}
