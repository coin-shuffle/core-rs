pub mod in_memory;
pub mod participants;
pub mod queues;
pub mod rooms;

pub trait Storage:
    queues::Storage<InternalError = <Self as Storage>::InternalError>
    + rooms::Storage<InternalError = <Self as Storage>::InternalError>
    + participants::Storage<InternalError = <Self as Storage>::InternalError>
    + Sync
    + Send
{
    type InternalError: std::error::Error + 'static;
}

pub trait Connection {
    type Storage: Storage;
    type Error: std::error::Error + 'static;

    fn new(storage: Self::Storage) -> Self;

    fn transaction<T>(&self) -> T
    where
        T: Transaction<Storage = Self::Storage, Error = Self::Error>;
}

pub trait Transaction {
    type Storage: Storage;
    type Error: std::error::Error + 'static;

    fn storage(&self) -> &Self::Storage;

    fn rollback(&self) -> Result<(), Self::Error>;

    fn commit(self) -> Result<(), Self::Error>;
}
