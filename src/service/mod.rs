pub mod storage;
pub mod types;
pub mod waiter;

use ethers_core::types::{Address, U256};

use self::{
    storage::{participants, queues, rooms},
    types::{participant::Participant, room::Room},
};

pub struct Service<S, W>
where
    S: storage::Storage,
    W: waiter::Waiter,
{
    storage: S,
    waiter: W,
}

impl<S, W> Service<S, W>
where
    S: storage::Storage,
    W: waiter::Waiter,
{
    pub fn new(storage: S, waiter: W) -> Self {
        Self { storage, waiter }
    }

    pub async fn add_participant(
        &self,
        token: &Address,
        amount: &U256,
        participant: &Participant,
    ) -> Result<(), Error<W::Error, <S as queues::Storage>::Error>> {
        self.waiter
            .add_to_queue(token, amount, &participant.utxo_id)
            .await
            .map_err(Error::Waiter)
    }

    pub async fn start_shuffle(
        &self,
        token: &Address,
        amount: &U256,
    ) -> Result<(), Error<W::Error, <S as rooms::Storage>::Error>> {
        let rooms = self
            .waiter
            .organize(token, amount)
            .await
            .map_err(Error::Waiter)?;

        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error<WE, SE>
where
    WE: std::error::Error,
    SE: std::error::Error,
{
    #[error("Waiter error: {0}")]
    Waiter(WE),
    #[error("Storage error: {0}")]
    Storage(SE),
}
