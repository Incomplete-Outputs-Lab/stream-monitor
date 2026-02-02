use crate::database::models::{Channel, StreamData};
use async_trait::async_trait;

#[async_trait]
pub trait Collector {
    async fn poll_channel(
        &self,
        channel: &Channel,
    ) -> Result<Option<StreamData>, Box<dyn std::error::Error + Send + Sync>>;
    async fn start_collection(
        &self,
        channel: &Channel,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
