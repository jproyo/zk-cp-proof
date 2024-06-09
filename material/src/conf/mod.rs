/// This module contains the configuration settings for the material crate.
/// It provides functions for initializing and accessing the configuration.
/// The main entry point is the `init` function, which initializes the configuration
/// based on the provided `config_path`.
/// The `MaterialConfig` type is also re-exported for convenience.
use self::settings::Settings;
use serde::Deserialize;

mod settings;

/// Initializes the configuration with the specified `config_path`.
/// The configuration is deserialized from the specified path using the `Deserialize` trait.
/// Returns a `Result` containing the deserialized configuration on success, or an `anyhow::Error` on failure.
pub fn init<'de, T: Deserialize<'de>>(config_path: Option<&'de str>) -> anyhow::Result<T> {
    init_tracing();
    tracing::info!("Initializing configuration with path {:?}", config_path);
    Settings::builder().path(config_path).build().init_conf()
}

/// Initializes the tracing subsystem.
fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_line_number(true)
        .with_file(true)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();
}

pub use settings::MaterialConfig;
