pub mod in_memory;
pub mod participants;
pub mod queues;
pub mod rooms;

use async_trait::async_trait;
use std::ops::Deref;
use tokio::runtime::Handle;

#[async_trait]
pub trait Storage:
    queues::Storage<InternalError = <Self as Storage>::InternalError>
    + rooms::Storage<InternalError = <Self as Storage>::InternalError>
    + participants::Storage<InternalError = <Self as Storage>::InternalError>
    + Sync
    + Send
{
    type InternalError: std::error::Error + 'static;
    type Transaction: Transaction<Error = <Self as Storage>::InternalError, Storage = Self>;

    async fn transaction(
        &self,
    ) -> Result<TransactionGuard<Self::Transaction>, <Self as Storage>::InternalError> {
        Ok(TransactionGuard::new(Transaction::new(self).await))
    }
}

/// Transaction - represents storage transaction logic.
#[async_trait]
pub trait Transaction: Send + Sync {
    type Storage: Storage;
    type Error: std::error::Error + 'static;

    async fn new(storage: &Self::Storage) -> Self;

    fn storage(&self) -> &Self::Storage;

    /// rollback is called on `transaction` dropping
    async fn rollback(&self) -> Result<(), Self::Error>;

    async fn commit(&self) -> Result<(), Self::Error>;
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
    fn new(transaction: T) -> Self {
        Self {
            inner: transaction,
            committed: false,
        }
    }

    pub async fn commit(mut self) -> Result<(), T::Error> {
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
        self.inner.storage()
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
