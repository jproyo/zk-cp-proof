use super::zkp_auth::auth_server::{Auth, AuthServer};
use super::zkp_auth::{
    AuthenticationAnswerRequest, AuthenticationAnswerResponse, AuthenticationChallengeRequest,
    AuthenticationChallengeResponse, RegisterRequest, RegisterResponse,
};
use crate::application::handler::{VerifierApplication, VerifierService};
use crate::conf::VerifierConfig;
use crate::domain::verifier::ChallengeVerificationResult;
use crate::infrastructure::grpc_registry::GrpcRegistryClient;
use crate::infrastructure::mem_storage::MemStorage;
use std::sync::Arc;
use tonic::async_trait;
use tonic::transport::Server;

#[derive(Debug, Clone)]
pub struct GrpcServer<APP> {
    application: Arc<APP>,
}

pub(crate) type DefaultApp = VerifierApplication<GrpcRegistryClient, MemStorage>;

impl GrpcServer<DefaultApp> {
    /// Creates a new gRPC server with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `conf` - The verifier configuration.
    ///
    /// # Returns
    ///
    /// A Result containing the authenticated server if successful, or an error if the server creation fails.
    pub fn new_server(conf: &VerifierConfig) -> anyhow::Result<AuthServer<impl Auth>> {
        let app = DefaultApp::new_with_config(conf)?;
        Ok(AuthServer::new(GrpcServer {
            application: Arc::new(app),
        }))
    }
}

#[async_trait]
impl<APP> Auth for GrpcServer<APP>
where
    APP: VerifierService + Send + Sync + 'static,
{
    async fn register(
        &self,
        request: tonic::Request<RegisterRequest>,
    ) -> Result<tonic::Response<RegisterResponse>, tonic::Status> {
        let request = request.into_inner();
        let register = request.into();
        self.application.register(register).await.map_err(|e| {
            tonic::Status::internal(format!("Error registering user: {:?}", e.to_string()))
        })?;
        Ok(tonic::Response::new(RegisterResponse {}))
    }

    async fn create_authentication_challenge(
        &self,
        request: tonic::Request<AuthenticationChallengeRequest>,
    ) -> Result<tonic::Response<AuthenticationChallengeResponse>, tonic::Status> {
        let request = request.into_inner();
        let challenge = request.into();
        let challenge_started =
            self.application
                .create_challenge(challenge)
                .await
                .map_err(|e| {
                    tonic::Status::internal(format!(
                        "Error creating authentication challenge: {:?}",
                        e.to_string()
                    ))
                })?;
        Ok(tonic::Response::new(challenge_started.into()))
    }

    async fn verify_authentication(
        &self,
        request: tonic::Request<AuthenticationAnswerRequest>,
    ) -> Result<tonic::Response<AuthenticationAnswerResponse>, tonic::Status> {
        let request = request.into_inner();
        let challenge = request.into();
        let challenge_verification =
            self.application
                .verify_challenge(challenge)
                .await
                .map_err(|e| {
                    tonic::Status::internal(format!(
                        "Error verifying authentication: {:?}",
                        e.to_string()
                    ))
                })?;
        match challenge_verification {
            ChallengeVerificationResult::ChallengeVerified(session_id) => {
                Ok(tonic::Response::new(AuthenticationAnswerResponse {
                    session_id: session_id.0,
                }))
            }
            ChallengeVerificationResult::ChallengeVerificationFailed => Err(
                tonic::Status::invalid_argument("Challenge verification failed"),
            ),
        }
    }
}

pub async fn run(settings: &VerifierConfig) -> anyhow::Result<()> {
    let material_server = GrpcServer::new_server(settings)?;

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<AuthServer<GrpcServer<DefaultApp>>>()
        .await;

    let timeout = tokio::time::Duration::from_secs(settings.response_timeout_in_secs);

    let grpc_layer = tower::ServiceBuilder::new().timeout(timeout);

    let server = Server::builder().timeout(timeout);

    let router = server
        .layer(grpc_layer)
        .add_service(health_service)
        .add_service(material_server);

    tracing::info!(
        "Successfully created server for material in port {:?}.",
        settings.port
    );

    router
        .serve(
            format!("0.0.0.0:{}", settings.port)
                .parse()
                .map_err(|e| anyhow::anyhow!("Error parsing address: {:?}", e))?,
        )
        .await?;
    Ok(())
}
