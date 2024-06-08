use anyhow::anyhow;
use num_primes::BigUint;
use num_traits::ToPrimitive;
use std::ops::Deref;
use typed_builder::TypedBuilder;

use crate::grpc::zkp_material;

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

#[derive(Debug, Clone)]
pub struct PrimeOrder(pub u64);

impl From<u64> for PrimeOrder {
    fn from(s: u64) -> Self {
        PrimeOrder(s)
    }
}

impl From<PrimeOrder> for BigUint {
    fn from(s: PrimeOrder) -> Self {
        BigUint::from(s.0)
    }
}

impl Deref for PrimeOrder {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, TypedBuilder, Eq, PartialEq)]
pub struct Material {
    pub g: BigUint,
    pub h: BigUint,
}

impl TryFrom<Material> for zkp_material::MaterialResponse {
    type Error = anyhow::Error;

    fn try_from(m: Material) -> anyhow::Result<Self> {
        Ok(zkp_material::MaterialResponse {
            g: m.g.to_i64().ok_or(anyhow!("cannot convert 'g' to i64"))?,
            h: m.h.to_i64().ok_or(anyhow!("cannot conver 'h' to i64"))?,
        })
    }
}

// Material generator is a trait that generates a material with a prime order q.
//
// If the prime order q is not provided, the generator should generate a prime order q
// and then generate the material with the generated prime order q.
#[async_trait::async_trait]
pub trait MaterialGenerator {
    // Generate a material with a prime order q.
    //
    // If the prime order q is not provided, the generator should generate a prime order q
    //
    // # Arguments
    // * `q` - The prime order q
    //
    // # Returns
    // A material with a prime order q
    //
    // # Errors
    // Returns an error if the material cannot be generated
    async fn generate(&self, q: Option<PrimeOrder>) -> anyhow::Result<Material>;
}

// Material storage is a trait that stores and retrieves a material for a user.
#[async_trait::async_trait]
pub trait MaterialStorage {
    // Store a material for a user.
    //
    // # Arguments
    //
    // * `user` - The User
    // * `material` - The MaterialStorage
    //
    // # Returns
    // () if the material is stored successfully
    //
    // # Errors
    //
    // Returns an error if the material cannot be stored
    async fn store(&self, user: User, material: Material) -> anyhow::Result<()>;

    // Get a material for a user.
    //
    // # Arguments
    // * `user` - The user
    //
    // # Returns
    // Returns None if the material does not exist
    // Returns Some(Material) if the material exists
    //
    // # Errors
    // Returns an error if the material cannot be retrieved
    async fn get(&self, user: &User) -> anyhow::Result<Option<Material>>;
}
