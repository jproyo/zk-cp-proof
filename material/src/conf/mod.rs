use self::settings::Settings;
use serde::Deserialize;
use tracing as int_tracing;

mod settings;
mod tracing;

pub fn init<'de, T: Deserialize<'de>>(config_path: Option<&'de str>) -> anyhow::Result<T> {
    int_tracing::init_tracing();
    ::tracing::info!("Initializing configuration with path {:?}", config_path);
    Settings::builder().path(config_path).build().init_conf()
}

pub use settings::MaterialConfig;
