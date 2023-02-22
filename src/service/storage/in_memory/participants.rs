use async_trait::async_trait;
use ethers::core::types::U256;

use crate::service::{
    storage::participants,
    types::{Participant, ShuffleRound},
};

use super::{InternalError, MapStorage};

#[async_trait]
impl participants::Storage for MapStorage {
    type InternalError = InternalError;

    async fn insert_participants(
        &self,
        participants: Vec<Participant>,
    ) -> Result<(), participants::InsertError<Self::InternalError>> {
        let mut participants_storage = self.participants.lock().await;

        for participant in participants {
            participants_storage.insert(participant.utxo_id, participant);
        }

        Ok(())
    }

    async fn update_participant_room(
        &self,
        participant: &U256,
        room_id: &uuid::Uuid,
    ) -> Result<(), participants::UpdateError<Self::InternalError>> {
        let mut participants_storage = self.participants.lock().await;

        let participant = participants_storage
            .get_mut(participant)
            .ok_or(participants::UpdateError::NotFound(*participant))?;

        participant.room_id = Some(*room_id);

        Ok(())
    }

    async fn get_participant(&self, id: &U256) -> Result<Option<Participant>, Self::InternalError> {
        let participants = self.participants.lock().await;

        Ok(participants.get(id).cloned())
    }

    async fn update_participant_round(
        &self,
        participant: &U256,
        round: ShuffleRound,
    ) -> Result<(), participants::UpdateError<Self::InternalError>> {
        let mut participants_storage = self.participants.lock().await;

        let participant = participants_storage
            .get_mut(participant)
            .ok_or(participants::UpdateError::NotFound(*participant))?;

        participant.status = round;

        Ok(())
    }
}
