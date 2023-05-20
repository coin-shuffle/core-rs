use std::collections::BTreeSet;

use coin_shuffle_contracts_bindings::shared_types::Output;
use ethers_core::{
    abi::Hash,
    types::{Address, U256},
};

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    /// None of the users are connected to the room.
    Waiting,
    /// Users are connecting to the room by sending their public keys.
    /// The set contains all the users that are connected to the room.
    Connecting(BTreeSet<U256>),
    /// Current shuffle round number of the room.
    Shuffle(usize),
    /// Decoded outputs of the last user in the shuffle process with UTXO's
    /// that already passed their signature of the outputs.
    Signatures((Vec<Output>, Vec<U256>)),
    /// Hash of the transaction that is going to be sent to the blockchain.
    TransactionHash(Hash),
}

#[derive(Debug, Clone)]
pub struct Room {
    pub id: uuid::Uuid,
    pub token: Address,
    pub amount: U256,
    pub state: State,

    /// List of all UTXO that are participating in the room.
    /// Order in this vector represents the order which user participating
    /// in shuffle round.
    pub participants: Vec<U256>,
}

impl Room {
    pub fn new(token: Address, amount: U256, participants: Vec<U256>) -> Self {
        Self::with_id(uuid::Uuid::new_v4(), token, amount, participants)
    }

    pub fn with_id(id: uuid::Uuid, token: Address, amount: U256, participants: Vec<U256>) -> Self {
        Self {
            id,
            token,
            amount,
            state: State::Waiting,
            participants,
        }
    }
}
