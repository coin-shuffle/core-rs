use crate::service::types::RoomState;
use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;
use uuid::Uuid;

use crate::service::types::Room;

#[derive(Clone)]
pub struct RoomsStorage {
    rooms: Arc<Mutex<HashMap<Uuid, Room>>>,
}

impl RoomsStorage {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn insert(&self, room: Room) {
        let mut rooms = self.rooms.lock().await;
        rooms.insert(room.id, room);
    }

    pub async fn get(&self, id: Uuid) -> Option<Room> {
        let rooms = self.rooms.lock().await;
        rooms.get(&id).cloned()
    }

    pub async fn delete(&self, id: Uuid) {
        let mut rooms = self.rooms.lock().await;
        rooms.remove(&id);
    }

    pub async fn update_state(&self, id: Uuid, state: RoomState) {
        let mut rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get_mut(&id) {
            room.state = state;
        }
    }
}
