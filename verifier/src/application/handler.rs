use async_trait::async_trait;
use typed_builder::TypedBuilder;

use crate::domain::verifier::{
    Challenge, ChallengeStarted, ChallengeStorage, ChallengeVerification,
    ChallengeVerificationResult, MaterialRegistry, Register,
};

#[async_trait]
pub trait VerifierService {
    async fn register(&self, register: Register) -> anyhow::Result<()>;

    async fn create_challenge(&self, challenge: Challenge) -> anyhow::Result<ChallengeStarted>;

    async fn verify_challenge(
        &self,
        challenge: ChallengeVerification,
    ) -> anyhow::Result<ChallengeVerificationResult>;
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct VerifierApplication<M, S> {
    material: M,
    storage: S,
}

#[async_trait]
impl<M, S> VerifierService for VerifierApplication<M, S>
where
    M: MaterialRegistry + Send + Sync,
    S: ChallengeStorage + Send + Sync,
{
    async fn register(&self, register: Register) -> anyhow::Result<()> {
        tracing::info!("Registering user: {:?}", register);
        let material = self.material.query(&register.user).await?;

        if material.is_none() {
            return Err(anyhow::anyhow!(
                "User material not found. You should generate material first."
            ));
        }

        tracing::info!("User material found. Registering user {:?}", register);
        self.storage.store_user(register).await
    }

    async fn create_challenge(&self, challenge: Challenge) -> anyhow::Result<ChallengeStarted> {
        self.storage
            .store_challenge(&challenge.auth_id, challenge)
            .await
    }

    async fn verify_challenge(
        &self,
        challenge: ChallengeVerification,
    ) -> anyhow::Result<ChallengeVerificationResult> {
        let material = self.material.query(&challenge.auth_id.user).await?;
        let challenge = self.storage.load_challenge(&challenge.auth_id).await?;
        let challenge = challenge.verify(&material, &challenge)?;
        self.storage
            .store_challenge(&challenge.auth_id, challenge)
            .await
    }
}
