use super::zkp_material::material_server::{Material, MaterialServer};
use super::zkp_material::{self, MaterialRequest, MaterialResponse};
use crate::application::handler::{MaterialApplication, MaterialService};
use crate::conf::MaterialConfig;
use crate::infrastructure::generator::DefaultMaterialGenerator;
use crate::infrastructure::mem_storage::MemStorage;
use std::sync::Arc;
use tonic::async_trait;
use tonic::transport::Server;

#[derive(Debug, Clone)]
pub struct GrpcServer<APP> {
    application: Arc<APP>,
}

pub(crate) type DefaultApp = MaterialApplication<DefaultMaterialGenerator, MemStorage>;

impl GrpcServer<DefaultApp> {
    pub fn new_server() -> MaterialServer<impl Material> {
        MaterialServer::new(GrpcServer {
            application: Arc::new(DefaultApp::new_default()),
        })
    }
}

#[async_trait]
impl<APP> Material for GrpcServer<APP>
where
    APP: MaterialService + Send + Sync + 'static,
{
    async fn generate(
        &self,
        request: tonic::Request<MaterialRequest>,
    ) -> Result<tonic::Response<MaterialResponse>, tonic::Status> {
        let req = request.into_inner();
        let user = req.user.into();
        let q = req.q.map(|q| (q as u64).into());
        let material = self
            .application
            .create_material(&user, q)
            .await
            .map_err(|e| {
                tracing::error!("Error generating material: {:?}", e);
                tonic::Status::invalid_argument(format!("Error generating material {}", e))
            })?;
        let resp = material.try_into().map_err(|e| {
            tracing::error!("Error converting material: {:?}", e);
            tonic::Status::internal("Error converting material")
        })?;
        Ok(tonic::Response::new(resp))
    }

    async fn get(
        &self,
        request: tonic::Request<zkp_material::QueryRequest>,
    ) -> Result<tonic::Response<zkp_material::MaterialResponse>, tonic::Status> {
        let user = request.into_inner().user.into();
        let material = self.application.get_material(&user).await.map_err(|e| {
            tracing::error!("Error getting material: {:?}", e);
            tonic::Status::internal("Error getting material")
        })?;
        let resp = material
            .map(|m| {
                m.try_into().map_err(|e| {
                    tracing::error!("Error converting material: {:?}", e);
                    tonic::Status::internal("Error converting material")
                })
            })
            .transpose()?;
        if let Some(resp) = resp {
            return Ok(tonic::Response::new(resp));
        } else {
            return Err(tonic::Status::not_found("Material not found"));
        }
    }
}

pub async fn run(settings: &MaterialConfig) -> anyhow::Result<()> {
    let material_server = GrpcServer::new_server();

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<MaterialServer<GrpcServer<DefaultApp>>>()
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
