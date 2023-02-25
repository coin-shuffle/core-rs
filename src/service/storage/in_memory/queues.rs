use std::collections::hash_map::Entry;

use async_trait::async_trait;
use ethers_core::{abi::Address, types::U256};

use crate::service::storage::queues;

use super::{InternalError, MapStorage};

#[async_trait]
impl queues::Storage for MapStorage {
    type InternalError = InternalError;

    async fn push_to_queue(
        &self,
        token: &Address,
        amount: &U256,
        participant_id: &U256,
    ) -> Result<(), Self::InternalError> {
        let mut queues_storage = self.queues.lock().await;

        let queue = match queues_storage.entry((*token, *amount)) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(Vec::new()),
        };

        queue.push(*participant_id);

        Ok(())
    }

    async fn pop_from_queue(
        &self,
        token: &Address,
        amount: &U256,
        number: usize,
    ) -> Result<Vec<U256>, queues::Error<Self::InternalError>> {
        let mut queues_storage = self.queues.lock().await;

        let queue = queues_storage
            .get_mut(&(*token, *amount))
            .ok_or(queues::Error::QueueNotFound)?;

        let split_at = queue.len().saturating_sub(number);

        let tail = queue.split_off(split_at);

        Ok(tail)
    }

    async fn queue_length(
        &self,
        token: &Address,
        amount: &U256,
    ) -> Result<usize, queues::Error<Self::InternalError>> {
        let queues_storage = self.queues.lock().await;

        let queue = queues_storage
            .get(&(*token, *amount))
            .ok_or(queues::Error::QueueNotFound)?;

        Ok(queue.len())
    }
}
