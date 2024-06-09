use self::settings::Settings;
use serde::Deserialize;

mod settings;

pub fn init<'de, T: Deserialize<'de>>(config_path: Option<&'de str>) -> anyhow::Result<T> {
    init_tracing();
    tracing::info!("Initializing configuration with path {:?}", config_path);
    Settings::builder().path(config_path).build().init_conf()
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_line_number(true)
        .with_file(true)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();
}

pub use settings::VerifierConfig;
