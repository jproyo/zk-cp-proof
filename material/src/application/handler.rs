use async_trait::async_trait;
use typed_builder::TypedBuilder;

use crate::domain::material::{Material, MaterialGenerator, MaterialStorage, PrimeOrder, User};
use crate::infrastructure::generator::DefaultMaterialGenerator;
use crate::infrastructure::mem_storage::MemStorage;

#[async_trait]
pub trait MaterialService {
    async fn create_material(&self, user: &User, q: Option<PrimeOrder>)
        -> anyhow::Result<Material>;

    async fn get_material(&self, user: &User) -> anyhow::Result<Option<Material>>;
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct MaterialApplication<G: MaterialGenerator, S: MaterialStorage> {
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
        let material = self.generator.generate(q).await?;

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
