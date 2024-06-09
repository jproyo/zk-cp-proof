use clap::Parser;
use zk_verifier::conf::{init, VerifierConfig};
use zk_verifier::grpc::server::run;

#[derive(Parser, Debug)]
struct Options {
    #[arg(
        short,
        long,
        default_value = "config/default.toml",
        help = "The path to the configuration file"
    )]
    config_path: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = Options::parse();
    let config: VerifierConfig = init(options.config_path.as_deref())?;

    run(&config).await
}
