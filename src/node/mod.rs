use ethers_core::types::U256;
use ethers_signers::LocalWallet;
use rsa::{RsaPrivateKey, RsaPublicKey};

use crate::{node::storage::RoomStorage, utxo_connector::Connector};

use self::room::Room;

mod room;
mod storage;

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
}

#[derive(Debug, Clone)]
pub struct Node<R: RoomStorage, C: Connector> {
    shuffle_service_addr: String,
    room_storage: R,
    utxo_conn: C,
}

impl<R, C> Node<R, C>
where
    R: RoomStorage,
    C: Connector,
{
    pub fn new(shuffle_service_addr: String, room_storage: R, utxo_conn: C) -> Self {
        Self {
            shuffle_service_addr,
            room_storage,
            utxo_conn,
        }
    }

    pub async fn init_room(
        &mut self,
        utxo_id: U256,
        output: String,
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
        serial_number: usize,
        utxo_id: U256,
    ) -> Result<(), Error<C::Error, R::Error>> {
        if let Some(mut room_inner) = self
            .room_storage
            .get(&utxo_id)
            .await
            .map_err(Error::FailedToGetRoom)?
        {
            room_inner.public_keys = public_keys;
            room_inner.serial_number = serial_number;

            self.room_storage
                .update(&room_inner)
                .await
                .map_err(Error::FailedToUpdateRoom)?;
        }

        Ok(())
    }
}
