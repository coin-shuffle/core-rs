use ethers_core::types::U256;
use rsa::RsaPublicKey;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
#[repr(u8)]
pub enum ShuffleStatus {
    Wait,
    Start,
    EncodingOutputs,
    SigningOutput,
    Finish,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Participant {
    pub room_id: Option<uuid::Uuid>,
    pub utxo_id: U256,
    pub rsa_pubkey: RsaPublicKey,
    pub status: ShuffleStatus,
}

impl Participant {
    pub fn new(utxo_id: U256, pubkey: RsaPublicKey) -> Self {
        Self {
            room_id: None, // because participant haven't entered room yet
            utxo_id,
            rsa_pubkey: pubkey,
            status: ShuffleStatus::Wait,
        }
    }

    pub fn enter_room(&mut self, room_id: uuid::Uuid) {
        self.room_id = Some(room_id);
        self.status = ShuffleStatus::Start;
    }
}
