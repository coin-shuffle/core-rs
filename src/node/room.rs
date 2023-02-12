use ethers_core::types::U256;
use ethers_signers::LocalWallet;
use rsa::{RsaPrivateKey, RsaPublicKey};

use crate::types::ShuffleStatus;
use crate::utxo_connector::{types::Utxo, Connector};

#[derive(thiserror::Error, Debug)]
pub enum Error<E: std::error::Error> {
    #[error("utxo doesn't exist id: {0}")]
    UtxoDoesntExist(U256),
    #[error("invalid owner: {0}")]
    InvalidOwner(String),
    #[error("utxo connector error: {0}")]
    UtxoConnector(E),
}

// todo #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Room {
    pub utxo: Utxo,
    pub public_keys: Vec<RsaPublicKey>,
    pub status: ShuffleStatus,
    pub rsa_private_key: RsaPrivateKey,
    pub ecdsa_private_key: LocalWallet,
}

impl Room {
    async fn new<C: Connector + 'static>(
        utxo_conn: C,
        utxo_id: U256,
        rsa_private_key: RsaPrivateKey,
        ecdsa_private_key: LocalWallet,
    ) -> Result<Self, Error<C::Error>> {
        let utxo = utxo_conn
            .get_utxo_by_id(utxo_id)
            .await
            .map_err(Error::UtxoConnector)?
            .ok_or(Error::UtxoDoesntExist(utxo_id))?;

        if ecdsa_private_key
            .signer()
            .verifying_key()
            .to_bytes()
            .as_slice()
            != utxo.owner.as_bytes()
        {
           return Err(Error::InvalidOwner(utxo.owner.to_string()))
        }

        Ok(Self {
            utxo,
            public_keys: Vec::new(),
            status: ShuffleStatus::SearchParticipants,
            rsa_private_key,
            ecdsa_private_key,
        })
    }
}
