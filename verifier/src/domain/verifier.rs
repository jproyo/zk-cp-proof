use num_bigint::BigUint;
use rand::Rng;
use std::ops::Deref;
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(Debug, Clone, TypedBuilder)]
pub struct Register {
    pub user: User,
    pub y1: BigUint,
    pub y2: BigUint,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct Challenge {
    pub user: User,
    pub r1: BigUint,
    pub r2: BigUint,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct ChallengeStarted {
    pub auth_id: AuthId,
    pub c: u32,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct ChallengeVerification {
    pub auth_id: AuthId,
    pub s: BigUint,
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

trait ChallengeState {}
impl ChallengeState for Challenge {}
impl ChallengeState for ChallengeVerificationResult {}
impl ChallengeState for ChallengeStarted {}
impl ChallengeState for ChallengeVerification {}

struct ChallengeTransition<S: ChallengeState> {
    state: S,
}

impl ChallengeTransition<Challenge> {
    fn change(self) -> ChallengeTransition<ChallengeStarted> {
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
    fn change(
        self,
        register: &Register,
        challenge: &Challenge,
        material: &Material,
        c: u32,
        s: u32,
    ) -> ChallengeTransition<ChallengeVerificationResult> {
        let y1 = &register.y1;
        let y2 = &register.y2;
        let r1 = &challenge.r1;
        let r2 = &challenge.r2;
        let g = &material.g;
        let h = &material.h;
        let r1_prime = g.pow(s) * y1.pow(c);
        let r2_prime = h.pow(s) * y2.pow(c);
        if r1 == &r1_prime && r2 == &r2_prime {
            ChallengeTransition {
                state: ChallengeVerificationResult::ChallengeVerified(SessionId(
                    Uuid::new_v4().to_string(),
                )),
            }
        } else {
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
pub trait ChallengeStorage {
    async fn store_user(&self, register: Register) -> anyhow::Result<()>;
    async fn store_challenge(
        &self,
        auth_id: &AuthId,
        challenge: Challenge,
    ) -> anyhow::Result<ChallengeStarted>;
    async fn get_user(&self, user: &User) -> anyhow::Result<Option<Register>>;
    async fn get_challenge(&self, auth_id: &User) -> anyhow::Result<Option<Challenge>>;
}
