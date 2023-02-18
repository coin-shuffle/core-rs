use ethers::core::types::{Address, U256};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Utxo {
    pub id: U256,
    pub amount: U256,
    pub token: String,
    pub owner: Address,
    pub is_spent: bool,
}
