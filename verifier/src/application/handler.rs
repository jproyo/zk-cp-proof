use crate::conf::VerifierConfig;
use crate::domain::verifier::{
    Challenge, ChallengeStarted, ChallengeStore, ChallengeTransition, ChallengeVerification,
    ChallengeVerificationResult, Params, Register, VerifierStorage,
};
use crate::infrastructure::file_params::FileParams;
use crate::infrastructure::mem_storage::MemStorage;
use async_trait::async_trait;
use typed_builder::TypedBuilder;

#[async_trait]
/// Trait representing a verifier service.
pub trait VerifierService {
    /// Asynchronously registers a user.
    ///
    /// # Arguments
    ///
    /// * `register` - The registration information.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure.
    async fn register(&self, register: Register) -> anyhow::Result<()>;

    /// Asynchronously creates a challenge for the user.
    ///
    /// # Arguments
    ///
    /// * `challenge` - The challenge information.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the started challenge or an error.
    async fn create_challenge(&self, challenge: Challenge) -> anyhow::Result<ChallengeStarted>;

    /// Asynchronously verifies a challenge.
    ///
    /// # Arguments
    ///
    /// * `challenge` - The challenge verification information.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the verification result or an error.
    async fn verify_challenge(
        &self,
        challenge: ChallengeVerification,
    ) -> anyhow::Result<ChallengeVerificationResult>;
}

/// Represents a Verifier Application.
#[derive(Debug, Clone, TypedBuilder)]
pub struct VerifierApplication<M, S> {
    params: M,
    storage: S,
}

#[async_trait]
impl<M, S> VerifierService for VerifierApplication<M, S>
where
    M: Params + Send + Sync,
    S: VerifierStorage + Send + Sync,
{
    async fn register(&self, register: Register) -> anyhow::Result<()> {
        tracing::info!("Registering user: {:?}", register);
        let material = self.params.query(&register.user)?;

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
        tracing::info!("Verifying challenge: {:?}", challenge_ver);
        let challenge = self
            .storage
            .get_challenge(&challenge_ver.auth_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Challenge not found"))?;
        let material = self
            .params
            .query(&challenge.challenge.user)?
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
        let s = challenge_ver.s;
        let result =
            <ChallengeVerification as Into<ChallengeTransition<ChallengeVerification>>>::into(
                challenge_ver,
            )
            .change(&user, &challenge, &material, s)
            .into_inner();
        Ok(result)
    }
}

impl<M, S> VerifierApplication<M, S>
where
    M: Params,
    S: VerifierStorage,
{
    pub fn new(params: M, storage: S) -> Self {
        Self { params, storage }
    }
}

impl VerifierApplication<FileParams, MemStorage> {
    pub fn new_with_config(conf: &VerifierConfig) -> anyhow::Result<Self> {
        let material = FileParams::new(conf)?;
        Ok(Self::new(material, MemStorage::new()))
    }
}
