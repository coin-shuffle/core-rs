use ethers_core::rand::seq::SliceRandom;
use rsa::RsaPublicKey;

use crate::types::Room;

/// provides shuffle operations for the of the shuffling process in
/// centralized provider.
pub trait Service {
    /// shuffle participants inside a room
    fn shuffle_room(mut room: Room) -> Room {
        room.participants.shuffle(&mut rand::thread_rng());
        room
    }

    /// return a list of keys needed to decrypt inputs for current participant
    fn participant_keys<'a>(
        room: &'a Room,
        participant: &uuid::Uuid,
    ) -> Option<Vec<&'a RsaPublicKey>> {
        // get participant position in the room
        let participant = room
            .participants
            .iter()
            .position(|p| participant == &p.id)?;

        // get the keys of the participants that are after the given participant
        let keys = room
            .participants
            .iter()
            .map(|p| &p.rsa_pubkey)
            .rev()
            .take(participant)
            .collect::<Vec<&'a RsaPublicKey>>();

        Some(keys)
    }
}
