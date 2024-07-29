use crate::{database::AccessDb, helper::HelperHub};

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
}