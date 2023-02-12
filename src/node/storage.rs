use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use ethers_core::types::U256;

use super::room::Room;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("utxo with this key already presented")]
    UtxoAlreadyPresented(),
}

/// Storage that is required for the Room storing
#[async_trait::async_trait]
pub trait RoomStorage {
    type Error: std::error::Error;
    
    async fn save_room(&mut self, room: &Room) -> Result<(), Self::Error>;
    async fn get_room(&self, utxo_id: &U256) -> Result<Option<Room>, Self::Error>; 
    async fn delete_room(&mut self, utxo_id: &U256) -> Result<Option<Room>, Self::Error>;
}

/// Defaul realization of the Node's RoomStorage
pub struct RoomMemoryStorage {
    room_list: Arc<Mutex<HashMap<U256, Room>>>
}

#[async_trait::async_trait]
impl RoomStorage for RoomMemoryStorage {
    type Error = Error;

    async fn save_room(&mut self, room: &Room) -> Result<(), Self::Error> {
        let mut storage = self.room_list.lock().await;

        if storage.contains_key(&room.utxo.id) {
            return Err(Error::UtxoAlreadyPresented())
        }

        storage.insert(room.utxo.id, room.clone());

        Ok(())
    }

    async fn get_room(&self, utxo_id: &U256) -> Result<Option<Room>, Self::Error> {
        let storage = self.room_list.lock().await;

        Ok(match storage.get(utxo_id) {
            Some(room) => Some(room.clone()),
            None => None
        })
    }

    async fn delete_room(&mut self, utxo_id: &U256) -> Result<Option<Room>, Self::Error> {
        let mut storage = self.room_list.lock().await;

        Ok(storage.remove(&utxo_id))
    }
}