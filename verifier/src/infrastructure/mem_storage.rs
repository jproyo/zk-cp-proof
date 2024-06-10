use crate::domain::verifier::{ChallengeStore, Register, User, VerifierStorage};
use dashmap::DashMap;
use zk_cp_protocol::protocol::cp::AuthId;

/// In-memory storage implementation for the verifier module.
pub(crate) struct MemStorage {
    pub(crate) users: DashMap<User, Register>,
    pub(crate) challenges: DashMap<AuthId, ChallengeStore>,
}

impl MemStorage {
    /// Creates a new instance of `MemStorage`.
    pub(crate) fn new() -> Self {
        Self {
            users: DashMap::new(),
            challenges: DashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl VerifierStorage for MemStorage {
    /// Stores a user registration in the memory storage.
    ///
    /// # Arguments
    ///
    /// * `register` - The user registration to store.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the operation is successful, or an `anyhow::Error` if an error occurs.
    async fn store_user(&self, register: Register) -> anyhow::Result<()> {
        self.users.insert(register.user.clone(), register);
        Ok(())
    }

    /// Stores a challenge in the memory storage.
    ///
    /// # Arguments
    ///
    /// * `auth_id` - The authentication ID associated with the challenge.
    /// * `challenge` - The challenge to store.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the operation is successful, or an `anyhow::Error` if an error occurs.
    async fn store_challenge(
        &self,
        auth_id: &AuthId,
        challenge: ChallengeStore,
    ) -> anyhow::Result<()> {
        self.challenges.insert(auth_id.clone(), challenge);
        Ok(())
    }

    /// Retrieves a user registration from the memory storage.
    ///
    /// # Arguments
    ///
    /// * `user` - The user to retrieve the registration for.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(register))` if the user registration is found, `Ok(None)` if the user is not found,
    /// or an `anyhow::Error` if an error occurs.
    async fn get_user(&self, user: &User) -> anyhow::Result<Option<Register>> {
        Ok(self.users.get(user).map(|r| r.value().clone()))
    }

    /// Retrieves a challenge from the memory storage.
    ///
    /// # Arguments
    ///
    /// * `auth_id` - The authentication ID associated with the challenge.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(challenge))` if the challenge is found, `Ok(None)` if the challenge is not found,
    /// or an `anyhow::Error` if an error occurs.
    async fn get_challenge(&self, auth_id: &AuthId) -> anyhow::Result<Option<ChallengeStore>> {
        Ok(self.challenges.get(auth_id).map(|c| c.value().clone()))
    }
}
