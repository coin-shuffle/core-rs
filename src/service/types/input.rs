use ethers_core::{abi::ethereum_types::Signature, types::U256};

/// Id of the UTXO
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct Input {
    pub id: U256,
    pub signature: Signature,
}
