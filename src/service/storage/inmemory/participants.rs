use std::{collections::HashMap, sync::Arc};

use ethers_core::types::U256;
use tokio::sync::Mutex;

use crate::service::types::{Participant, ParticipantState};

/// `ParticipantsStorage` - provides inmemory storage for [`Participant`] entities.
#[derive(Clone)]
pub struct ParticipantsStorage {
    participants: Arc<Mutex<HashMap<U256, Participant>>>,
}

impl ParticipantsStorage {
    pub fn new() -> Self {
        Self {
            participants: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn insert(&self, participant: Participant) {
        let mut participants = self.participants.lock().await;
        participants.insert(participant.utxo_id, participant);
    }

    pub async fn get(&self, utxo_id: U256) -> Option<Participant> {
        let participants = self.participants.lock().await;
        participants.get(&utxo_id).cloned()
    }

    pub async fn get_many(&self, utxo_ids: &[U256]) -> Vec<Participant> {
        let participants = self.participants.lock().await;
        utxo_ids
            .iter()
            .filter_map(|utxo_id| participants.get(utxo_id).cloned())
            .collect()
    }

    pub async fn delete(&self, utxo_id: U256) {
        let mut participants = self.participants.lock().await;
        participants.remove(&utxo_id);
    }

    pub async fn update_state(&self, utxo_id: U256, state: ParticipantState) {
        let mut participants = self.participants.lock().await;
        if let Some(participant) = participants.get_mut(&utxo_id) {
            participant.state = state;
        }
    }
}
