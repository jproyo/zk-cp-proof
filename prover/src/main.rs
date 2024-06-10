use clap::Parser;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use tokio::time::Duration;
use tonic::transport::Endpoint;
use zk_cp_protocol::protocol::cp::{
    Challenge, ChallengeResponse, MaterialSerde, ProtocolState, ProtocolTransition, Register,
};
use zk_prover::grpc::zkp_auth::{self, AuthenticationAnswerRequest};

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_line_number(true)
        .with_file(true)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();
}

#[derive(Debug, Parser)]
pub struct Verifier {
    #[clap(short, long, default_value = "http://localhost:50000")]
    prover_address: String,

    #[clap(short, long, default_value = "user")]
    user: String,

    #[clap(short, long)]
    x: String,

    #[clap(short, long, default_value = "material.json")]
    material_path: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conf = Verifier::parse();
    init_tracing();
    let endpoint = Endpoint::new(conf.prover_address.clone())?.timeout(Duration::from_secs(60));
    tracing::info!("Connecting to prover at {}", conf.prover_address);
    let client = endpoint.connect().await?;

    let mut service = zkp_auth::auth_client::AuthClient::new(client);

    tracing::info!("Getting material from file {}", conf.material_path);
    let material: MaterialSerde =
        serde_json::from_str(&std::fs::read_to_string(&conf.material_path)?)?;

    let material = material.to_material();
    tracing::info!("Material: {:?}", material);

    let x = &BigInt::parse_bytes(conf.x.as_bytes(), 10)
        .ok_or_else(|| anyhow::anyhow!("BigInt conversion error for x"))?;

    let register_zk = Register::new(material.clone(), x);

    let register = zkp_auth::RegisterRequest {
        user: conf.user.to_string(),
        y1: register_zk.y1.to_i64().ok_or_else(|| {
            anyhow::anyhow!("BigInt conversion error to i64 for sending result to grpc")
        })?,
        y2: register_zk.y2.to_i64().ok_or_else(|| {
            anyhow::anyhow!("BigInt conversion error to i64 for sending result to grpc")
        })?,
    };
    tracing::info!("Registering user: {:?}", register);
    service.register(register).await?;
    tracing::info!("User registered successfully");

    let challenge = <Register as Into<ProtocolState<_>>>::into(register_zk)
        .change()
        .into_inner();

    let auth_req = zkp_auth::AuthenticationChallengeRequest {
        user: conf.user.to_string(),
        r1: challenge.r1.to_i64().ok_or_else(|| {
            anyhow::anyhow!("BigInt conversion error to i64 for sending result to grpc")
        })?,
        r2: challenge.r2.to_i64().ok_or_else(|| {
            anyhow::anyhow!("BigInt conversion error to i64 for sending result to grpc")
        })?,
    };
    tracing::info!("Sending challenge: {:?}", auth_req);
    let response = service.create_authentication_challenge(auth_req).await?;
    tracing::info!("Challenge sent successfully {:?}", response);

    let challenge_response = response.into_inner();
    let verification = ProtocolState::from(ChallengeResponse {
        challenge: Challenge::builder()
            .auth_id(challenge_response.auth_id)
            .c(BigInt::from(challenge_response.c))
            .build(),
        material: material.clone(),
        x: x.clone(),
        k: challenge.k,
    })
    .change()
    .into_inner();

    let req = AuthenticationAnswerRequest {
        auth_id: verification.auth_id.to_string(),
        s: verification.s.to_i32().ok_or_else(|| {
            anyhow::anyhow!("BigInt conversion error to i32 for sending result to grpc")
        })?,
    };

    tracing::info!("Verifying authentication: {:?}", req);
    let result = service.verify_authentication(req).await?;
    tracing::info!("Verification result: {:?}", result);

    Ok(())
}
