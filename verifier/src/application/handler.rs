use crate::conf::VerifierConfig;
use crate::domain::verifier::{
    Answer, AnswerResult, Challenge, ChallengeResponse, ChallengeStore, Params, Register,
    VerifierStorage,
};
use crate::infrastructure::file_params::FileParams;
use crate::infrastructure::mem_storage::MemStorage;
use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};
use typed_builder::TypedBuilder;
use zk_cp_protocol::protocol::cp::{Material, ProtocolState, ProtocolTransition, Verification};

/// Trait representing a verifier service.
#[cfg_attr(test, automock)]
#[async_trait]
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

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;

    use super::*;
    use crate::domain::verifier::{MockParams, MockVerifierStorage};

    #[tokio::test]
    async fn test_register() {
        let mut params = MockParams::new();
        params
            .expect_query()
            .times(1)
            .returning(|_| Ok(Some(Material::default())));
        let mut storage = MockVerifierStorage::new();
        storage.expect_store_user().times(1).returning(|_| Ok(()));
        let app = VerifierApplication::new(params, storage);
        let register = Register::builder()
            .user("test".into())
            .y1(BigInt::from(11))
            .y2(BigInt::from(13))
            .build();
        assert!(app.register(register).await.is_ok());
    }

    #[tokio::test]
    async fn test_register_error() {
        let mut params = MockParams::new();
        params.expect_query().times(1).returning(|_| Ok(None));
        let storage = MockVerifierStorage::new();
        let app = VerifierApplication::new(params, storage);
        let register = Register::builder()
            .user("test".into())
            .y1(BigInt::from(11))
            .y2(BigInt::from(13))
            .build();
        assert!(app.register(register).await.is_err());
    }

    #[tokio::test]
    async fn test_create_challenge() {
        let mut params = MockParams::new();
        params
            .expect_query()
            .times(1)
            .returning(|_| Ok(Some(Material::default())));
        let mut storage = MockVerifierStorage::new();
        storage
            .expect_store_challenge()
            .times(1)
            .returning(|_, _| Ok(()));
        let app = VerifierApplication::new(params, storage);
        let challenge = Challenge::builder()
            .user("test".into())
            .r1(BigInt::from(11))
            .r2(BigInt::from(13))
            .build();
        assert!(app.create_challenge(challenge).await.is_ok());
    }

    #[tokio::test]
    async fn test_create_challenge_error() {
        let mut params = MockParams::new();
        params.expect_query().times(1).returning(|_| Ok(None));
        let storage = MockVerifierStorage::new();
        let app = VerifierApplication::new(params, storage);
        let challenge = Challenge::builder()
            .user("test".into())
            .r1(BigInt::from(11))
            .r2(BigInt::from(13))
            .build();
        assert!(app.create_challenge(challenge).await.is_err());
    }

    #[tokio::test]
    async fn test_verify_challenge() {
        let mut params = MockParams::new();
        let material = Material::builder()
            .p(BigInt::from(1))
            .q(BigInt::from(1))
            .g(BigInt::from(1))
            .h(BigInt::from(1))
            .build();
        params
            .expect_query()
            .times(1)
            .returning(move |_| Ok(Some(material.clone())));
        let mut storage = MockVerifierStorage::new();
        storage.expect_get_challenge().times(1).returning(|_| {
            Ok(Some(
                ChallengeStore::builder()
                    .challenge(
                        Challenge::builder()
                            .r1(BigInt::from(1))
                            .r2(BigInt::from(1))
                            .user("test".into())
                            .build(),
                    )
                    .response(
                        ChallengeResponse::builder()
                            .auth_id("test".into())
                            .c(BigInt::from(1))
                            .build(),
                    )
                    .build(),
            ))
        });
        storage.expect_get_user().times(1).returning(|_| {
            Ok(Some(
                Register::builder()
                    .y1(BigInt::from(1))
                    .y2(BigInt::from(1))
                    .user("test".into())
                    .build(),
            ))
        });
        let app = VerifierApplication::new(params, storage);
        let answer = Answer::builder()
            .auth_id("test".into())
            .s(BigInt::from(1))
            .build();
        assert!(app.verify_challenge(answer).await.is_ok());
    }

    #[tokio::test]
    async fn test_verify_challenge_error() {
        let mut params = MockParams::new();
        params
            .expect_query()
            .times(1)
            .returning(|_| Ok(Some(Material::default())));
        let mut storage = MockVerifierStorage::new();
        storage.expect_get_challenge().times(1).returning(|_| {
            Ok(Some(
                ChallengeStore::builder()
                    .challenge(
                        Challenge::builder()
                            .r1(BigInt::from(18))
                            .r2(BigInt::from(16))
                            .user("test".into())
                            .build(),
                    )
                    .response(
                        ChallengeResponse::builder()
                            .auth_id("test".into())
                            .c(BigInt::from(87))
                            .build(),
                    )
                    .build(),
            ))
        });
        storage.expect_get_user().times(1).returning(|_| {
            Ok(Some(
                Register::builder()
                    .y1(BigInt::from(22))
                    .y2(BigInt::from(54))
                    .user("test".into())
                    .build(),
            ))
        });
        let app = VerifierApplication::new(params, storage);
        let answer = Answer::builder()
            .auth_id("test".into())
            .s(BigInt::from(11))
            .build();
        let result = app.verify_challenge(answer).await.unwrap();
        assert_eq!(result, AnswerResult::Failure);
    }
}
