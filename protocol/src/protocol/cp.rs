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

#[derive(Debug, Clone, TypedBuilder, Serialize, Deserialize)]
pub struct Material {
    pub g: BigInt,
    pub h: BigInt,
    pub q: BigInt,
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

#[derive(Debug, Clone, TypedBuilder)]
pub struct Challenge {
    pub auth_id: AuthId,
    pub c: BigInt,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct ChallengeResponse {
    pub challenge: Challenge,
    pub material: Material,
    pub x: BigInt,
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
    pub y1: BigInt,
    pub y2: BigInt,
    pub r1: BigInt,
    pub r2: BigInt,
    pub c: BigInt,
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
mod tests {
    use super::*;

    #[test]
    fn test_challenge_transition_change() {
        let x = BigInt::from(11);
        let material = Material::default();
        let register = Register::new(material.clone(), &x);
        let commit_proto = <Register as Into<ProtocolState<_>>>::into(register.clone()).change();
        let commitment = commit_proto.clone().into_inner();
        let challenge_proto = <Material as Into<ProtocolState<_>>>::into(material).change();
        let challenge = challenge_proto.into_inner();

        let challenge_response = ChallengeResponse::builder()
            .challenge(challenge.clone())
            .material(commitment.material)
            .x(x)
            .k(commitment.k)
            .build();

        let verification_proto =
            <ChallengeResponse as Into<ProtocolState<_>>>::into(challenge_response).change();
        let verification = verification_proto.into_inner();

        let verification = Verification::builder()
            .material(register.material)
            .y1(register.y1)
            .y2(register.y2)
            .r1(commitment.r1)
            .r2(commitment.r2)
            .c(challenge.c)
            .s(verification.s)
            .build();

        let verification_proto = ProtocolState::from(verification);
        let result = verification_proto.change().into_inner();

        assert_eq!(result, VerificationResult::ChallengeVerifiedSuccess);
    }
}
