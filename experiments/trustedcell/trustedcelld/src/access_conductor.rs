use crate::{
    access::{AccessVector, Decision},
    database::AccessDb,
    helper::HelperHub,
    rule::Ruleset,
};
use anyhow::anyhow;

pub struct AccessConductor {
    access_db: AccessDb,
    ruleset: Ruleset,
    helper_hub: HelperHub,
}
impl AccessConductor {
    pub fn new(access_db: AccessDb, ruleset: Ruleset, helper_hub: HelperHub) -> Self {
        Self {
            access_db,
            ruleset,
            helper_hub,
        }
    }

    pub fn remember(&self, access_vector: &AccessVector, decision: Decision) -> anyhow::Result<()> {
        self.access_db.remember(access_vector, decision)?;
        Ok(())
    }

    pub async fn decide(&self, access_vector: &AccessVector) -> anyhow::Result<Decision> {
        if let Some(decision) = self.access_db.decide(access_vector)? {
            return Ok(decision);
        }
        if let Some(allowed) = self.ruleset.decide(access_vector) {
            // Rulesets are static, so the results are always cachable.
            return Ok(match allowed {
                true => Decision::Allow,
                false => Decision::Deny,
            });
        }
        if let Ok(decision) = self.helper_hub.decide(access_vector).await {
            return Ok(decision);
        }
        Err(anyhow!("not found"))
    }
}
