/// This module contains the gRPC server implementation for the verifier service.
/// It provides the necessary server functionality for user registration, authentication challenge creation,
/// and authentication verification.
///
/// The `GrpcServer` struct is responsible for handling incoming gRPC requests and delegating them to the
/// appropriate methods in the `VerifierApplication` implementation.
///
/// The `new_server` function creates a new gRPC server with the given verifier configuration.
///
/// The `Auth` trait defines the gRPC service methods for user registration, authentication challenge creation,
/// and authentication verification. The `GrpcServer` struct implements this trait to provide the actual
/// implementation for these methods.
///
/// The `run` function starts the gRPC server and serves incoming requests.
///
/// Example usage:
///
/// ```rust
/// use crate::conf::VerifierConfig;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let settings: VerifierConfig = conf::init()?;
///     run(&settings).await?;
///     Ok(())
/// }
/// ```
use super::zkp_auth::auth_server::{Auth, AuthServer};
use super::zkp_auth::{
    AuthenticationAnswerRequest, AuthenticationAnswerResponse, AuthenticationChallengeRequest,
    AuthenticationChallengeResponse, RegisterRequest, RegisterResponse,
};
use crate::application::handler::{VerifierApplication, VerifierService};
use crate::conf::VerifierConfig;
use crate::infrastructure::file_params::FileParams;
use crate::infrastructure::mem_storage::MemStorage;
use std::sync::Arc;
use tonic::async_trait;
use tonic::transport::Server;

#[derive(Debug, Clone)]
pub struct GrpcServer<APP> {
    application: Arc<APP>,
}

pub(crate) type DefaultApp = VerifierApplication<FileParams, MemStorage>;

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
        let resp = challenge_started.try_into().map_err(|e: anyhow::Error| {
            tonic::Status::internal(format!(
                "Error converting challenge response: {:?}",
                e.to_string()
            ))
        })?;
        Ok(tonic::Response::new(resp))
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
        let resp = challenge_verification.try_into()?;
        tracing::info!("Verification Response: {:?}", resp);
        Ok(tonic::Response::new(resp))
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
