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
