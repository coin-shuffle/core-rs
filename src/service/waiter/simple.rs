use async_trait::async_trait;
use ethers_core::types::{Address, U256};

use crate::service::storage;
use crate::service::types::Room;

use super::{Error, Waiter};

/// SimpleWaiter - is a waiter that organizes participants to partially equal rooms
/// by the order they joined to the system.
#[derive(Clone)]
pub struct SimpleWaiter<S>
where
    S: storage::Storage,
{
    storage: S,
    room_size: usize,
}

#[async_trait]
impl<S> Waiter for SimpleWaiter<S>
where
    S: storage::Storage,
{
    async fn add_to_queue(
        &self,
        token: &Address,
        amount: &U256,
        participant: &U256,
    ) -> Result<(), Error> {
        self.storage
            .push_to_queue(token, amount, participant)
            .await
            .map_err(|e| Error::Storage(e.into()))?;

        Ok(())
    }

    async fn organize(&self, token: &Address, amount: &U256) -> Result<Vec<Room>, Error> {
        let mut rooms = Vec::new();

        let tx = self.storage.begin().await.map_err(Error::Storage)?;

        // TODO: remake it to for loop
        loop {
            let participants = self
                .storage
                .pop_from_queue(token, amount, self.room_size)
                .await
                .map_err(|e| Error::Storage(e.into()))?;

            let room = Room::new(*token, *amount, participants);

            self.storage
                .insert_room(&room)
                .await
                .map_err(|e| Error::Storage(e.into()))?;

            for participant in &room.participants {
                self.storage
                    .update_participant_room(participant, &room.id)
                    .await
                    .map_err(|e| Error::Storage(e.into()))?;
            }

            if room.participants.len() < self.room_size {
                let _ = tx.commit().await;

                // FIXME: especially incorrect when there is only one left participant
                rooms.push(room);

                return Ok(rooms);
            }

            rooms.push(room);
        }
    }
}

impl<S> SimpleWaiter<S>
where
    S: storage::Storage,
{
    pub fn new(room_size: usize, storage: S) -> Self {
        Self { storage, room_size }
    }
}
