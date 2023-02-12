use async_trait::async_trait;
use ethers_core::types::U256;
use types::Utxo;

pub mod default;
pub mod types;

#[async_trait]
pub trait Connector {
    type Error: std::error::Error;

    async fn get_utxo_by_id(&self, id: U256) -> Result<Option<Utxo>, Self::Error>;
}