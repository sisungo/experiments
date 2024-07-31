mod lexer;

use crate::access::AccessVector;
use anyhow::anyhow;
use lexer::{Kind, Token};
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
        let mut rules = Vec::new();
        let tokens = lexer::process_file(path)?;
        let stmts = tokens.split(|token| token.kind == Kind::Semicolon);
        for stmt in stmts {
            if stmt.is_empty() {
                continue;
            }
            rules.push(Rule::from_tokens(stmt)?);
        }
        Ok(Self { rules })
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

    pub fn from_tokens(tokens: &[Token]) -> anyhow::Result<Self> {
        assert!(!tokens.is_empty());
        if tokens.len() == 1 {
            return Err(anyhow!("incomplete statement at {}", &tokens[0].span));
        }
        match &tokens[0].kind {
            Kind::Ident(x) if x == "allow" => Ok(Self {
                condition: Condition::from_tokens(&tokens[1..])?,
                allowed: true,
            }),
            Kind::Ident(x) if x == "deny" => Ok(Self {
                condition: Condition::from_tokens(&tokens[1..])?,
                allowed: false,
            }),
            _ => Err(anyhow!(
                "unexpected token `{:?}` at {}",
                &tokens[0].kind,
                &tokens[0].span
            )),
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

    pub fn from_tokens(tokens: &[Token]) -> anyhow::Result<Self> {
        assert!(!tokens.is_empty());
        let (action, objsbj) =
            match tokens.split_once(|tok| matches!(&tok.kind, Kind::Ident(x) if x == "on")) {
                Some(tup) => tup,
                None => return Err(anyhow!("incomplete condition at {}", &tokens[0].span)),
            };
        let (obj, sbj) =
            match objsbj.split_once(|tok| matches!(&tok.kind, Kind::Ident(x) if x == "from")) {
                Some(tup) => tup,
                None => return Err(anyhow!("incomplete condition at {}", &tokens[0].span)),
            };
        todo!()
    }
}

pub enum Matching<T> {
    Anything,
    Something(T),
    Multiple(Vec<T>),
}
impl<T: PartialEq> Matching<T> {
    pub fn may_match(&self, val: &T) -> bool {
        match self {
            Self::Anything => true,
            Self::Something(sth) => sth == val,
        }
    }
}
impl Matching<String> {
    pub fn from_token(tok: &Token) -> anyhow::Result<Self> {
        match &tok.kind {
            Kind::Ident(x) if x == "*" => Ok(Self::Anything),
            Kind::Literal(x) => Ok(Self::Something(x.clone())),
            _ => Err(anyhow!(
                "expected string literal or `*`, found `{:?}` at {}",
                &tok.kind,
                &tok.span
            )),
        }
    }
}
impl Matching<libc::uid_t> {
    pub fn from_token(tok: &Token) -> anyhow::Result<Self> {
        match &tok.kind {
            Kind::Ident(x) if x == "*" => Ok(Self::Anything),
            Kind::Ident(x) => Ok(Self::Something(x.parse().map_err(|_| {
                anyhow!(
                    "expected UID integer literal or `*`, found `{:?}` at {}",
                    &tok.kind,
                    &tok.span
                )
            })?)),
            _ => Err(anyhow!(
                "expected integer literal or `*`, found `{:?}` at {}",
                &tok.kind,
                &tok.span
            )),
        }
    }
}
