//! This module contains the implementation of a Chaum-Pedersen ZK protocol for registration, commitment, challenge, and verification.
//!
//! The protocol is defined by a series of steps, each represented by a corresponding struct. The steps include:
//! - `Register`: Represents the registration step, where a user registers with a material and generates `y1` and `y2` values.
//! - `Commitment`: Represents the commitment step, where the user generates commitment values `r1` and `r2` based on the material and a random value `k`.
//! - `Challenge`: Represents the challenge step, where a challenge value `c` is generated.
//! - `ChallengeResponse`: Represents the challenge response step, where the user calculates a response `s` based on the challenge, material, and private key `x`.
//! - `VerificationRequest`: Represents the verification request step, where the user sends a verification request with the authentication ID and response `s`.
//! - `Verification`: Represents the verification step, where the server verifies the response `s` based on the received values and material.
//! - `VerificationResult`: Represents the result of the verification step, indicating whether the challenge was successfully verified or not.
//!
//! The protocol steps are implemented as structs with associated methods for generating the next step based on the current state. The protocol steps also implement the `ProtocolStep` trait, which allows them to be used generically in the `ProtocolState` struct.
//!
//! The `ProtocolState` struct represents the current state of the protocol and provides methods for transitioning to the next step. It is parameterized by the current step type and enforces type safety in the protocol transitions.
//!
//! The protocol transitions are defined by the `ProtocolTransition` trait, which provides a `change` method to transition to the next step. Each step implements the `ProtocolTransition` trait for the corresponding next step.
//!
//! The module also includes unit tests for the protocol transitions, ensuring that the protocol progresses correctly from one step to another.
use num_bigint::{BigInt, RandBigInt};
use num_primes::Generator;
use num_traits::{One, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(Debug, Clone, TypedBuilder)]
pub struct Register {
    pub material: Material,
    pub y1: BigInt,
    pub y2: BigInt,
}

