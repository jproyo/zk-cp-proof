use num_bigint::BigUint;
use rand::Rng;
use std::ops::Deref;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::grpc::zkp_auth::{
    AuthenticationAnswerRequest, AuthenticationChallengeRequest, AuthenticationChallengeResponse,
    RegisterRequest,
};
use crate::grpc::zkp_material::MaterialResponse;

#[derive(Debug, Clone, TypedBuilder)]
pub struct Register {
    pub user: User,
    pub y1: BigUint,
    pub y2: BigUint,
}

impl From<RegisterRequest> for Register {
    fn from(req: RegisterRequest) -> Self {
        Register {
            user: req.user.into(),
            y1: BigUint::from(req.y1 as u64),
            y2: BigUint::from(req.y2 as u64),
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct Challenge {
    pub user: User,
    pub r1: BigUint,
    pub r2: BigUint,
}

impl From<AuthenticationChallengeRequest> for Challenge {
    fn from(req: AuthenticationChallengeRequest) -> Self {
        Challenge {
            user: req.user.into(),
            r1: BigUint::from(req.r1 as u64),
            r2: BigUint::from(req.r2 as u64),
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct ChallengeStarted {
    pub auth_id: AuthId,
    pub c: u32,
}

impl From<ChallengeStarted> for AuthenticationChallengeResponse {
    fn from(resp: ChallengeStarted) -> Self {
        AuthenticationChallengeResponse {
            auth_id: resp.auth_id.to_string(),
            c: resp.c,
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct ChallengeStore {
    pub challenge: Challenge,
    pub challenge_started: ChallengeStarted,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct ChallengeVerification {
    pub auth_id: AuthId,
    pub s: u32,
}

impl From<AuthenticationAnswerRequest> for ChallengeVerification {
    fn from(req: AuthenticationAnswerRequest) -> Self {
        ChallengeVerification {
            auth_id: req.auth_id.into(),
            s: req.s,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionId(pub String);

#[derive(Debug, Clone)]
pub enum ChallengeVerificationResult {
    ChallengeVerified(SessionId),
    ChallengeVerificationFailed,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct AuthId(pub String);

impl From<String> for AuthId {
    fn from(s: String) -> Self {
        AuthId(s)
    }
}

impl From<&str> for AuthId {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}

impl Deref for AuthId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
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

#[derive(Debug, Clone, TypedBuilder)]
pub struct Material {
    pub g: BigUint,
    pub h: BigUint,
}

impl From<MaterialResponse> for Material {
    fn from(resp: MaterialResponse) -> Self {
        Material {
            g: BigUint::from(resp.g),
            h: BigUint::from(resp.h),
        }
    }
}

pub(crate) trait ChallengeState {}
impl ChallengeState for Challenge {}
impl ChallengeState for ChallengeVerificationResult {}
impl ChallengeState for ChallengeStarted {}
impl ChallengeState for ChallengeVerification {}

pub struct ChallengeTransition<S: ChallengeState> {
    state: S,
}

impl<S> ChallengeTransition<S>
where
    S: ChallengeState,
{
    pub fn into_inner(self) -> S {
        self.state
    }
}

impl<S> From<S> for ChallengeTransition<S>
where
    S: ChallengeState,
{
    fn from(state: S) -> Self {
        ChallengeTransition { state }
    }
}

impl ChallengeTransition<Challenge> {
    pub fn change(self) -> ChallengeTransition<ChallengeStarted> {
        let mut rng = rand::thread_rng();
        let random_c: u32 = rng.gen();
        ChallengeTransition {
            state: ChallengeStarted {
                auth_id: AuthId(Uuid::new_v4().to_string()),
                c: random_c,
            },
        }
    }
}

impl ChallengeTransition<ChallengeVerification> {
    pub fn change(
        self,
        register: &Register,
        challenge: &ChallengeStore,
        material: &Material,
        s: u32,
    ) -> ChallengeTransition<ChallengeVerificationResult> {
        let c = challenge.challenge_started.c;
        let challenge = &challenge.challenge;
        let y1 = &register.y1;
        let y2 = &register.y2;
        let r1 = &challenge.r1;
        let r2 = &challenge.r2;
        let g = &material.g;
        let h = &material.h;
        let r1_prime = g.pow(s) * y1.pow(c);
        let r2_prime = h.pow(s) * y2.pow(c);
        if r1 == &r1_prime && r2 == &r2_prime {
            tracing::info!("Challenge verified successfully");
            ChallengeTransition {
                state: ChallengeVerificationResult::ChallengeVerified(SessionId(
                    Uuid::new_v4().to_string(),
                )),
            }
        } else {
            tracing::info!(
                "Challenge verification failed due to mismatch - expected: {:?}, actual: {:?}",
                (r1_prime, r2_prime),
                (r1, r2)
            );
            ChallengeTransition {
                state: ChallengeVerificationResult::ChallengeVerificationFailed,
            }
        }
    }
}

#[async_trait::async_trait]
pub trait MaterialRegistry {
    async fn query(&self, user: &User) -> anyhow::Result<Option<Material>>;
}

#[async_trait::async_trait]
pub trait VerifierStorage {
    async fn store_user(&self, register: Register) -> anyhow::Result<()>;
    async fn store_challenge(
        &self,
        auth_id: &AuthId,
        challenge: ChallengeStore,
    ) -> anyhow::Result<()>;
    async fn get_user(&self, user: &User) -> anyhow::Result<Option<Register>>;
    async fn get_challenge(&self, auth_id: &AuthId) -> anyhow::Result<Option<ChallengeStore>>;
}
