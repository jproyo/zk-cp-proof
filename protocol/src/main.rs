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
    #[arg(short, long, default_value = "material.json")]
    output_file: String,

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let conf = GenMaterial::parse();
    init_tracing();
    tracing::info!("Generated random material ... ");
    let material = Material::default();
    let mut file = tokio::fs::File::create(conf.output_file.clone()).await?;
    let material_serde = MaterialSerde::from_material(&material, conf.user.as_str());
    let s = serde_json::to_string(&material_serde)?;
    file.write_all(s.as_bytes()).await?;
    file.write_all(b"\n").await?;
    file.flush().await?;
    tracing::info!(
        "Material: {:?} write to file {:?}",
        material,
        conf.output_file
    );
    Ok(())
}
