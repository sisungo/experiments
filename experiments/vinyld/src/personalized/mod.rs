use crate::{AppState, user::Uid};
use serde::{Deserialize, Serialize};

/// The personalized manager.
#[derive(Debug)]
pub struct Personalized<'a>(&'a AppState);
impl Personalized<'_> {
    pub async fn homepage(&self, uid: Option<Uid>) -> Homepage {
        todo!()
    }
}

/// The personalized homepage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Homepage {
    pub a: usize,
}

impl AppState {
    pub fn personalized(&self) -> Personalized {
        Personalized(self)
    }
}
