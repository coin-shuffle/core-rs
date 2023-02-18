use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use async_trait::async_trait;
use ethers_core::types::{Address, U256};
use tokio::sync::Mutex;

use crate::service::types::{Participant, Room, ShuffleRound};

use super::{
    participants::{self, UpdateError},
    queues, rooms,
};

pub type QueueKey = (Address, U256);

#[derive(Clone, Default)]
pub struct MapStorage {
    queues: Arc<Mutex<HashMap<QueueKey, Vec<U256>>>>,
    rooms: Arc<Mutex<HashMap<uuid::Uuid, Room>>>,
    participants: Arc<Mutex<HashMap<U256, Participant>>>,
}

#[derive(thiserror::Error, Debug)]
pub enum InternalError {}

#[async_trait]
impl rooms::Storage for MapStorage {
    type InternalError = InternalError;

    async fn get_room(&self, id: &uuid::Uuid) -> Result<Option<Room>, Self::InternalError> {
        let rooms = self.rooms.lock().await;

        Ok(rooms.get(id).cloned())
    }

    async fn update_room_round(
        &self,
        id: &uuid::Uuid,
        round: usize,
    ) -> Result<(), rooms::UpdateError<Self::InternalError>> {
        let mut rooms = self.rooms.lock().await;

        match rooms.get_mut(id) {
            Some(room) => {
                room.current_round = round;
                Ok(())
            }
            None => Err(rooms::UpdateError::NotFound),
        }
    }

    async fn insert_room(
        &self,
        room: &Room,
    ) -> Result<(), rooms::InsertError<Self::InternalError>> {
        let mut rooms = self.rooms.lock().await;

        rooms.insert(room.id, room.clone());

        Ok(())
    }
}

#[async_trait]
impl participants::Storage for MapStorage {
    type InternalError = InternalError;

    async fn insert_participants(
        &self,
        participants: Vec<Participant>,
    ) -> Result<(), participants::InsertError<Self::InternalError>> {
        let mut participants_storage = self.participants.lock().await;

        for participant in participants {
            participants_storage.insert(participant.utxo_id, participant);
        }

        Ok(())
    }

    async fn update_participant_room(
        &self,
        participant: &U256,
        room_id: &uuid::Uuid,
    ) -> Result<(), participants::UpdateError<Self::InternalError>> {
        let mut participants_storage = self.participants.lock().await;

        let participant = participants_storage
            .get_mut(participant)
            .ok_or(UpdateError::NotFound(*participant))?;

        participant.room_id = Some(*room_id);

        Ok(())
    }

    async fn get_participant(&self, id: &U256) -> Result<Option<Participant>, Self::InternalError> {
        let participants = self.participants.lock().await;

        Ok(participants.get(id).cloned())
    }

    async fn update_participant_round(
        &self,
        participant: &U256,
        round: ShuffleRound,
    ) -> Result<(), UpdateError<Self::InternalError>> {
        let mut participants_storage = self.participants.lock().await;

        let participant = participants_storage
            .get_mut(participant)
            .ok_or(UpdateError::NotFound(*participant))?;

        participant.status = round;

        Ok(())
    }
}

#[async_trait]
impl queues::Storage for MapStorage {
    type InternalError = InternalError;

    async fn push_to_queue(
        &self,
        token: &Address,
        amount: &U256,
        participant_id: &U256,
    ) -> Result<(), Self::InternalError> {
        let mut queues_storage = self.queues.lock().await;

        let queue = match queues_storage.entry((*token, *amount)) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(Vec::new()),
        };

        queue.push(*participant_id);

        Ok(())
    }

    async fn pop_from_queue(
        &self,
        token: &Address,
        amount: &U256,
        number: usize,
    ) -> Result<Vec<U256>, queues::Error<Self::InternalError>> {
        let mut queues_storage = self.queues.lock().await;

        let queue = queues_storage
            .get_mut(&(*token, *amount))
            .ok_or(queues::Error::QueueNotFound)?;

        let split_at = queue.len().saturating_sub(number);

        let tail = queue.split_off(split_at);

        Ok(tail)
    }
}

#[async_trait]
impl super::Storage for MapStorage {
    type InternalError = InternalError;
}
