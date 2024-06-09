use crate::domain::material::{Material, MaterialGenerator, MaterialStorage, PrimeOrder, User};
use crate::infrastructure::generator::DefaultMaterialGenerator;
use crate::infrastructure::mem_storage::MemStorage;
use async_trait::async_trait;
use typed_builder::TypedBuilder;

/// Trait representing a material service.
#[async_trait]
pub trait MaterialService {
    /// Creates a new material.
    ///
    /// # Arguments
    ///
    /// * `user` - The user creating the material.
    /// * `q` - An optional prime order.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the created `Material` if successful, or an `anyhow::Error` otherwise.
    async fn create_material(
        &self,
        user: &User,
        q: Option<PrimeOrder>,
        p: Option<PrimeOrder>,
    ) -> anyhow::Result<Material>;

    /// Retrieves the material for the given user.
    ///
    /// # Arguments
    ///
    /// * `user` - The user to retrieve the material for.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing an `Option` of the retrieved `Material` if it exists, or an `anyhow::Error` otherwise.
    async fn get_material(&self, user: &User) -> anyhow::Result<Option<Material>>;
}

/// Represents a Material application.
///
/// This struct holds a generator and a storage, which are used by the application.
#[derive(Debug, Clone, TypedBuilder)]
pub struct MaterialApplication<G, S> {
    generator: G,
    storage: S,
}

#[async_trait]
impl<G, S> MaterialService for MaterialApplication<G, S>
where
    G: MaterialGenerator + Send + Sync,
    S: MaterialStorage + Send + Sync,
{
    async fn create_material(
        &self,
        user: &User,
        q: Option<PrimeOrder>,
        p: Option<PrimeOrder>,
    ) -> anyhow::Result<Material> {
        tracing::info!("Creating material for user {:?}", user);
        let material = self.storage.get(user).await?;
        if let Some(material) = material {
            tracing::warn!(
                "Material already exists for user {:?}. Returning existing",
                user
            );
            return Ok(material);
        }

        tracing::info!("Generating material for user {:?}", user);
        let material = self.generator.generate(q, p).await?;

        tracing::info!("Storing material {:?} for user {:?}", material, user);
        self.storage
            .store(user.to_owned(), material.clone())
            .await?;
        Ok(material)
    }

    async fn get_material(&self, user: &User) -> anyhow::Result<Option<Material>> {
        tracing::info!("Getting material for user {:?}", user);
        self.storage.get(user).await
    }
}

impl<G, S> MaterialApplication<G, S>
where
    G: MaterialGenerator,
    S: MaterialStorage,
{
    pub fn new(generator: G, storage: S) -> Self {
        Self { generator, storage }
    }
}

impl MaterialApplication<DefaultMaterialGenerator, MemStorage> {
    pub fn new_default() -> Self {
        Self::new(DefaultMaterialGenerator, MemStorage::new())
    }
}

#[cfg(test)]
/// Module containing unit tests for the `MaterialApplication` struct.
mod tests {
    use super::*;

    /// Test case for creating and retrieving a material for a user.
    #[tokio::test]
    async fn test_material_application() {
        let generator = DefaultMaterialGenerator;
        let storage = MemStorage::new();
        let application = MaterialApplication::new(generator, storage);

        let user = "test_user".into();
        let material = application
            .create_material(&user, None, None)
            .await
            .unwrap();
        assert_eq!(
            material,
            application.get_material(&user).await.unwrap().unwrap()
        );
    }

    /// Test case for creating and retrieving a material for an existing user.
    #[tokio::test]
    async fn test_material_application_existing() {
        let generator = DefaultMaterialGenerator;
        let storage = MemStorage::new();
        let application = MaterialApplication::new(generator, storage);

        let user = "test_user".into();
        let material = application
            .create_material(&user, None, None)
            .await
            .unwrap();
        assert_eq!(
            material,
            application.get_material(&user).await.unwrap().unwrap()
        );

        let material = application
            .create_material(&user, None, None)
            .await
            .unwrap();
        assert_eq!(
            material,
            application.get_material(&user).await.unwrap().unwrap()
        );
    }

    /// Test case for creating and retrieving materials for different users.
    #[tokio::test]
    async fn test_material_application_different_users() {
        let generator = DefaultMaterialGenerator;
        let storage = MemStorage::new();
        let application = MaterialApplication::new(generator, storage);

        let user1 = "test_user1".into();
        let user2 = "test_user2".into();
        let material1 = application
            .create_material(&user1, None, None)
            .await
            .unwrap();
        let material2 = application
            .create_material(&user2, None, None)
            .await
            .unwrap();
        assert_eq!(
            material1,
            application.get_material(&user1).await.unwrap().unwrap()
        );
        assert_eq!(
            material2,
            application.get_material(&user2).await.unwrap().unwrap()
        );
    }

    /// Test case for creating and retrieving different materials for the same user.
    #[tokio::test]
    async fn test_material_application_different_materials() {
        let generator = DefaultMaterialGenerator;
        let storage = MemStorage::new();
        let application = MaterialApplication::new(generator, storage);

        let user = "test_user".into();
        let material1 = application
            .create_material(&user, None, None)
            .await
            .unwrap();
        let material2 = application
            .create_material(&user, None, None)
            .await
            .unwrap();
        assert_eq!(material1, material2);
    }

    /// Test case for retrieving a non-existent material for a user.
    #[tokio::test]
    async fn test_material_application_get_non_existent() {
        let generator = DefaultMaterialGenerator;
        let storage = MemStorage::new();
        let application = MaterialApplication::new(generator, storage);

        let user = "test_user".into();
        let material = application.get_material(&user).await.unwrap();
        assert!(material.is_none());
    }

    /// Test case for storing and retrieving a material.
    #[tokio::test]
    async fn test_material_application_store() {
        let generator = DefaultMaterialGenerator;
        let storage = MemStorage::new();
        let user: User = "test_user".into();
        let material = Material::builder()
            .g(1u64.into())
            .h(2u64.into())
            .p(7u64.into())
            .q(11u64.into())
            .build();
        storage.store(user.clone(), material.clone()).await.unwrap();
        let application = MaterialApplication::new(generator, storage);

        let stored_material = application.get_material(&user).await.unwrap().unwrap();
        assert_eq!(material, stored_material);
    }

    /// Test case for storing multiple materials for the same user and retrieving the latest one.
    #[tokio::test]
    async fn test_material_application_store_existing() {
        let generator = DefaultMaterialGenerator;
        let storage = MemStorage::new();
        let user: User = "test_user".into();
        let material_1 = Material::builder()
            .g(1u64.into())
            .h(2u64.into())
            .p(11u64.into())
            .q(7u64.into())
            .build();
        let material_2 = Material::builder()
            .g(3u64.into())
            .h(4u64.into())
            .p(11u64.into())
            .q(7u64.into())
            .build();
        storage
            .store(user.clone(), material_1.clone())
            .await
            .unwrap();
        storage
            .store(user.clone(), material_2.clone())
            .await
            .unwrap();
        let application = MaterialApplication::new(generator, storage);

        let stored_material = application.get_material(&user).await.unwrap().unwrap();
        assert_eq!(material_2, stored_material);

        let stored_material = application.get_material(&user).await.unwrap().unwrap();
        assert_eq!(material_2, stored_material);
    }
}
