use crate::node::signer::Signer;
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

pub type Output = Vec<u8>;
pub type Outputs = Vec<Output>;

/// Storage that is required for the Room storing
#[async_trait::async_trait]
pub trait RoomStorage<S: Signer + Clone + Send + Sync> {
    type Error: std::error::Error;

    async fn insert(&mut self, room: &Room<S>) -> Result<(), Self::Error>;
    async fn update(&mut self, room: &Room<S>) -> Result<(), Self::Error>;
    async fn get(&self, utxo_id: &U256) -> Result<Option<Room<S>>, Self::Error>;
    async fn remove(&mut self, utxo_id: &U256) -> Result<Option<Room<S>>, Self::Error>;
}

/// Default realization of the Node's RoomStorage
#[derive(Debug, Default, Clone)]
pub struct RoomMemoryStorage<S: Signer + Clone + Send + Sync> {
    room_list: Arc<Mutex<HashMap<U256, Room<S>>>>,
}

impl<S: Signer + Clone + Send + Sync> RoomMemoryStorage<S> {
    pub fn new() -> Self {
        Self {
            room_list: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl<S: Signer + Clone + Send + Sync> RoomStorage<S> for RoomMemoryStorage<S> {
    type Error = Error;

    async fn insert(&mut self, room: &Room<S>) -> Result<(), Self::Error> {
        let mut storage = self.room_list.lock().await;

        if storage.contains_key(&room.utxo.id) {
            return Err(Error::UtxoAlreadyPresented(room.utxo.id));
        }

        storage.insert(room.utxo.id, room.clone());

        Ok(())
    }

    async fn update(&mut self, room: &Room<S>) -> Result<(), Self::Error> {
        let mut storage = self.room_list.lock().await;

        if !storage.contains_key(&room.utxo.id) {
            return Err(Error::UtxoIsNotPresented(room.utxo.id));
        }

        storage.insert(room.utxo.id, room.clone());

        Ok(())
    }

    async fn get(&self, utxo_id: &U256) -> Result<Option<Room<S>>, Self::Error> {
        let storage = self.room_list.lock().await;

        Ok(storage.get(utxo_id).cloned())
    }

    async fn remove(&mut self, utxo_id: &U256) -> Result<Option<Room<S>>, Self::Error> {
        let mut storage = self.room_list.lock().await;

        Ok(storage.remove(utxo_id))
    }
}
