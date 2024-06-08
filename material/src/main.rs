use clap::Parser;
use zk_material::conf::{init, MaterialConfig};
use zk_material::grpc::server::run;

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
    let config: MaterialConfig = init(options.config_path.as_deref())?;

    run(&config).await
}
