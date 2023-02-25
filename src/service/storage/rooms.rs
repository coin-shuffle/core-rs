use async_trait::async_trait;

use crate::service::Room;

#[async_trait]
pub trait Storage {
    async fn get_room(&self, id: &uuid::Uuid) -> Result<Option<Room>, Error>;

    async fn update_room_round(&self, id: &uuid::Uuid, round: usize) -> Result<(), Error>;

    async fn insert_room(&self, room: &Room) -> Result<(), Error>;
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("internal error")]
    Internal(String),
}
