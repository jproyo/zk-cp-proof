use crate::grpc::zkp_auth::{
    AuthenticationAnswerRequest, AuthenticationChallengeRequest, AuthenticationChallengeResponse,
    RegisterRequest,
};
use num_bigint::BigInt;
use num_traits::{Euclid, One};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(Debug, Clone, TypedBuilder)]
pub struct Register {
    pub user: User,
    pub y1: i64,
    pub y2: i64,
}

impl From<RegisterRequest> for Register {
    fn from(req: RegisterRequest) -> Self {
        Register {
            user: req.user.into(),
            y1: req.y1,
            y2: req.y2,
        }
    }
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

#[derive(Debug, Clone, TypedBuilder, Serialize, Deserialize)]
pub struct Material {
    pub g: i64,
    pub h: i64,
    pub q: i64,
    pub p: i64,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct Challenge {
    pub user: User,
    pub r1: i64,
    pub r2: i64,
}

impl From<AuthenticationChallengeRequest> for Challenge {
    fn from(req: AuthenticationChallengeRequest) -> Self {
        Challenge {
            user: req.user.into(),
            r1: req.r1,
            r2: req.r2,
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct ChallengeStarted {
    pub auth_id: AuthId,
    pub c: i32,
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
    pub s: i32,
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

/// Trait Type State Pattern
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
    /// Changes the state of the challenge to `ChallengeStarted`.
    ///
    /// This method generates a random value `c` and creates a new `ChallengeTransition`
    /// with the state set to `ChallengeStarted` and the `auth_id` and `c` values initialized.
    ///
    /// # Returns
    ///
    /// Returns a new `ChallengeTransition` with the state set to `ChallengeStarted`.
    pub fn change(self) -> ChallengeTransition<ChallengeStarted> {
        let mut rng = rand::thread_rng();
        let random_c: i32 = rng.gen_range(0..=1000);
        ChallengeTransition {
            state: ChallengeStarted {
                auth_id: AuthId(Uuid::new_v4().to_string()),
                c: random_c,
            },
        }
    }
}

/// Implements the `change` method for the `ChallengeTransition<ChallengeVerification>` struct.
/// This method is used to change the state of the challenge transition based on the provided parameters.
impl ChallengeTransition<ChallengeVerification> {
    /// Changes the state of the challenge transition.
    ///
    /// # Arguments
    ///
    /// * `self` - The current `ChallengeTransition<ChallengeVerification>` instance.
    /// * `register` - The register containing the values used in the challenge.
    /// * `challenge` - The challenge store containing the challenge and its metadata.
    /// * `material` - The material containing the cryptographic parameters.
    /// * `s` - The value used in the calculation of `r1_prime` and `r2_prime`.
    /// * `p` - The prime order.
    /// # Returns
    ///
    /// Returns a new `ChallengeTransition<ChallengeVerificationResult>` instance with the updated state.
    pub fn change(
        self,
        register: &Register,
        challenge: &ChallengeStore,
        material: &Material,
        s: &BigInt,
    ) -> ChallengeTransition<ChallengeVerificationResult> {
        let c: BigInt = challenge.challenge_started.c.into();
        let challenge = &challenge.challenge;
        let y1: BigInt = register.y1.into();
        let y2: BigInt = register.y2.into();
        let r1: BigInt = challenge.r1.into();
        let r2: BigInt = challenge.r2.into();
        let g: BigInt = material.g.into();
        let h: BigInt = material.h.into();
        let p: BigInt = material.p.into();
        let r1_prime = (g.modpow(s, &p) * y1.modpow(&c, &p)).modpow(&BigInt::one(), &p);
        let r2_prime = (h.modpow(s, &p) * y2.modpow(&c, &p)).modpow(&BigInt::one(), &p);
        if r1 == r1_prime && r2 == r2_prime {
            tracing::info!("Challenge verified successfully");
            ChallengeTransition {
                state: ChallengeVerificationResult::ChallengeVerified(SessionId(
                    Uuid::new_v4().to_string(),
                )),
            }
        } else {
            println!(
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

pub trait Params {
    fn query(&self, user: &User) -> anyhow::Result<Option<Material>>;
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::RandBigInt;
    use num_primes::Generator;
    use num_traits::{One, ToPrimitive};

    fn common_params() -> (Register, ChallengeStarted, ChallengeStore, Material, BigInt) {
        let p_prime = Generator::safe_prime(16);
        let p: BigInt = p_prime.to_u64().unwrap().into();
        let q: BigInt = (p.clone() - BigInt::one()) / 2;
        let g: BigInt = 7.into();
        let h: BigInt = g.modpow(&BigInt::from(11), &p);
        let x: BigInt = 3.into();
        let c = rand::thread_rng().gen_bigint_range(&2.into(), &(&q - 1));
        let y1 = g.modpow(&x, &p);
        let y2 = h.modpow(&x, &p);
        let register = Register::builder()
            .user(User::from("test_user"))
            .y1(y1.to_i64().unwrap())
            .y2(y2.to_i64().unwrap())
            .build();

        let k = rand::thread_rng().gen_bigint_range(&2.into(), &(&p - 2));

        let r1 = g.modpow(&k, &p);
        let r2 = h.modpow(&k, &p);

        let challenge = Challenge::builder()
            .user(User::from("test_user"))
            .r1(r1.to_i64().unwrap())
            .r2(r2.to_i64().unwrap())
            .build();

        let challenge_started = ChallengeStarted::builder()
            .auth_id(AuthId::from("test_auth_id"))
            .c(c.to_i32().unwrap())
            .build();

        let cx = c * &x;
        let s = if k > cx {
            (k - cx).modpow(&BigInt::one(), &q)
        } else {
            &q - (cx - k).modpow(&BigInt::one(), &q)
        };

        //let s = (&k - c * &x).rem_euclid(&q);
        let challenge_store = ChallengeStore::builder()
            .challenge(challenge.clone())
            .challenge_started(challenge_started.clone())
            .build();

        let material = Material::builder()
            .g(g.to_i64().unwrap())
            .h(h.to_i64().unwrap())
            .p(p.to_i64().unwrap())
            .q(q.to_i64().unwrap())
            .build();

        let transition = ChallengeTransition::<Challenge>::from(challenge)
            .change()
            .into_inner();

        assert_ne!(transition.auth_id.to_string(), "");
        assert_ne!(transition.c, challenge_started.c);

        (register, challenge_started, challenge_store, material, s)
    }

    #[tokio::test]
    async fn test_challenge_transition_change() {
        let (register, challenge_started, challenge_store, material, s) = common_params();
        let challenge_verification = ChallengeVerification::builder()
            .auth_id(challenge_started.auth_id)
            .s(s.to_i32().unwrap())
            .build();

        let transition = ChallengeTransition::<ChallengeVerification>::from(challenge_verification)
            .change(&register, &challenge_store, &material, &s)
            .into_inner();

        match transition {
            ChallengeVerificationResult::ChallengeVerified(session_id) => {
                assert_ne!(session_id.0, "");
            }
            ChallengeVerificationResult::ChallengeVerificationFailed => {
                unreachable!("Challenge verification failed unexpectedly");
            }
        }
    }

    #[tokio::test]
    async fn test_challenge_transition_change_failed() {
        let (register, challenge_started, challenge_store, material, s) = common_params();
        let challenge_verification = ChallengeVerification::builder()
            .auth_id(challenge_started.auth_id)
            .s(s.to_i32().unwrap())
            .build();

        let transition = ChallengeTransition::<ChallengeVerification>::from(challenge_verification)
            .change(&register, &challenge_store, &material, &(s + 1))
            .into_inner();

        match transition {
            ChallengeVerificationResult::ChallengeVerified(_) => {
                unreachable!("Challenge verification succeeded unexpectedly");
            }
            ChallengeVerificationResult::ChallengeVerificationFailed => {}
        }
    }
}
