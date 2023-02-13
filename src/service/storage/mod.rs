pub mod participants;
pub mod queues;
pub mod rooms;

pub trait Storage:
    queues::Storage + rooms::Storage + participants::Storage + Sync + Send + 'static
{
}

#[derive(thiserror::Error, Debug)]
pub enum Error<QE, RE, PE>
where
    QE: std::error::Error,
    RE: std::error::Error,
    PE: std::error::Error,
{
    #[error("queues storage error: {0}")]
    QueuesStorage(QE),
    #[error("rooms storage error: {0}")]
    RoomsStorage(RE),
    #[error("participants storage error: {0}")]
    ParticipantsStorage(PE),
}