impl Register {
    pub fn new(material: Material, x: &BigInt) -> Self {
        let y1 = material.g.modpow(x, &material.p);
        let y2 = material.h.modpow(x, &material.p);
        Register { material, y1, y2 }
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

#[derive(Debug, Clone, TypedBuilder)]
pub struct Material {
    #[builder(setter(into))]
    pub g: BigInt,
    #[builder(setter(into))]
    pub h: BigInt,
    #[builder(setter(into))]
    pub q: BigInt,
    #[builder(setter(into))]
    pub p: BigInt,
}

impl Material {
    pub fn generate(g: Option<BigInt>) -> Self {
        let p_prime = Generator::safe_prime(16);
        let p: BigInt = p_prime.to_u64().unwrap().into();
        let q: BigInt = (p.clone() - BigInt::one()) / 2;
        let g: BigInt = g.unwrap_or(7.into());
        let h: BigInt = g.modpow(&BigInt::from(11), &p);
        Material { g, h, p, q }
    }
}

impl Default for Material {
    fn default() -> Self {
        Material::generate(None)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaterialSerde {
    pub user: String,
    pub g: String,
    pub h: String,
    pub q: String,
    pub p: String,
}

impl MaterialSerde {
    pub fn from_material(material: &Material, user: &str) -> Self {
        MaterialSerde {
            user: user.to_string(),
            g: material.g.to_str_radix(16),
            h: material.h.to_str_radix(16),
            q: material.q.to_str_radix(16),
            p: material.p.to_str_radix(16),
        }
    }

    pub fn to_material(&self) -> Material {
        Material {
            g: BigInt::parse_bytes(self.g.as_bytes(), 16).unwrap(),
            h: BigInt::parse_bytes(self.h.as_bytes(), 16).unwrap(),
            q: BigInt::parse_bytes(self.q.as_bytes(), 16).unwrap(),
            p: BigInt::parse_bytes(self.p.as_bytes(), 16).unwrap(),
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct Challenge {
    #[builder(setter(into))]
    pub auth_id: AuthId,
    #[builder(setter(into))]
    pub c: BigInt,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct ChallengeResponse {
    pub challenge: Challenge,
    pub material: Material,
    #[builder(setter(into))]
    pub x: BigInt,
    #[builder(setter(into))]
    pub k: BigInt,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct VerificationRequest {
    pub auth_id: AuthId,
    pub s: BigInt,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum VerificationResult {
    ChallengeVerifiedSuccess,
    ChallengeVerificationFailed,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct Commitment {
    pub material: Material,
    pub r1: BigInt,
    pub r2: BigInt,
    pub k: BigInt,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct Verification {
    pub material: Material,
    #[builder(setter(into))]
    pub y1: BigInt,
    #[builder(setter(into))]
    pub y2: BigInt,
    #[builder(setter(into))]
    pub r1: BigInt,
    #[builder(setter(into))]
    pub r2: BigInt,
    #[builder(setter(into))]
    pub c: BigInt,
    #[builder(setter(into))]
    pub s: BigInt,
}

/// Trait Type State Pattern
pub trait ProtocolStep {}
impl ProtocolStep for Register {}
impl ProtocolStep for Commitment {}
impl ProtocolStep for Challenge {}
impl ProtocolStep for ChallengeResponse {}
impl ProtocolStep for Material {}
impl ProtocolStep for VerificationRequest {}
impl ProtocolStep for Verification {}
impl ProtocolStep for VerificationResult {}

#[derive(Debug, Clone)]
pub struct ProtocolState<S: ProtocolStep> {
    state: S,
}

impl<S> ProtocolState<S>
where
    S: ProtocolStep,
{
    pub fn into_inner(self) -> S {
        self.state
    }
}

impl<S> From<S> for ProtocolState<S>
where
    S: ProtocolStep,
{
    fn from(state: S) -> Self {
        ProtocolState { state }
    }
}

pub trait ProtocolTransition {
    type NewState: ProtocolStep;
    fn change(self) -> ProtocolState<Self::NewState>;
}

impl ProtocolTransition for ProtocolState<Register> {
    type NewState = Commitment;
    fn change(self) -> ProtocolState<Self::NewState> {
        let k = &rand::thread_rng().gen_bigint_range(&2.into(), &(&self.state.material.q - 2));
        let p = &self.state.material.p;
        let r1 = self.state.material.g.modpow(k, p);
        let r2 = self.state.material.h.modpow(k, p);
        ProtocolState {
            state: Commitment {
                material: self.state.material.clone(),
                r1,
                r2,
                k: k.clone(),
            },
        }
    }
}

impl ProtocolTransition for ProtocolState<Material> {
    type NewState = Challenge;
    fn change(self) -> ProtocolState<Self::NewState> {
        let q = &self.state.q;
        let c = rand::thread_rng().gen_bigint_range(&2.into(), &(q - 1));
        ProtocolState {
            state: Challenge {
                auth_id: AuthId(Uuid::new_v4().to_string()),
                c,
            },
        }
    }
}

impl ProtocolTransition for ProtocolState<ChallengeResponse> {
    type NewState = VerificationRequest;
    fn change(self) -> ProtocolState<Self::NewState> {
        let c: BigInt = self.state.challenge.c;
        let q = &self.state.material.q;
        let k = self.state.k;
        let cx = c * &self.state.x;
        let one = &BigInt::one();
        let s = if k > cx {
            (k - cx).modpow(one, q)
        } else {
            q - (cx - k).modpow(one, q)
        };

        ProtocolState {
            state: VerificationRequest {
                auth_id: self.state.challenge.auth_id,
                s,
            },
        }
    }
}

impl ProtocolState<Verification> {
    pub fn change(self) -> ProtocolState<VerificationResult> {
        let c = &self.state.c;
        let y1 = &self.state.y1;
        let y2 = &self.state.y2;
        let r1 = &self.state.r1;
        let r2 = &self.state.r2;
        let g = &self.state.material.g;
        let h = &self.state.material.h;
        let p = &self.state.material.p;
        let s = &self.state.s;
        let one = &BigInt::one();
        let r1_prime = (g.modpow(s, p) * y1.modpow(c, p)).modpow(one, p);
        let r2_prime = (h.modpow(s, p) * y2.modpow(c, p)).modpow(one, p);
        if r1 == &r1_prime && r2 == &r2_prime {
            tracing::info!("Challenge verified successfully");
            ProtocolState {
                state: VerificationResult::ChallengeVerifiedSuccess,
            }
        } else {
            tracing::info!(
                "Challenge verification failed due to mismatch - expected: {:?}, actual: {:?}",
                (r1_prime, r2_prime),
                (r1, r2)
            );
            ProtocolState {
                state: VerificationResult::ChallengeVerificationFailed,
            }
        }
    }
}

#[cfg(test)]
/// Module containing tests for the `cp` module.
mod tests {
    use super::*;

    /// Test for the challenge transition change.
    #[test]
    fn test_challenge_transition_change() {
        // Initialize variables
        let x = BigInt::from(11);
        let material = Material::default();
        let register = Register::new(material.clone(), &x);

        // Create commit protocol
        let commit_proto = <Register as Into<ProtocolState<_>>>::into(register.clone()).change();
        let commitment = commit_proto.clone().into_inner();

        // Create challenge protocol
        let challenge_proto = <Material as Into<ProtocolState<_>>>::into(material).change();
        let challenge = challenge_proto.into_inner();

        // Create challenge response
        let challenge_response = ChallengeResponse::builder()
            .challenge(challenge.clone())
            .material(commitment.material)
            .x(x)
            .k(commitment.k)
            .build();

        // Create verification protocol
        let verification_proto =
            <ChallengeResponse as Into<ProtocolState<_>>>::into(challenge_response).change();
        let verification = verification_proto.into_inner();

        // Create verification
        let verification = Verification::builder()
            .material(register.material)
            .y1(register.y1)
            .y2(register.y2)
            .r1(commitment.r1)
            .r2(commitment.r2)
            .c(challenge.c)
            .s(verification.s)
            .build();

        // Convert verification to protocol state
        let verification_proto = ProtocolState::from(verification);
        let result = verification_proto.change().into_inner();

        // Assert the result
        assert_eq!(result, VerificationResult::ChallengeVerifiedSuccess);
    }
}
