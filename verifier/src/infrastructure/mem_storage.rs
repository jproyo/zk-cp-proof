use dashmap::DashMap;

use crate::domain::verifier::{AuthId, ChallengeStore, Register, User, VerifierStorage};

pub(crate) struct MemStorage {
    pub(crate) users: DashMap<User, Register>,
    pub(crate) challenges: DashMap<AuthId, ChallengeStore>,
}

impl MemStorage {
    pub(crate) fn new() -> Self {
        Self {
            users: DashMap::new(),
            challenges: DashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl VerifierStorage for MemStorage {
    async fn store_user(&self, register: Register) -> anyhow::Result<()> {
        self.users.insert(register.user.clone(), register);
        Ok(())
    }

    async fn store_challenge(
        &self,
        auth_id: &AuthId,
        challenge: ChallengeStore,
    ) -> anyhow::Result<()> {
        self.challenges.insert(auth_id.clone(), challenge);
        Ok(())
    }

    async fn get_user(&self, user: &User) -> anyhow::Result<Option<Register>> {
        Ok(self.users.get(user).map(|r| r.value().clone()))
    }

    async fn get_challenge(&self, auth_id: &AuthId) -> anyhow::Result<Option<ChallengeStore>> {
        Ok(self.challenges.get(auth_id).map(|c| c.value().clone()))
    }
}
