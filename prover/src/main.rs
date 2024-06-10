use clap::Parser;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use tokio::time::Duration;
use tonic::transport::Endpoint;
use zk_cp_protocol::protocol::cp::{Material, Register};
use zk_prover::grpc::zkp_auth;

#[derive(Debug, Parser)]
pub struct Verifier {
    #[clap(short, long, default_value = "http://localhost:50000")]
    prover_address: String,

    #[clap(short, long, default_value = "user")]
    user: String,

    #[clap(short, long)]
    x: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conf = Verifier::parse();
    let endpoint = Endpoint::new(conf.prover_address)?.timeout(Duration::from_secs(60));
    let client = endpoint.connect().await?;

    let mut service = zkp_auth::auth_client::AuthClient::new(client);

    let material: Material;

    let register = Register::new(
        material,
        &BigInt::parse_bytes(conf.x.as_bytes(), 10)
            .ok_or_else(|| anyhow::anyhow!("BigInt conversion error for x"))?,
    );

    let register = zkp_auth::RegisterRequest {
        user: conf.user,
        y1: register.y1.to_i64().ok_or_else(|| {
            anyhow::anyhow!("BigInt conversion error to i64 for sending result to grpc")
        })?,
        y2: register.y2.to_i64().ok_or_else(|| {
            anyhow::anyhow!("BigInt conversion error to i64 for sending result to grpc")
        })?,
    };
    service.register(register).await?;

    Ok(())
}
