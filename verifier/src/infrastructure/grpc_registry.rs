use crate::conf::VerifierConfig;
use crate::domain::verifier::MaterialRegistry;
use crate::grpc::zkp_material::material_client::MaterialClient;
use crate::grpc::zkp_material::QueryRequest;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tonic::transport::{Channel, Endpoint};
use typed_builder::TypedBuilder;

#[derive(Clone, TypedBuilder)]
pub struct GrpcRegistryClient {
    client: Arc<Mutex<Channel>>,
}

impl GrpcRegistryClient {
    pub fn new(config: &VerifierConfig) -> Result<Self, anyhow::Error> {
        let endpoints = config
            .material
            .addresses
            .iter()
            .map(|endpoint| Endpoint::new(endpoint.to_string()))
            .collect::<Result<Vec<Endpoint>, _>>()
            .map_err(|e| anyhow::anyhow!("Error connecting to blockchain {:?}", e))?;
        let endpoints = endpoints
            .into_iter()
            .map(|e| e.timeout(Duration::from_secs(60)).clone());
        let client = Channel::balance_list(endpoints);

        Ok(GrpcRegistryClient {
            client: Arc::new(Mutex::new(client)),
        })
    }
}

#[async_trait::async_trait]
impl MaterialRegistry for GrpcRegistryClient {
    async fn query(
        &self,
        user: &crate::domain::verifier::User,
    ) -> anyhow::Result<Option<crate::domain::verifier::Material>> {
        let mut query = MaterialClient::new(self.client.lock().await.clone());
        let query_req = QueryRequest {
            user: user.to_string(),
        };
        tracing::info!("Querying material for user: {:?}", user);
        let resp = query.get(query_req).await;
        match resp {
            Err(e) => {
                if e.code() == tonic::Code::NotFound {
                    return Ok(None);
                }
                Err(anyhow::anyhow!("Error querying material {:?}", e))
            }
            Ok(resp) => Ok(Some(resp.into_inner().into())),
        }
    }
}
