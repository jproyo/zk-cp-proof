use clap::Parser;
use std::error::Error;
use tokio::io::AsyncWriteExt;
use zk_cp_protocol::protocol::cp::{Material, MaterialSerde};

#[derive(Debug, Parser)]
#[clap(
    name = "Generate Random Material",
    version = "1.0",
    about = "Generate random material for testing"
)]
pub struct GenMaterial {
    #[arg(short, long, default_value = "client_material.json")]
    client_output_file: String,

    #[arg(short, long, default_value = "server_material.json")]
    server_output_file: String,

    #[arg(short, long, default_value = "user")]
    user: String,
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_line_number(true)
        .with_file(true)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();
}

/// Generates random material for testing.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let conf = GenMaterial::parse();
    init_tracing();
    tracing::info!("Generated random material ... ");
    let material = Material::default();
    let mut client_file = tokio::fs::File::create(conf.client_output_file.clone()).await?;
    let mut server_file = tokio::fs::File::create(conf.server_output_file.clone()).await?;
    let material_serde = MaterialSerde::from_material(&material, conf.user.as_str());
    let client_s = serde_json::to_string(&material_serde)?;
    client_file.write_all(client_s.as_bytes()).await?;
    client_file.write_all(b"\n").await?;
    client_file.flush().await?;

    let server_s = serde_json::to_string(&vec![material_serde])?;
    server_file.write_all(server_s.as_bytes()).await?;
    server_file.write_all(b"\n").await?;
    server_file.flush().await?;

    tracing::info!(
        "Material: {:?} write to file {:?} and {:?}",
        material,
        conf.client_output_file,
        conf.server_output_file
    );
    Ok(())
}
