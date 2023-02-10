use ethers::types::{Address, Signature, U256};
use rsa::RsaPublicKey;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Room {
    pub id: uuid::Uuid,
    pub token: Address,
    pub amount: U256,
    pub current_round: u64,
    pub participants: Vec<Participant>,
}

impl Room {
    pub fn new(token: Address, amount: U256) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            token,
            amount,
            current_round: 0,
            participants: Vec::new(),
        }
    }
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
    pub rsa_pubkey: RsaPublicKey,
    pub state: ShuffleState,
}

impl Participant {
    pub fn new(room_id: uuid::Uuid, utxo_id: U256, pubkey: RsaPublicKey) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            room_id,
            utxo_id,
            rsa_pubkey: pubkey,
            state: ShuffleState::Start,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
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
