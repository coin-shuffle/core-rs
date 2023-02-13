use ethers_core::types::{Address, U256};

use crate::service::storage::{self, participants, rooms};
use crate::service::{storage::queues, types::Room};

use super::Waiter;

/// Simple waiter that that organizes participants to partially equal rooms
/// by the order they joined to the system.
pub struct SimpleWaiter<S>
where
    S: storage::Storage,
{
    storage: S,
    room_size: u64,
}

#[async_trait::async_trait]
impl<S> Waiter for SimpleWaiter<S>
where
    S: storage::Storage,
{
    type Error = Error<
        <S as queues::Storage>::Error,
        <S as rooms::Storage>::Error,
        <S as participants::Storage>::Error,
    >;

    async fn add_to_queue(
        &self,
        token: &Address,
        amount: &U256,
        participant: &U256,
    ) -> Result<(), Self::Error> {
        self.storage
            .push_to_queue(token, amount, participant)
            .await
            .map_err(|err| Error::Storage(storage::Error::QueuesStorage(err)))
    }

    async fn organize(&self, token: &Address, amount: &U256) -> Result<Vec<Room>, Self::Error> {
        let mut rooms = Vec::new();

        loop {
            // TODO: remake it to for loop
            let participants = self
                .storage
                .pop_from_queue(token, amount, self.room_size)
                .await
                .map_err(|err| Error::Storage(storage::Error::QueuesStorage(err)))?;

            if participants.len() < (self.room_size as usize) {
                // FIXME: distribute left participants between existing rooms
                return Ok(rooms);
            }

            let room = Room::new(token.clone(), amount.clone(), participants);

            self.storage
                .insert_room(&room)
                .await
                .map_err(|err| Error::Storage(storage::Error::RoomsStorage(err)))?;

            for participant in &room.participants {
                self.storage
                    .update_participant_room(&participant, &room.id)
                    .await
                    .map_err(|err| Error::Storage(storage::Error::ParticipantsStorage(err)))?;
            }

            rooms.push(room);
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error<SE, RE, PE>
where
    SE: std::error::Error,
    RE: std::error::Error,
    PE: std::error::Error,
{
    #[error("storage error: {0}")]
    Storage(#[from] storage::Error<SE, RE, PE>),
}
