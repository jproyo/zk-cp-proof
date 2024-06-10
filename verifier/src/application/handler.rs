use crate::conf::VerifierConfig;
use crate::domain::verifier::{
    Answer, AnswerResult, Challenge, ChallengeResponse, ChallengeStore, Params, Register,
    VerifierStorage,
};
use crate::infrastructure::file_params::FileParams;
use crate::infrastructure::mem_storage::MemStorage;
use async_trait::async_trait;
use typed_builder::TypedBuilder;
use zk_cp_protocol::protocol::cp::{Material, ProtocolState, ProtocolTransition, Verification};

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
    async fn create_challenge(&self, challenge: Challenge) -> anyhow::Result<ChallengeResponse>;

    /// Asynchronously verifies a challenge.
    ///
    /// # Arguments
    ///
    /// * `challenge` - The challenge verification information.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the verification result or an error.
    async fn verify_challenge(&self, challenge: Answer) -> anyhow::Result<AnswerResult>;
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

    async fn create_challenge(&self, challenge: Challenge) -> anyhow::Result<ChallengeResponse> {
        tracing::info!("Creating challenge: {:?}", challenge);
        let material = self
            .params
            .query(&challenge.user)?
            .ok_or_else(|| anyhow::anyhow!("Material not found"))?;

        let created = <Material as Into<ProtocolState<_>>>::into(material)
            .change()
            .into_inner();
        let response: ChallengeResponse = created.into();
        let store = ChallengeStore::builder()
            .challenge(challenge.clone())
            .response(response.clone())
            .build();
        tracing::info!("Challenge created: {:?} .... Storing", store);
        self.storage
            .store_challenge(&response.auth_id, store)
            .await
            .map(|_| response)
    }

    async fn verify_challenge(&self, answer: Answer) -> anyhow::Result<AnswerResult> {
        tracing::info!("Verifying challenge: {:?}", answer);
        let challenge = self
            .storage
            .get_challenge(&answer.auth_id)
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

        let verification: ProtocolState<Verification> = Verification::builder()
            .material(material)
            .c(challenge.response.c)
            .y1(user.y1)
            .y2(user.y2)
            .r1(challenge.challenge.r1)
            .r2(challenge.challenge.r2)
            .s(answer.s)
            .build()
            .into();

        tracing::info!("Verifying challenge: {:?}", verification);

        let result = verification.change().into_inner();

        tracing::info!("Challenge verification Result: {:?}", result);
        Ok(result.into())
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
