use config::{Config, File};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder, Deserialize, Serialize, Clone, Default)]
pub struct VerifierConfig {
    pub port: u16,
    pub response_timeout_in_secs: u64,
    pub material_path: String,
}

#[derive(TypedBuilder)]
pub struct Settings<'a> {
    path: Option<&'a str>,
}

impl<'a> Settings<'a> {
    /// Creates a new instance of `Settings`.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be created or deserialized.
    pub fn init_conf<'de, T: Deserialize<'de>>(&self) -> anyhow::Result<T> {
        let mut s = Config::builder()
            .add_source(File::with_name("config/default").required(cfg!(not(test))))
            .add_source(File::with_name("config/verifier").required(false))
            .add_source(File::with_name("/etc/config/verifier.toml").required(false));

        if let Some(path) = self.path {
            s = s.add_source(File::with_name(path).required(false))
        };

        let s = s
            .add_source(
                config::Environment::default()
                    .prefix("ZK_VERIFIER")
                    .separator("_")
                    .list_separator(","),
            )
            .build()?;

        let settings: T = s.try_deserialize()?;

        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        let conf: VerifierConfig = Settings::builder()
            .path(Some("config/default"))
            .build()
            .init_conf()
            .unwrap();

        assert_eq!(conf.port, 50_000);
        assert_eq!(conf.response_timeout_in_secs, 60);
        assert_eq!(conf.material_path, "./config/users.json");
    }
}
