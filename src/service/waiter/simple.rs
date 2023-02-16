use ethers_core::types::{Address, U256};

use crate::service::storage::{self, participants, queues, rooms};
use crate::service::types::Room;

use super::Waiter;

/// SimpleWaiter - is a waiter that organizes participants to partially equal rooms
/// by the order they joined to the system.
pub struct SimpleWaiter<S>
where
    S: storage::Storage,
{
    storage: S,
    room_size: usize,
}

#[async_trait::async_trait]
impl<S> Waiter for SimpleWaiter<S>
where
    S: storage::Storage,
{
    type InternalError = Error<<S as storage::Storage>::InternalError>;

    async fn add_to_queue(
        &self,
        token: &Address,
        amount: &U256,
        participant: &U256,
    ) -> Result<(), Self::InternalError> {
        self.storage
            .push_to_queue(token, amount, participant)
            .await
            .map_err(Error::AddToQueue)?;

        Ok(())
    }

    async fn organize(
        &self,
        token: &Address,
        amount: &U256,
    ) -> Result<Vec<uuid::Uuid>, Self::InternalError> {
        let mut rooms = Vec::new();

        // TODO: remake it to for loop
        loop {
            let participants = self
                .storage
                .pop_from_queue(token, amount, self.room_size)
                .await?;

            if participants.len() < self.room_size {
                // FIXME: distribute left participants between existing rooms
                return Ok(rooms);
            }

            let room = Room::new(*token, *amount, participants);

            self.storage.insert_room(&room).await?;

            for participant in &room.participants {
                self.storage
                    .update_participant_room(participant, &room.id)
                    .await?;
            }

            rooms.push(room.id);
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

#[derive(thiserror::Error, Debug)]
pub enum Error<DE>
where
    DE: std::error::Error,
{
    #[error("failed to add to queue: {0}")]
    AddToQueue(DE),
    #[error("failed to get participants: {0}")]
    GetParticipants(#[from] queues::Error<DE>),
    #[error("failed to insert room: {0}")]
    InsertRoom(#[from] rooms::InsertError<DE>),
    #[error("failed to update participant: {0}")]
    UpdateParticipantRoom(#[from] participants::UpdateError<DE>),
}

#[cfg(test)]
mod tests {
    use ethers_core::types::{Address, U256};
    use rand::rngs::OsRng;
    use rsa::{RsaPrivateKey, RsaPublicKey};

    use crate::service::{
        storage::{
            in_memory::MapStorage, participants::Storage as ParticipantsStorage,
            rooms::Storage as RoomsStorage,
        },
        types::participant::Participant,
        waiter::Waiter,
    };

    use super::SimpleWaiter;

    /// Create 15 participants, and 3 rooms with size 5
    #[tokio::test]
    async fn happy_path() {
        const ROOM_SIZE: usize = 5;
        const PARTICIPANTS_NUMBER: usize = 15;
        let token: Address = Address::default();
        let amount: U256 = U256::from(5);

        let storage = MapStorage::default();

        let waiter = SimpleWaiter::new(ROOM_SIZE, storage.clone());

        let bits = 2048;

        let mut participants = Vec::with_capacity(PARTICIPANTS_NUMBER);

        // FIXME: generting of rsa key is very slow and must be fixed somehow
        let private_key = RsaPrivateKey::new(&mut OsRng, bits).expect("failed to generate a key");
        let public_key = RsaPublicKey::from(&private_key);

        for i in 1..=PARTICIPANTS_NUMBER {
            let participant = Participant::new(U256::from(i), public_key.clone());

            waiter
                .add_to_queue(&token, &amount, &participant.utxo_id)
                .await
                .expect("should add to queue successfully");

            participants.push(participant);
        }

        storage.insert_participants(participants).await.unwrap();

        let rooms = waiter.organize(&token, &amount).await.unwrap();

        for room_id in rooms {
            let room = storage
                .get_room(&room_id)
                .await
                .unwrap()
                .expect("should create rooms in storage");

            assert_eq!(
                room.participants.len(),
                ROOM_SIZE,
                "should create room with ROOM_SIZE participants"
            );
        }
    }
}
