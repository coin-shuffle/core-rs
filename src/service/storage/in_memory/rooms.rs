use async_trait::async_trait;

use crate::service::{storage::rooms, types::Room};

use super::{InternalError, MapStorage};

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
