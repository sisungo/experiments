use crate::{
    access::{AccessVector, Decision},
    database::AccessDb,
    helper::HelperHub,
};
use anyhow::anyhow;

pub struct AccessConductor {
    access_db: AccessDb,
    helper_hub: HelperHub,
}
impl AccessConductor {
    pub fn new(access_db: AccessDb, helper_hub: HelperHub) -> Self {
        Self {
            access_db,
            helper_hub,
        }
    }

    pub async fn decide(&self, access_vector: &AccessVector) -> anyhow::Result<Decision> {
        self.access_db
            .decide(access_vector)?
            .ok_or_else(|| anyhow!("not found"))
    }
}
