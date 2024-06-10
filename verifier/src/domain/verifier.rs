use crate::grpc::zkp_auth::{
    AuthenticationAnswerRequest, AuthenticationAnswerResponse, AuthenticationChallengeRequest,
    AuthenticationChallengeResponse, RegisterRequest,
};
#[cfg(test)]
use mockall::{automock, predicate::*};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use tonic::Status;
use typed_builder::TypedBuilder;
use uuid::Uuid;
use zk_cp_protocol::protocol::cp::{AuthId, Material};

#[derive(Debug, Clone, TypedBuilder)]
pub struct Register {
    #[builder(setter(into))]
    pub user: User,
    #[builder(setter(into))]
    pub y1: BigInt,
    #[builder(setter(into))]
    pub y2: BigInt,
}

impl From<RegisterRequest> for Register {
    fn from(request: RegisterRequest) -> Self {
        Self {
            user: request.user.into(),
            y1: request.y1.into(),
            y2: request.y2.into(),
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct Challenge {
    #[builder(setter(into))]
    pub user: User,
    #[builder(setter(into))]
    pub r1: BigInt,
    #[builder(setter(into))]
    pub r2: BigInt,
}

impl From<AuthenticationChallengeRequest> for Challenge {
    fn from(request: AuthenticationChallengeRequest) -> Self {
        Self {
            user: request.user.into(),
            r1: request.r1.into(),
            r2: request.r2.into(),
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct ChallengeResponse {
    #[builder(setter(into))]
    pub auth_id: AuthId,
    #[builder(setter(into))]
    pub c: BigInt,
}

impl TryFrom<ChallengeResponse> for AuthenticationChallengeResponse {
    type Error = anyhow::Error;
    fn try_from(response: ChallengeResponse) -> anyhow::Result<Self> {
        let c = response.c.to_i32().ok_or_else(|| {
            anyhow::anyhow!("BigInt conversion error to i64 for sending result to grpc")
        })?;
        Ok(Self {
            auth_id: response.auth_id.to_string(),
            c,
        })
    }
}

impl From<AuthenticationChallengeResponse> for ChallengeResponse {
    fn from(response: AuthenticationChallengeResponse) -> Self {
        Self {
            auth_id: response.auth_id.into(),
            c: response.c.into(),
        }
    }
}

impl From<zk_cp_protocol::protocol::cp::Challenge> for ChallengeResponse {
    fn from(challenge: zk_cp_protocol::protocol::cp::Challenge) -> Self {
        Self {
            auth_id: challenge.auth_id,
            c: challenge.c,
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct ChallengeStore {
    pub challenge: Challenge,
    pub response: ChallengeResponse,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct Answer {
    #[builder(setter(into))]
    pub auth_id: AuthId,
    #[builder(setter(into))]
    pub s: BigInt,
}

impl From<AuthenticationAnswerRequest> for Answer {
    fn from(request: AuthenticationAnswerRequest) -> Self {
        Self {
            auth_id: request.auth_id.into(),
            s: request.s.into(),
        }
    }
}

#[derive(Debug, Clone, TypedBuilder, Eq, Hash, PartialEq)]
pub struct Success {
    pub session_id: SessionId,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum AnswerResult {
    Success(Success),
    Failure,
}

impl From<zk_cp_protocol::protocol::cp::VerificationResult> for AnswerResult {
    fn from(result: zk_cp_protocol::protocol::cp::VerificationResult) -> Self {
        match result {
            zk_cp_protocol::protocol::cp::VerificationResult::ChallengeVerifiedSuccess => {
                Self::Success(Success {
                    session_id: SessionId(Uuid::new_v4().to_string()),
                })
            }
            zk_cp_protocol::protocol::cp::VerificationResult::ChallengeVerificationFailed => {
                Self::Failure
            }
        }
    }
}

impl TryFrom<AnswerResult> for AuthenticationAnswerResponse {
    type Error = Status;
    fn try_from(result: AnswerResult) -> Result<Self, Self::Error> {
        match result {
            AnswerResult::Success(success) => Ok(Self {
                session_id: success.session_id.0,
            }),
            AnswerResult::Failure => Err(Status::invalid_argument("Challenge verification failed")),
        }
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct SessionId(pub String);

#[derive(Debug, Clone, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct User(pub String);

impl From<String> for User {
    fn from(s: String) -> Self {
        User(s)
    }
}

impl From<&str> for User {
    fn from(s: &str) -> Self {
        User(s.to_string())
    }
}

impl Deref for User {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Represents the public parameters (material) for a user.
#[cfg_attr(test, automock)]
pub trait Params {
    fn query(&self, user: &User) -> anyhow::Result<Option<Material>>;
}

#[cfg_attr(test, automock)]
#[async_trait::async_trait]
/// Trait representing the storage interface for the verifier.
pub trait VerifierStorage {
    /// Asynchronously stores a user's register.
    ///
    /// # Arguments
    ///
    /// * `register` - The register to store.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the operation is successful, otherwise returns an `anyhow::Error`.
    async fn store_user(&self, register: Register) -> anyhow::Result<()>;

    /// Asynchronously stores a challenge for a given authentication ID.
    ///
    /// # Arguments
    ///
    /// * `auth_id` - The authentication ID.
    /// * `challenge` - The challenge to store.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the operation is successful, otherwise returns an `anyhow::Error`.
    async fn store_challenge(
        &self,
        auth_id: &AuthId,
        challenge: ChallengeStore,
    ) -> anyhow::Result<()>;

    /// Asynchronously retrieves a user's register.
    ///
    /// # Arguments
    ///
    /// * `user` - The user to retrieve the register for.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(register))` if the user's register is found, `Ok(None)` if the user is not found,
    /// otherwise returns an `anyhow::Error`.
    async fn get_user(&self, user: &User) -> anyhow::Result<Option<Register>>;

    /// Asynchronously retrieves a challenge for a given authentication ID.
    ///
    /// # Arguments
    ///
    /// * `auth_id` - The authentication ID.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(challenge))` if the challenge is found, `Ok(None)` if the challenge is not found,
    /// otherwise returns an `anyhow::Error`.
    async fn get_challenge(&self, auth_id: &AuthId) -> anyhow::Result<Option<ChallengeStore>>;
}
