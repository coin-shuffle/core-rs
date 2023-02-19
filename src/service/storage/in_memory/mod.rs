pub mod participants;
pub mod queues;
pub mod rooms;
pub mod transaction;

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use ethers_core::types::{Address, U256};
use tokio::sync::Mutex;

use crate::service::types::{Participant, Room};

use self::transaction::InMemoryTransaction;

use super::{Transaction, TransactionGuard};

pub type QueueKey = (Address, U256);

#[derive(Clone, Default)]
pub struct MapStorage {
    queues: Arc<Mutex<HashMap<QueueKey, Vec<U256>>>>,
    rooms: Arc<Mutex<HashMap<uuid::Uuid, Room>>>,
    participants: Arc<Mutex<HashMap<U256, Participant>>>,
}

impl MapStorage {
    async fn merge(&self, other: &Self) {
        let mut queues = self.queues.lock().await;
        let mut rooms = self.rooms.lock().await;
        let mut participants = self.participants.lock().await;

        for (key, queue) in other.queues.lock().await.iter() {
            queues.insert(*key, queue.clone());
        }

        for (key, room) in other.rooms.lock().await.iter() {
            rooms.insert(*key, room.clone());
        }

        for (key, participant) in other.participants.lock().await.iter() {
            participants.insert(*key, participant.clone());
        }
    }

    async fn copy(&self) -> Self {
        Self {
            queues: Arc::new(Mutex::new(self.queues.lock().await.clone())),
            rooms: Arc::new(Mutex::new(self.rooms.lock().await.clone())),
            participants: Arc::new(Mutex::new(self.participants.lock().await.clone())),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum InternalError {}

#[async_trait]
impl super::Storage for MapStorage {
    type InternalError = InternalError;
    type Transaction = InMemoryTransaction;

    async fn transaction(
        &self,
    ) -> Result<TransactionGuard<Self::Transaction>, <Self as super::Storage>::InternalError> {
        Ok(TransactionGuard::new(InMemoryTransaction::new(self).await))
    }
}
