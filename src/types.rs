use ethers::types::{Address, U256, Signature};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Room {
    pub id: uuid::Uuid,
    pub token: Address,
    pub amount: U256,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
#[repr(u8)]
pub enum ShuffleState {
    Start,
    EncodingOutputs,
    SigningOutput,
    Finish,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Participant {
    pub id: uuid::Uuid,
    pub room_id: uuid::Uuid,
    pub utxo_id: U256,
    pub state: ShuffleState,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone)]
pub struct Input {
    pub id: U256,
    pub signature: Signature,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Output {
    pub amount: U256,
    pub owner: Address,
}
