mod lexer;
mod parser;

use crate::access::AccessVector;
use std::path::Path;

pub struct Ruleset {
    rules: Vec<Rule>,
}
impl Ruleset {
    pub fn decide(&self, access_vector: &AccessVector) -> Option<bool> {
        for rule in &self.rules {
            if let Some(decision) = rule.decide(access_vector) {
                return Some(decision);
            }
        }
        None
    }

    pub fn compile_file(path: &Path) -> anyhow::Result<Self> {
        todo!()
    }
}

pub struct Rule {
    condition: Condition,
    allowed: bool,
}
impl Rule {
    pub fn decide(&self, access_vector: &AccessVector) -> Option<bool> {
        if self.condition.covers(access_vector) {
            Some(self.allowed)
        } else {
            None
        }
    }
}

pub struct Condition {
    subject_uid: Matching<libc::uid_t>,
    subject_cell: Matching<String>,
    object_category: Matching<String>,
    object_owner: Matching<String>,
    action: Matching<String>,
}
impl Condition {
    pub fn covers(&self, access_vector: &AccessVector) -> bool {
        self.subject_uid.may_match(&access_vector.subject.uid)
            && self.subject_cell.may_match(&access_vector.subject.cell)
            && self
                .object_category
                .may_match(&access_vector.object.category)
            && self.object_owner.may_match(&access_vector.object.owner)
            && self.action.may_match(&access_vector.action)
    }
}

pub enum Matching<T> {
    Anything,
    Something(T),
}
impl<T: PartialEq> Matching<T> {
    pub fn may_match(&self, val: &T) -> bool {
        match self {
            Self::Anything => true,
            Self::Something(sth) => sth == val,
        }
    }
}
