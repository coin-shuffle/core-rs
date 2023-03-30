use ethers_core::types::U256;

mod participants;
mod rooms;

#[derive(Clone)]
pub struct ServiceStorage {
    participants: participants::ParticipantsStorage,
    rooms: rooms::RoomsStorage,
}

impl ServiceStorage {
    pub fn new() -> Self {
        Self {
            participants: participants::ParticipantsStorage::new(),
            rooms: rooms::RoomsStorage::new(),
        }
    }

    pub fn participants(&self) -> &participants::ParticipantsStorage {
        &self.participants
    }

    pub fn rooms(&self) -> &rooms::RoomsStorage {
        &self.rooms
    }

    /// Delete room, participants instances in storage and return deleted participants
    /// UTXO ids
    pub async fn clear_room(&self, room: &uuid::Uuid) -> Vec<U256> {
        let room = self.rooms.get(*room).await;

        if let Some(room) = room {
            self.rooms.delete(room.id).await;

            for participant in room.participants.iter() {
                self.participants.delete(*participant).await;
            }

            return room.participants;
        }

        vec![]
    }
}

impl Default for ServiceStorage {
    fn default() -> Self {
        Self::new()
    }
}
