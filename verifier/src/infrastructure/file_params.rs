use zk_cp_protocol::protocol::cp::Material;

use crate::conf::VerifierConfig;
use crate::domain::verifier::{Params, User};
use std::collections::HashMap;

pub struct FileParams {
    materials: HashMap<User, Material>,
}

impl FileParams {
    pub fn new(conf: &VerifierConfig) -> anyhow::Result<Self> {
        let materials = serde_json::from_str(&std::fs::read_to_string(&conf.material_path)?)?;
        Ok(Self { materials })
    }
}

impl Params for FileParams {
    fn query(&self, user: &User) -> anyhow::Result<Option<Material>> {
        Ok(self.materials.get(user).cloned())
    }
}
