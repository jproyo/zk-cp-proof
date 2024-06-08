use async_trait::async_trait;
use typed_builder::TypedBuilder;

use crate::domain::verifier::{
    Challenge, ChallengeStarted, ChallengeStore, ChallengeTransition, ChallengeVerification,
    ChallengeVerificationResult, MaterialRegistry, Register, VerifierStorage,
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
    S: VerifierStorage + Send + Sync,
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
        tracing::info!("Creating challenge: {:?}", challenge);
        let created = <Challenge as Into<ChallengeTransition<Challenge>>>::into(challenge.clone())
            .change()
            .into_inner();
        tracing::info!("Challenge created: {:?} .... Storing", created);
        self.storage
            .store_challenge(
                &created.auth_id,
                ChallengeStore {
                    challenge,
                    challenge_started: created.clone(),
                },
            )
            .await
            .map(|_| created)
    }

    async fn verify_challenge(
        &self,
        challenge_ver: ChallengeVerification,
    ) -> anyhow::Result<ChallengeVerificationResult> {
        let challenge = self
            .storage
            .get_challenge(&challenge_ver.auth_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Challenge not found"))?;
        let material = self
            .material
            .query(&challenge.challenge.user)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Material not found for user: {:?}",
                    challenge.challenge.user
                )
            })?;
        let user = self
            .storage
            .get_user(&challenge.challenge.user)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;
        let c = challenge.challenge_started.c;
        let s = challenge_ver.s;
        let result =
            <ChallengeVerification as Into<ChallengeTransition<ChallengeVerification>>>::into(
                challenge_ver,
            )
            .change(&user, &challenge.challenge, &material, c, s)
            .into_inner();
        Ok(result)
    }
}

impl<M, S> VerifierApplication<M, S>
where
    M: MaterialRegistry,
    S: VerifierStorage,
{
    pub fn new(material: M, storage: S) -> Self {
        Self { material, storage }
    }
}

//impl VerifierApplication<GrpcClientMaterialRegistry, MemStorage> {
//    pub fn new_default() -> Self {
//        Self::new(DefaultMaterialGenerator, MemStorage::new())
//    }
//}
