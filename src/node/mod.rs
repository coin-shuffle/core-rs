use crate::rsa::{Error as RSAError, RsaPrivateKey, RsaPublicKey};
use ethers::core::types::U256;
use ethers::signers::{LocalWallet, Signer, WalletError};

use self::{room::Room, storage::Outputs};
use crate::{node::storage::RoomStorage, rsa, utxo_connector::Connector};

pub mod room;
pub mod storage;

#[derive(thiserror::Error, Debug)]
pub enum Error<E, R>
where
    E: std::error::Error,
    R: std::error::Error,
{
    #[error("utxo doesn't exist id: {0}")]
    UtxoDoesntExist(U256),
    #[error("invalid owner: {0}")]
    InvalidOwner(String),
    #[error("utxo connector error: {0}")]
    UtxoConnector(E),
    #[error("failed to get room: {0}")]
    FailedToGetRoom(R),
    #[error("failed to insert room: {0}")]
    FailedToInsertRoom(R),
    #[error("failed to update room: {0}")]
    FailedToUpdateRoom(R),
    #[error("room with specified UTXO doesn't exist utxo_id: {0}")]
    RoomDoesntExist(U256),
    #[error("failed to decode by chanks: {0}")]
    FailedToDecodeByChanks(RSAError),
    #[error("failed to encode by chanks: {0}")]
    FailedToEncodeByChanks(RSAError),
    #[error("incorrect signing data: incorrect outputs size")]
    IncorrectOutputsSize,
    #[error("incorrect signing data: self outputs is absent")]
    SelfOutputsIsAbsent,
    #[error("failed to sing the message: {0}")]
    FailedToSignMessage(#[from] WalletError),
}

#[derive(Debug, Clone)]
pub struct ShuffleRoundResult {
    pub outputs: Outputs,
    pub nonce: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Node<R: RoomStorage, C: Connector> {
    room_storage: R,
    utxo_conn: C,
}

impl<R, C> Node<R, C>
where
    R: RoomStorage,
    C: Connector,
{
    pub fn new(room_storage: R, utxo_conn: C) -> Self {
        Self {
            room_storage,
            utxo_conn,
        }
    }

    pub async fn init_room(
        &mut self,
        utxo_id: U256,
        output: Vec<u8>,
        rsa_private_key: RsaPrivateKey,
        ecdsa_private_key: LocalWallet,
    ) -> Result<Room, Error<C::Error, R::Error>> {
        let utxo = self
            .utxo_conn
            .get_utxo_by_id(utxo_id)
            .await
            .map_err(Error::UtxoConnector)?
            .ok_or(Error::UtxoDoesntExist(utxo_id))?;

        let room = Room::new(utxo, rsa_private_key, ecdsa_private_key, output);

        self.room_storage
            .insert(&room)
            .await
            .map_err(Error::FailedToInsertRoom)?;

        Ok(room)
    }

    pub async fn start_shuffle(
        &mut self,
        public_keys: Vec<RsaPublicKey>,
        utxo_id: U256,
    ) -> Result<(), Error<C::Error, R::Error>> {
        if let Some(mut room_inner) = self
            .room_storage
            .get(&utxo_id)
            .await
            .map_err(Error::FailedToGetRoom)?
        {
            room_inner.public_keys = public_keys;

            self.room_storage
                .update(&room_inner)
                .await
                .map_err(Error::FailedToUpdateRoom)?;
        }

        Ok(())
    }

    pub async fn shuffle_round(
        &mut self,
        encoded_outputs: Outputs,
        utxo_id: U256,
    ) -> Result<Outputs, Error<C::Error, R::Error>> {
        //
        // todo validate encoded outputs size
        //

        let mut result_outputs = Outputs::default();

        let mut room = self
            .room_storage
            .get(&utxo_id)
            .await
            .map_err(Error::FailedToGetRoom)?
            .ok_or(Error::RoomDoesntExist(utxo_id))?;

        room.participants_number = encoded_outputs.len() + room.public_keys.len() + 1;
        self.room_storage
            .update(&room.clone())
            .await
            .map_err(Error::FailedToUpdateRoom)?;

        for encoded_output in encoded_outputs {
            result_outputs.push(
                rsa::decode_by_chanks(encoded_output, room.clone().rsa_private_key)
                    .map_err(Error::FailedToDecodeByChanks)?,
            );
        }

        let mut nonce = Vec::<u8>::new();
        let mut encoded_self_output = room.output;
        for public_key in room.public_keys {
            let encoding_result =
                rsa::encode_by_chanks(encoded_self_output.clone(), public_key, nonce.clone())
                    .map_err(Error::FailedToEncodeByChanks)?;

            nonce = encoding_result.nonce;
            encoded_self_output = encoding_result.encoded_msg;
        }

        result_outputs.push(encoded_self_output);

        Ok(result_outputs)
    }

    pub async fn sign_tx(
        &self,
        utxo_id: U256,
        outputs: Outputs,
    ) -> Result<Vec<u8>, Error<C::Error, R::Error>> {
        let room = self
            .room_storage
            .get(&utxo_id)
            .await
            .map_err(Error::FailedToGetRoom)?
            .ok_or(Error::RoomDoesntExist(utxo_id))?;

        if room.participants_number != outputs.len() {
            Err(Error::IncorrectOutputsSize)?
        }

        if outputs.iter().position(|o| o == &room.output).is_none() {
            Err(Error::IncorrectOutputsSize)?;
        };

        let mut sign_message = room.utxo.id.to_string().as_bytes().to_vec();

        for mut output in outputs.clone() {
            sign_message.append(&mut output)
        }

        Ok(room
            .ecdsa_private_key
            .sign_message(sign_message)
            .await?
            .to_vec())
    }
}
