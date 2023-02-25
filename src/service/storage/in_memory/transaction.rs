use async_trait::async_trait;

use crate::service::storage::Transaction;

use super::{InternalError, MapStorage};

pub struct InMemoryTransaction {
    // pointer to the storage, after commit all changes will be merged into the storage
    storage: MapStorage,
    // copy of the storage before the transaction. on this copy all changes will be made
    snapshot: MapStorage,
}

#[async_trait]
impl Transaction for InMemoryTransaction {
    type Storage = MapStorage;
    type Error = InternalError;

    async fn new(storage: &Self::Storage) -> Self {
        Self {
            storage: storage.clone(),
            snapshot: storage.copy().await,
        }
    }

    #[allow(clippy::misnamed_getters)]
    fn storage(&self) -> &Self::Storage {
        &self.snapshot
    }

    /// rollback is called on `transaction` dropping
    async fn rollback(&self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn commit(&self) -> Result<(), Self::Error> {
        self.storage.merge(&self.snapshot).await;

        Ok(())
    }
}
