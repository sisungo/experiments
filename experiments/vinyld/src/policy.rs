//! Access policy framework.

use crate::user::{Uid, User};
use serde::{Deserialize, Serialize};

/// Policy of accessing an object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectPolicy {
    pub class: String,
    pub items: Vec<PolicyItem>,
}
impl ObjectPolicy {
    /// Returns `true` if the given subject is allowed using this policy.
    pub fn allows(&self, subject: &Subject) -> bool {
        let mut allowed = false;

        for item in &self.items {
            match item {
                PolicyItem::Allow(condition) => {
                    if condition.matches(subject) {
                        allowed = true;
                    }
                }
                PolicyItem::Deny(condition) => {
                    if condition.matches(subject) {
                        allowed = false;
                    }
                }
            }
        }

        allowed
    }

    /// Returns `true` if the given subject is denied using this policy.
    pub fn denies(&self, subject: &Subject) -> bool {
        !self.allows(subject)
    }

    /// Returns an object policy that any access is allowed.
    pub fn allowed() -> Self {
        Self {
            class: "allowed".into(),
            items: vec![PolicyItem::Allow(Condition::MatchAny)],
        }
    }

    /// Returns an object policy that any access is denied.
    pub fn denied() -> Self {
        Self {
            class: "denied".into(),
            items: vec![PolicyItem::Deny(Condition::MatchAny)],
        }
    }

    /// Returns JSON form of the policy.
    pub fn json(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("any object policy should be able to be parsed into JSON")
    }
}
impl Default for ObjectPolicy {
    fn default() -> Self {
        Self::allowed()
    }
}

/// An item in an object policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyItem {
    Allow(Condition),
    Deny(Condition),
}

/// Matching condition in an object policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    MatchAny,
    All(Vec<Condition>),
    Any(Vec<Condition>),
    Not(Box<Condition>),
    MatchUid(Uid),
    MatchGroup(String),
    MatchAnon,
}
impl Condition {
    /// Returns `true` if the given subject is covered by this policy.
    pub fn matches(&self, subject: &Subject) -> bool {
        match self {
            Condition::MatchAny => true,
            Condition::All(conditions) => conditions.iter().all(|c| c.matches(subject)),
            Condition::Any(conditions) => conditions.iter().any(|c| c.matches(subject)),
            Condition::Not(condition) => !condition.matches(subject),
            Condition::MatchUid(uid) => !subject.anon && subject.uid == *uid,
            Condition::MatchGroup(group) => subject.groups.contains(group),
            Condition::MatchAnon => subject.anon,
        }
    }
}

/// A subject.
#[derive(Debug, Clone)]
pub struct Subject {
    anon: bool,
    uid: Uid,
    groups: Vec<String>,
}
impl Subject {
    /// Constructs a subject from a user.
    pub fn from_user(user: &User) -> Self {
        Self {
            anon: false,
            uid: user.uid,
            groups: user.groups.clone(),
        }
    }

    /// Returns an anonymous subject.
    pub fn anon() -> Self {
        Self {
            anon: true,
            uid: Uid(0),
            groups: vec![],
        }
    }
}
