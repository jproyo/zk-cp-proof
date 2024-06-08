use tonic::transport::{Channel, Endpoint};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;
use typed_builder::TypedBuilder;
use crate::conf::VerifierConfig;

#[derive(Clone, TypedBuilder)]
pub struct GrpcRegistryClient {
    client: Arc<Mutex<Channel>>,
}

impl GrpcRegistryClient {
    pub async fn new(config: &VerifierConfig) -> Result<Self, anyhow::Error> {
        let endpoints = config.material.
            .addresses
            .iter()
            .map(|endpoint| Endpoint::new(endpoint.to_string()))
            .collect::<Result<Vec<Endpoint>, _>>()
            .map_err(|e| {
               anyhow::anyhow!("Error connecting to blockchain {:?}", e)
            })?;
        let endpoints = endpoints
            .into_iter()
            .map(|e| e.timeout(Duration::from_secs(60)).clone());
        let client = Channel::balance_list(endpoints);

        Ok(GrpcRegistryClient {
            client: Arc::new(Mutex::new(client)),
        })
    }

}


