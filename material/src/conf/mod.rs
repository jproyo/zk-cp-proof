use std::error::Error;

use serde::Deserialize;

use self::settings::Settings;

mod settings;
mod tracing;

pub fn init<'de, T: Deserialize<'de>>(
    config_path: Option<&'de str>,
) -> Result<T, Box<dyn Error + 'static>> {
    tracing::init_tracing();
    Settings::builder()
        .path(config_path)
        .build()
        .init_conf()
        .map_err(Into::into)
}
