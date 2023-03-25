mod participants;
mod queues;
mod rooms;

#[derive(Clone)]
pub struct Storage {
    participants: participants::ParticipantsStorage,
    queues: queues::QueuesStorage,
    rooms: rooms::RoomsStorage,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            participants: participants::ParticipantsStorage::new(),
            queues: queues::QueuesStorage::new(),
            rooms: rooms::RoomsStorage::new(),
        }
    }

    pub fn participants(&self) -> &participants::ParticipantsStorage {
        &self.participants
    }

    pub fn queues(&self) -> &queues::QueuesStorage {
        &self.queues
    }

    pub fn rooms(&self) -> &rooms::RoomsStorage {
        &self.rooms
    }

    pub async fn clear_room(&self, room: uuid::Uuid) {
        let room = self.rooms.get(room).await;

        if let Some(room) = room {
            self.rooms.delete(room.id).await;

            for participant in room.participants {
                self.participants.delete(participant).await;

                self.queues.push(room.token, room.amount, participant).await;
            }
        }
    }
}
