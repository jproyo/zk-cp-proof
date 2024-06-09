use dashmap::DashMap;
use crate::domain::material::{Material, MaterialStorage, User};

/// In-memory storage implementation for materials.
pub(crate) struct MemStorage {
    pub(crate) materials: DashMap<User, Material>,
}

impl MemStorage {
    /// Creates a new instance of `MemStorage`.
    pub(crate) fn new() -> Self {
        Self {
            materials: DashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl MaterialStorage for MemStorage {
    /// Retrieves the material associated with the given user.
    ///
    /// # Arguments
    ///
    /// * `user` - The user for which to retrieve the material.
    ///
    /// # Returns
    ///
    /// An `Option` containing the material if found, or `None` if not found.
    async fn get(&self, user: &User) -> anyhow::Result<Option<Material>> {
        Ok(self.materials.get(user).map(|m| m.value().clone()))
    }

    /// Stores the material for the given user.
    ///
    /// # Arguments
    ///
    /// * `user` - The user for which to store the material.
    /// * `material` - The material to store.
    ///
    /// # Returns
    ///
    /// An `Ok` result if the material was stored successfully, or an `Err` if an error occurred.
    async fn store(&self, user: User, material: Material) -> anyhow::Result<()> {
        self.materials.insert(user, material);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::material::Material;

    #[tokio::test]
    async fn test_mem_storage() {
        let storage = MemStorage::new();
        let user = "test_user".into();
        let material = Material::builder().g(1u64.into()).h(2u64.into()).build();

        let stored_material = storage.get(&user).await.unwrap();
        assert!(stored_material.is_none());

        storage.store(user.clone(), material.clone()).await.unwrap();
        let stored_material = storage.get(&user).await.unwrap().unwrap();
        assert_eq!(material, stored_material);
    }
}
