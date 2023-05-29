use std::{collections::BTreeMap, sync::Arc};

use coin_shuffle_contracts_bindings::shared_types::Utxo;
use ethers_core::{abi::AbiEncode, types::U256};
use rsa::{RsaPrivateKey, RsaPublicKey};
use tokio::sync::Mutex;

use super::session::{Session, SessionWithKeys};

enum SessionState<O: AbiEncode + Clone> {
    Passive(Session<O>),
    Active(SessionWithKeys<O>),
    Finished,
}

/// A wrapper that lets manipulate user multiple session at the same time.
pub struct SessionManager<O: AbiEncode + Clone> {
    sessions: Arc<Mutex<BTreeMap<U256, SessionState<O>>>>,
}

#[derive(thiserror::Error, Debug)]
pub enum SessionManagerError {
    #[error("Session not found: {utxo_id}")]
    SessionNotFound { utxo_id: U256 },
    #[error("Session already has keys: {utxo_id}")]
    SessionAlreadyHasKeys { utxo_id: U256 },
}

use SessionManagerError::*;

type SessionManagerResult<T> = Result<T, SessionManagerError>;

impl<O: AbiEncode + Clone> SessionManager<O> {
    pub async fn add_session(&self, utxo: Utxo, output: O, rsa_priv_key: RsaPrivateKey) {
        let mut sessions = self.sessions.lock().await;

        let utxo_id = utxo.id;
        let session = Session::new(utxo, output, rsa_priv_key);

        sessions.insert(utxo_id, SessionState::Passive(session));
    }

    pub async fn session_add_keys(
        &self,
        utxo_id: U256,
        rsa_pub_keys: Vec<RsaPublicKey>,
    ) -> SessionManagerResult<()> {
        let mut sessions = self.sessions.lock().await;

        let session = sessions
            .get_mut(&utxo_id)
            .ok_or(SessionNotFound { utxo_id })?;

        session = match session {
            SessionState::Passive(session) => {
                let session_with_keys = session.add_keys(rsa_pub_keys);
                SessionState::Active(session)
            }
            _ => return Err(SessionAlreadyHasKeys { utxo_id }),
        };

        Ok(())
    }
}
