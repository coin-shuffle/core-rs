use ethers_core::types::{Address, U256};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Room {
    pub id: uuid::Uuid,
    pub token: Address,
    pub amount: U256,
    pub current_round: u64,

    /// List of all UTXO's that are participating in the room.
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
            current_round: 0,
            participants,
        }
    }
}
