use crate::utxo_connector::{types::Utxo, Connector};
use async_trait::async_trait;
use ethers::core::{k256::ecdsa::SigningKey, types::U256};

pub struct DefaultConnector {
    private_key: SigningKey,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {}

#[async_trait]
impl Connector for DefaultConnector {
    type Error = Error;

    async fn get_utxo_by_id(&self, id: U256) -> Result<Option<Utxo>, Error> {
        todo!()
    }
}
