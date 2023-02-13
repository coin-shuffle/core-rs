use ethers_core::types::U256;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use super::room::Room;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("utxo with this key already presented: {0}")]
    UtxoAlreadyPresented(U256),
    #[error("utxo with this key is not presented: {0}")]
    UtxoIsNotPresented(U256),
}

/// Storage that is required for the Room storing
#[async_trait::async_trait]
pub trait RoomStorage {
    type Error: std::error::Error;

    async fn insert(&mut self, room: &Room) -> Result<(), Self::Error>;
    async fn update(&mut self, room: &Room) -> Result<(), Self::Error>;
    async fn get(&self, utxo_id: &U256) -> Result<Option<Room>, Self::Error>;
    async fn remove(&mut self, utxo_id: &U256) -> Result<Option<Room>, Self::Error>;
}

/// Defaul realization of the Node's RoomStorage
pub struct RoomMemoryStorage {
    room_list: Arc<Mutex<HashMap<U256, Room>>>,
}

#[async_trait::async_trait]
impl RoomStorage for RoomMemoryStorage {
    type Error = Error;

    async fn insert(&mut self, room: &Room) -> Result<(), Self::Error> {
        let mut storage = self.room_list.lock().await;

        if storage.contains_key(&room.utxo.id) {
            return Err(Error::UtxoAlreadyPresented(room.utxo.id));
        }

        storage.insert(room.utxo.id, room.clone());

        Ok(())
    }

    async fn update(&mut self, room: &Room) -> Result<(), Self::Error> {
        let mut storage = self.room_list.lock().await;

        if !storage.contains_key(&room.utxo.id) {
            return Err(Error::UtxoIsNotPresented(room.utxo.id));
        }

        storage.insert(room.utxo.id, room.clone());

        Ok(())
    }

    async fn get(&self, utxo_id: &U256) -> Result<Option<Room>, Self::Error> {
        let storage = self.room_list.lock().await;

        Ok(match storage.get(utxo_id) {
            Some(room) => Some(room.clone()),
            None => None,
        })
    }

    async fn remove(&mut self, utxo_id: &U256) -> Result<Option<Room>, Self::Error> {
        let mut storage = self.room_list.lock().await;

        Ok(storage.remove(&utxo_id))
    }
}
