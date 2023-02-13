use ethers_core::types::{Signature, U256};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct Input {
    pub id: U256,
    pub timestamp: u64,
    pub signature: Signature,
}
