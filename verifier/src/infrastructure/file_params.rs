use zk_cp_protocol::protocol::cp::Material;

use crate::conf::VerifierConfig;
use crate::domain::verifier::{Params, User};
use std::collections::HashMap;

/// Represents the parameters loaded from a file.
pub struct FileParams {
    materials: HashMap<User, Material>,
}

impl FileParams {
    /// Creates a new instance of `FileParams` by loading the materials from the specified file path.
    ///
    /// # Arguments
    ///
    /// * `conf` - The verifier configuration containing the file path to load the materials from.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `FileParams` instance if the materials are successfully loaded, or an `anyhow::Error` if an error occurs.
    pub fn new(conf: &VerifierConfig) -> anyhow::Result<Self> {
        let materials = serde_json::from_str(&std::fs::read_to_string(&conf.material_path)?)?;
        Ok(Self { materials })
    }
}

impl Params for FileParams {
    /// Retrieves the material associated with the specified user.
    ///
    /// # Arguments
    ///
    /// * `user` - The user to query the material for.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option` with the material if it exists for the user, or `None` if it doesn't exist.
    pub fn query(&self, user: &User) -> anyhow::Result<Option<Material>> {
        Ok(self.materials.get(user).cloned())
    }
}
