mod lexer;

use crate::access::AccessVector;
use anyhow::anyhow;
use lexer::{Kind, Token};
use std::path::Path;

pub struct Ruleset {
    rules: Vec<Rule>,
    private_categories: Vec<String>,
}
impl Ruleset {
    pub fn decide(&self, access_vector: &AccessVector) -> Option<bool> {
        for rule in &self.rules {
            if let Some(decision) = rule.decide(access_vector) {
                return Some(decision);
            }
        }
        if self
            .private_categories
            .contains(&access_vector.object.category)
        {
            if access_vector.subject.cell == access_vector.object.owner {
                return Some(true);
            }
        }
        None
    }

    pub fn compile_file(path: &Path) -> anyhow::Result<Self> {
        let mut rules = Vec::new();
        let mut private_categories = Vec::new();
        let tokens = lexer::process_file(path)?;
        let stmts = tokens.split(|token| token.kind == Kind::Semicolon);
        for stmt in stmts {
            if stmt.is_empty() {
                continue;
            }
            if matches!(&stmt[0].kind, Kind::Ident(x) if x == "auto_private") {
                match stmt.get(1) {
                    Some(x) => match &x.kind {
                        Kind::Literal(y) => private_categories.push(y.to_owned()),
                        _ => {
                            return Err(anyhow!(
                                "expected string literal, found `{:?}` at {}",
                                &x.kind,
                                &x.span
                            ))
                        }
                    },
                    None => return Err(anyhow!("incomplete auto_private at {}", &stmt[0].span)),
                }
            }
            rules.push(Rule::from_tokens(stmt)?);
        }
        Ok(Self {
            private_categories,
            rules,
        })
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
    subject_cell: Matching<String>,
    object_category: Matching<String>,
    object_owner: Matching<String>,
    action: Matching<String>,
}
impl Condition {
    pub fn covers(&self, access_vector: &AccessVector) -> bool {
        self.subject_cell.may_match(&access_vector.subject.cell)
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
        let (category, owner) =
            match obj.split_once(|tok| matches!(&tok.kind, Kind::Ident(x) if x == "of")) {
                Some(tup) => tup,
                None => return Err(anyhow!("incomplete condition at {}", &tokens[0].span)),
            };
        if action.is_empty() || sbj.is_empty() || category.is_empty() || owner.is_empty() {
            return Err(anyhow!("incomplete condition at {}", &tokens[0].span));
        }
        Ok(Self {
            subject_cell: Matching::from_token(&sbj[0])?,
            object_category: Matching::from_token(&category[0])?,
            object_owner: Matching::from_token(&owner[0])?,
            action: Matching::from_token(&action[0])?,
        })
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
