use coin_shuffle_contracts_bindings::shared_types::Utxo;
use ethers_core::types::U256;

use self::errors::Error;
use self::room::Room;
use crate::rsa::{EncryptionResult, Error as RSAError, RsaPrivateKey, RsaPublicKey};
use crate::types::EncodedOutput;
use crate::{node::storage::memory, rsa};

pub mod errors;
pub mod room;
pub mod storage;

#[derive(Debug, Clone)]
pub struct Node {
    room_storage: memory::RoomStorage,
}

impl Node {
    pub fn new() -> Self {
        Self {
            room_storage: memory::RoomStorage::new(),
        }
    }

    /// Initiate new **shuffle** process with specified UTXO.
    pub async fn add_session(
        &mut self,
        utxo: Utxo,
        output: Vec<u8>,
        rsa_private_key: RsaPrivateKey,
    ) -> Result<Room, Error> {
        let room = Room::new(utxo, rsa_private_key, output);

        self.room_storage.insert(&room).await?;

        Ok(room)
    }

    /// Add keys of other participants to the **shuffle** process.
    ///
    /// They will be used to encrypt the output of the current participant.
    pub async fn add_participants_keys(
        &mut self,
        utxo_id: U256,
        public_keys: Vec<RsaPublicKey>,
    ) -> Result<(), Error> {
        if let Some(mut room_inner) = self.room_storage.get(&utxo_id).await {
            room_inner.public_keys = public_keys;

            self.room_storage.update(&room_inner).await?;
        }

        Ok(())
    }

    /// Perform **shuffle** round.
    ///
    /// Receive encoded outputs from previous participants, decode them with RSA
    /// private key and of current participant and encode them with RSA public keys
    /// of next participants.
    pub async fn shuffle_round(
        &mut self,
        encoded_outputs: Vec<EncodedOutput>,
        utxo_id: U256,
    ) -> Result<Vec<EncodedOutput>, Error> {
        // TODO(OmegaTymbJIep): validate encoded outputs size

        let room = self
            .room_storage
            .get(&utxo_id)
            .await
            .ok_or(Error::RoomDoesntExist(utxo_id))?;

        // room.participants_number = encoded_outputs.len() + room.public_keys.len() + 1;
        self.room_storage.update(&room.clone()).await?;

        // Decode outputs of other participants.
        let mut outputs = encoded_outputs
            .into_iter()
            .map(|encoded_output| rsa::decode_by_chunks(encoded_output, &room.rsa_private_key))
            .collect::<Result<Vec<Vec<u8>>, RSAError>>()
            .map_err(Error::DecodeByChunks)?;

        // Add encoded output of current participant.
        let mut last_nonce = Vec::<u8>::new();
        let mut encoded_self_output = room.output;

        for public_key in room.public_keys {
            let EncryptionResult { nonce, encoded_msg } =
                rsa::encode_by_chunks(encoded_self_output.clone(), public_key, last_nonce.clone())
                    .map_err(Error::EncodeByChunks)?;

            last_nonce = nonce;
            encoded_self_output = encoded_msg;
        }

        outputs.push(encoded_self_output);

        Ok(outputs)
    }
}
