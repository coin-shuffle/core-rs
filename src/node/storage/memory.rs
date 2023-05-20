use ethers_core::types::U256;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::node::room::Room;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("utxo with this key already presented: {0}")]
    UtxoAlreadyPresented(U256),
    #[error("utxo with this key is not presented: {0}")]
    UtxoIsNotPresented(U256),
}

/// Default realization of the Node's RoomStorage
#[derive(Debug, Default, Clone)]
pub struct RoomStorage {
    room_list: Arc<Mutex<HashMap<U256, Room>>>,
}

impl RoomStorage {
    pub fn new() -> Self {
        Self {
            room_list: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn insert(&mut self, room: &Room) -> Result<(), Error> {
        let mut storage = self.room_list.lock().await;

        if storage.contains_key(&room.utxo.id) {
            return Err(Error::UtxoAlreadyPresented(room.utxo.id));
        }

        storage.insert(room.utxo.id, room.clone());

        Ok(())
    }

    pub async fn update(&mut self, room: &Room) -> Result<(), Error> {
        let mut storage = self.room_list.lock().await;

        if !storage.contains_key(&room.utxo.id) {
            return Err(Error::UtxoIsNotPresented(room.utxo.id));
        }

        storage.insert(room.utxo.id, room.clone());

        Ok(())
    }

    pub async fn get(&self, utxo_id: &U256) -> Option<Room> {
        let storage = self.room_list.lock().await;

        storage.get(utxo_id).cloned()
    }

    pub async fn remove(&mut self, utxo_id: &U256) -> Option<Room> {
        let mut storage = self.room_list.lock().await;

        storage.remove(utxo_id)
    }
}
