pub mod participants;
pub mod queues;
pub mod rooms;

use async_trait::async_trait;
use std::ops::Deref;
use tokio::runtime::Handle;

#[async_trait]
pub trait Storage:
    queues::Storage + rooms::Storage + participants::Storage + Sync + Send + Clone
{
    type Transaction: Transaction<Storage = Self> + Send + Sync;

    async fn begin(&self) -> Result<TransactionGuard<Self::Transaction>, Error>;
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("transaction error: {0}")]
    Transaction(#[from] TransactionError),
    #[error("queue error: {0}")]
    Queue(#[from] queues::Error),
    #[error("room error: {0}")]
    Room(#[from] rooms::Error),
    #[error("participant error: {0}")]
    Participant(#[from] participants::Error),
}

/// Transaction - represents storage transaction logic.
#[async_trait]
pub trait Transaction: Deref<Target = Self::Storage> + Send + Sync {
    type Storage: Storage;

    /// rollback is called on `transaction` dropping
    async fn rollback(&self) -> Result<(), TransactionError>;

    async fn commit(&self) -> Result<(), TransactionError>;
}

#[derive(thiserror::Error, Debug)]
pub enum TransactionError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct TransactionGuard<T>
where
    T: Transaction,
{
    inner: T,
    committed: bool,
}

impl<T> TransactionGuard<T>
where
    T: Transaction,
{
    pub fn new(transaction: T) -> Self {
        Self {
            inner: transaction,
            committed: false,
        }
    }

    pub async fn commit(mut self) -> Result<(), TransactionError> {
        self.committed = true;
        self.inner.commit().await
    }
}

impl<T> Deref for TransactionGuard<T>
where
    T: Transaction,
{
    type Target = T::Storage;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<T> Drop for TransactionGuard<T>
where
    T: Transaction,
{
    fn drop(&mut self) {
        if !self.committed {
            // TODO: do something with error
            if let Err(err) = Handle::current().block_on(self.inner.rollback()) {
                log::error!("Error on transaction rollback: {}", err);
            }
        }
    }
}
