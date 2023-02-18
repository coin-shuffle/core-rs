use async_trait::async_trait;

use crate::service::Room;

#[async_trait]
pub trait Storage {
    type InternalError: std::error::Error;

    async fn get_room(&self, id: &uuid::Uuid) -> Result<Option<Room>, Self::InternalError>;

    async fn update_room_round(
        &self,
        id: &uuid::Uuid,
        round: usize,
    ) -> Result<(), UpdateError<Self::InternalError>>;

    async fn insert_room(&self, room: &Room) -> Result<(), InsertError<Self::InternalError>>;
}

#[derive(thiserror::Error, Debug)]
pub enum InsertError<IE>
where
    IE: std::error::Error,
{
    #[error("room already exists")]
    RoomAlreadyExists,
    #[error("internal error: {0}")]
    Internal(IE),
}

#[derive(thiserror::Error, Debug)]
pub enum UpdateError<IE>
where
    IE: std::error::Error,
{
    #[error("room not found")]
    NotFound,
    #[error("internal error: {0}")]
    Internal(IE),
}
