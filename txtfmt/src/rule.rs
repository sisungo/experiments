use once_cell::sync::Lazy;
use std::{collections::HashSet, fmt::Write, str::FromStr, sync::RwLock};

macro_rules! procedure_trim_matcher {
    ($v:expr, $c:expr) => {
        match $v {
            "" => Self::Trim($c),
            "begin" => Self::TrimBegin($c),
            "end" => Self::TrimEnd($c),
            _ => unreachable!(),
        }
    };
}

macro_rules! procedure_ifuntil_matcher {
    ($v:expr, $a:expr, $b:expr) => {
        match $v {
            "if" => Self::If($a, $b),
            "until" => Self::RepeatUntil($a, $b),
            _ => unreachable!(),
        }
    };
}

macro_rules! procedure_oneliteral_matcher {
    ($v:expr, $p:expr) => {
        match $v {
            "append" => Self::Append($p),
            "appendline" => Self::AppendLine($p),
            "opb" => Self::OnParaBegin($p),
            "mpbw" => Self::MakeParaBeginWith($p),
            "addflag" => Self::AddFlag($p),
            "delflag" => Self::DelFlag($p),
            _ => unreachable!(),
        }
    };
}

macro_rules! condition_allany_matcher {
    ($v:expr, $a:expr, $b:expr) => {
        match $v {
            "all" => Self::All(Box::new($a), Box::new($b)),
            "any" => Self::Any(Box::new($a), Box::new($b)),
            _ => unreachable!(),
        }
    };
}

macro_rules! condition_oneliteral_matcher {
    ($v:expr, $p:expr) => {
        match $v {
            "contains" => Self::Contains($p),
            "content_eq" => Self::ContentEq($p),
            "flag" => Self::Flag($p),
            _ => unreachable!(),
        }
    };
}

static FLAGS: Lazy<RwLock<HashSet<String>>> = Lazy::new(RwLock::default);

#[derive(Debug, Clone)]
pub enum Procedure {
    Replace(String, String),
    UnsplitLines,
    OnParaBegin(String),
    MakeParaBeginWith(String),
    Trim(Option<String>),
    TrimBegin(Option<String>),
    TrimEnd(Option<String>),
    Append(String),
    AppendLine(String),
    RepeatN(u32, Box<Procedure>),
    RepeatUntil(Condition, Box<Procedure>),
    If(Condition, Box<Procedure>),
    AddFlag(String),
    DelFlag(String),
    EachLine(Box<Procedure>),
}
impl Procedure {
    pub fn run(&self, s: String) -> String {
        match self {
            Self::Replace(from, to) => crate::tools::replace(s, (from, to)),
            Self::UnsplitLines => crate::tools::unsplit_lines(s),
            Self::OnParaBegin(content) => crate::tools::on_para_begin(s, content),
            Self::MakeParaBeginWith(content) => crate::tools::make_para_begin_with(s, content),
            Self::Trim(mat) => crate::tools::trim(s, mat.as_deref()),
            Self::TrimBegin(mat) => crate::tools::trim_begin(s, mat.as_deref()),
            Self::TrimEnd(mat) => crate::tools::trim_end(s, mat.as_deref()),
            Self::RepeatN(n, p) => Self::repeatn(*n, p, s),
            Self::Append(p) => crate::tools::append(s, p),
            Self::AppendLine(p) => crate::tools::append_line(s, p),
            Self::RepeatUntil(cond, p) => Self::repeat_until(cond, p, s),
            Self::If(cond, p) => Self::if_do(cond, p, s),
            Self::AddFlag(flag) => {
                addflag(flag.clone());
                s
            }
            Self::DelFlag(flag) => {
                delflag(flag);
                s
            }
            Self::EachLine(p) => Self::each_line(p, s),
        }
    }

    fn repeatn(n: u32, p: &Self, mut s: String) -> String {
        for _ in 0..n {
            s = p.run(s);
        }
        s
    }

    fn if_do(cond: &Condition, p: &Self, s: String) -> String {
        if cond.run(&s) {
            p.run(s)
        } else {
            s
        }
    }

    fn each_line(p: &Self, s: String) -> String {
        s.lines()
            .fold(String::with_capacity(s.len()), |mut acc, line| {
                writeln!(&mut acc, "{}", p.run(line.to_owned())).unwrap();
                acc
            })
    }

    fn repeat_until(cond: &Condition, p: &Self, mut s: String) -> String {
        while !cond.run(&s) {
            s = p.run(s);
        }
        s
    }

    fn parse_tokens(tokens: &[Token]) -> Result<Self, Error> {
        if let Some(Token::Ident(ident)) = tokens.first() {
            match &ident[..] {
                "replace" => Self::try_parse_replace(&tokens[1..]),
                "unsplit_lines" => Self::try_parse_unsplit_lines(&tokens[1..]),
                "on_para_begin" => Self::try_parse_oneliteral("opb", &tokens[1..]),
                "make_para_begin_with" => Self::try_parse_oneliteral("mpbw", &tokens[1..]),
                "trim" => Self::try_parse_trim("", &tokens[1..]),
                "trim_begin" => Self::try_parse_trim("begin", &tokens[1..]),
                "trim_end" => Self::try_parse_trim("end", &tokens[1..]),
                "repeatn" => Self::try_parse_repeatn(&tokens[1..]),
                "repeat_until" => Self::try_parse_ifuntil("until", &tokens[1..]),
                "append" => Self::try_parse_oneliteral("append", &tokens[1..]),
                "append_line" => Self::try_parse_oneliteral("appendline", &tokens[1..]),
                "if" => Self::try_parse_ifuntil("if", &tokens[1..]),
                "addflag" => Self::try_parse_oneliteral("addflag", &tokens[1..]),
                "delflag" => Self::try_parse_oneliteral("delflag", &tokens[1..]),
                "each_line" => Self::try_parse_each_line(&tokens[1..]),
                _ => Err(Error::FnNotFound(ident.clone())),
            }
        } else {
            Err(Error::Expected("ident", tokens.first().cloned()))
        }
    }

    fn try_parse_replace(tokens: &[Token]) -> Result<Self, Error> {
        let left_paren = matches!(tokens.first(), Some(Token::LeftParen));
        let right_paren = matches!(tokens.get(4), Some(Token::RightParen));
        let comma = matches!(tokens.get(2), Some(Token::Comma));
        let valid_len = tokens.len() == 5;
        if !(left_paren && right_paren && comma && valid_len) {
            return Err(Error::BadFunctionCall);
        }
        let Some(Token::Literal(from)) = tokens.get(1) else {
            return Err(Error::Expected("literal", tokens.get(1).cloned()));
        };
        let Some(Token::Literal(to)) = tokens.get(3) else {
            return Err(Error::Expected("literal", tokens.get(3).cloned()));
        };

        Ok(Self::Replace(from.clone(), to.clone()))
    }

    fn try_parse_unsplit_lines(tokens: &[Token]) -> Result<Self, Error> {
        let left_paren = matches!(tokens.first(), Some(Token::LeftParen));
        let right_paren = matches!(tokens.get(1), Some(Token::RightParen));
        let valid_len = tokens.len() == 2;
        if !(left_paren && right_paren && valid_len) {
            return Err(Error::BadFunctionCall);
        }

        Ok(Self::UnsplitLines)
    }

    fn try_parse_oneliteral(var: &'static str, tokens: &[Token]) -> Result<Self, Error> {
        let left_paren = matches!(tokens.first(), Some(Token::LeftParen));
        let right_paren = matches!(tokens.get(2), Some(Token::RightParen));
        let valid_len = tokens.len() == 3;
        if !(left_paren && right_paren && valid_len) {
            return Err(Error::BadFunctionCall);
        }
        let Some(Token::Literal(content)) = tokens.get(1) else {
            return Err(Error::Expected("literal", tokens.get(1).cloned()));
        };

        Ok(procedure_oneliteral_matcher!(var, content.clone()))
    }

    fn try_parse_trim(var: &'static str, tokens: &[Token]) -> Result<Self, Error> {
        let left_paren = matches!(tokens.first(), Some(Token::LeftParen));
        match tokens.len() {
            2 => {
                let right_paren = matches!(tokens.get(1), Some(Token::RightParen));
                if !(left_paren && right_paren) {
                    return Err(Error::BadFunctionCall);
                }
                Ok(procedure_trim_matcher!(var, None))
            }
            3 => {
                let right_paren = matches!(tokens.get(2), Some(Token::RightParen));
                if !(left_paren && right_paren) {
                    return Err(Error::BadFunctionCall);
                }
                let Some(Token::Literal(mat)) = tokens.get(1) else {
                    return Err(Error::Expected("literal", tokens.get(1).cloned()));
                };
                Ok(procedure_trim_matcher!(var, Some(mat.clone())))
            }
            _ => Err(Error::BadFunctionCall),
        }
    }

    fn try_parse_repeatn(tokens: &[Token]) -> Result<Self, Error> {
        let left_paren = matches!(tokens.first(), Some(Token::LeftParen));
        let right_paren = matches!(tokens.last(), Some(Token::RightParen));
        let comma = matches!(tokens.get(2), Some(Token::Comma));
        if !(left_paren && right_paren && comma) {
            return Err(Error::BadFunctionCall);
        }
        let Some(Token::Ident(times)) = tokens.get(1) else {
            return Err(Error::Expected("ident", tokens.get(1).cloned()));
        };
        let Ok(times) = times.parse::<u32>() else {
            return Err(Error::Expected("number", Some(Token::Ident(times.clone()))));
        };
        let p = Self::parse_tokens(&tokens[3..tokens.len() - 1])?;
        Ok(Self::RepeatN(times, Box::new(p)))
    }

    fn try_parse_ifuntil(var: &'static str, tokens: &[Token]) -> Result<Self, Error> {
        let left_paren = matches!(tokens.first(), Some(Token::LeftParen));
        let right_paren = matches!(tokens.last(), Some(Token::RightParen));
        if !(left_paren && right_paren) {
            return Err(Error::BadFunctionCall);
        }
        let (cond, proc) = parse_arg2(&tokens[1..tokens.len() - 1])?;
        let proc = Self::parse_tokens(proc)?;
        let cond = Condition::parse_tokens(cond)?;
        Ok(procedure_ifuntil_matcher!(var, cond, Box::new(proc)))
    }

    fn try_parse_each_line(tokens: &[Token]) -> Result<Self, Error> {
        let left_paren = matches!(tokens.first(), Some(Token::LeftParen));
        let right_paren = matches!(tokens.last(), Some(Token::RightParen));
        if !(left_paren && right_paren) {
            return Err(Error::BadFunctionCall);
        }
        let proc = Self::parse_tokens(&tokens[1..tokens.len() - 1])?;
        Ok(Self::EachLine(Box::new(proc)))
    }
}
impl FromStr for Procedure {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = tokenize(s)?;
        Self::parse_tokens(&tokens)
    }
}

pub fn parse(s: &str) -> Result<Vec<Procedure>, Error> {
    let mut procedures = Vec::with_capacity(s.len() / 8);

    for l in s.lines() {
        if l.starts_with("//") || l.trim().is_empty() {
            continue;
        }
        procedures.push(l.parse()?);
    }

    Ok(procedures)
}

#[derive(Debug, Clone)]
pub enum Condition {
    Contains(String),
    ContentEq(String),
    Not(Box<Condition>),
    All(Box<Condition>, Box<Condition>),
    Any(Box<Condition>, Box<Condition>),
    True,
    False,
    Flag(String),
}
impl Condition {
    pub fn run(&self, s: &str) -> bool {
        match self {
            Self::Contains(pat) => s.contains(pat),
            Self::ContentEq(pat) => s == pat,
            Self::Not(cond) => !cond.run(s),
            Self::All(a, b) => a.run(s) && b.run(s),
            Self::Any(a, b) => a.run(s) || b.run(s),
            Self::True => true,
            Self::False => false,
            Self::Flag(f) => flag(f),
        }
    }

    fn parse_tokens(tokens: &[Token]) -> Result<Self, Error> {
        if let Some(Token::Ident(ident)) = tokens.first() {
            match &ident[..] {
                "not" => Self::try_parse_not(&tokens[1..]),
                "contains" => Self::try_parse_oneliteral("contains", &tokens[1..]),
                "content_eq" => Self::try_parse_oneliteral("content_eq", &tokens[1..]),
                "all" => Self::try_parse_allany("all", &tokens[1..]),
                "any" => Self::try_parse_allany("any", &tokens[1..]),
                "true" => Ok(Self::True),
                "false" => Ok(Self::False),
                "flag" => Self::try_parse_oneliteral("flag", &tokens[1..]),
                _ => Err(Error::FnNotFound(ident.clone())),
            }
        } else {
            Err(Error::Expected("ident", tokens.first().cloned()))
        }
    }

    fn try_parse_not(tokens: &[Token]) -> Result<Self, Error> {
        let left_paren = matches!(tokens.first(), Some(Token::LeftParen));
        let right_paren = matches!(tokens.last(), Some(Token::RightParen));
        if !(left_paren && right_paren) {
            return Err(Error::BadFunctionCall);
        }
        let cond = Self::parse_tokens(&tokens[1..tokens.len() - 1])?;
        Ok(Self::Not(Box::new(cond)))
    }

    fn try_parse_oneliteral(var: &'static str, tokens: &[Token]) -> Result<Self, Error> {
        let left_paren = matches!(tokens.first(), Some(Token::LeftParen));
        let right_paren = matches!(tokens.get(2), Some(Token::RightParen));
        if !(left_paren && right_paren) {
            return Err(Error::BadFunctionCall);
        }
        let Some(Token::Literal(pat)) = tokens.get(1) else {
            return Err(Error::Expected("literal", tokens.get(1).cloned()));
        };
        Ok(condition_oneliteral_matcher!(var, pat.clone()))
    }

    fn try_parse_allany(var: &'static str, tokens: &[Token]) -> Result<Self, Error> {
        let left_paren = matches!(tokens.first(), Some(Token::LeftParen));
        let right_paren = matches!(tokens.last(), Some(Token::RightParen));
        if !(left_paren && right_paren) {
            return Err(Error::BadFunctionCall);
        }
        let (a, b) = parse_arg2(&tokens[1..tokens.len() - 1])?;
        let a = Self::parse_tokens(a)?;
        let b = Self::parse_tokens(b)?;
        Ok(condition_allany_matcher!(var, a, b))
    }
}

#[derive(Debug, Clone)]
pub enum Token {
    Ident(String),
    Comma,
    Literal(String),
    LeftParen,
    RightParen,
}

fn tokenize(s: &str) -> Result<Vec<Token>, Error> {
    let mut tokens = Vec::with_capacity(s.len() / 4);
    let mut in_literal = false;
    let mut in_escape = false;
    let mut this = Vec::with_capacity(8);

    for b in s.bytes() {
        if in_literal {
            if in_escape {
                match b {
                    b'n' => this.push(b'\n'),
                    b'r' => this.push(b'\r'),
                    b'"' => this.push(b'"'),
                    b'\\' => this.push(b'\\'),
                    _ => return Err(Error::UnknownEscape(b as char)),
                };
                in_escape = false;
            } else {
                match b {
                    b'\\' => in_escape = true,
                    b'"' => {
                        in_literal = false;
                        tokens.push(Token::Literal(
                            String::from_utf8(this.clone()).map_err(|_| Error::CorruptUnicode)?,
                        ));
                        this.clear();
                    }
                    _ => this.push(b),
                }
            }
        } else if b"\"() ,".contains(&b) {
            if !this.is_empty() {
                tokens.push(Token::Ident(
                    String::from_utf8(this.clone()).map_err(|_| Error::CorruptUnicode)?,
                ));
                this.clear();
            }

            match b {
                b'"' => in_literal = true,
                b'(' => tokens.push(Token::LeftParen),
                b')' => tokens.push(Token::RightParen),
                b' ' => { /* skip */ }
                b',' => tokens.push(Token::Comma),
                _ => unreachable!(),
            }
        } else {
            this.push(b);
        }
    }

    if in_literal {
        return Err(Error::IncompleteLiteral);
    }
    if !this.is_empty() {
        tokens.push(Token::Ident(
            String::from_utf8(this.clone()).map_err(|_| Error::CorruptUnicode)?,
        ));
    }

    Ok(tokens)
}

fn parse_arg2(tokens: &[Token]) -> Result<(&[Token], &[Token]), Error> {
    // Input: do_a(do_b(), "a"), do_c()
    // Output I: do_a(do_b())
    // Output II: do_c()
    let mut current_parens = 0;
    let mut comma = None;
    for (pos, token) in tokens.iter().enumerate() {
        match token {
            Token::Comma => {
                if current_parens == 0 {
                    comma = Some(pos);
                }
            }
            Token::LeftParen => current_parens += 1,
            Token::RightParen => current_parens -= 1,
            _ => (),
        }
    }
    if current_parens != 0 {
        return Err(Error::BadFunctionCall);
    }
    let Some(comma) = comma else {
        return Err(Error::BadFunctionCall);
    };
    if tokens.len() == comma {
        return Err(Error::BadFunctionCall);
    }
    Ok((&tokens[0..comma], &tokens[comma + 1..]))
}

pub fn addflag(s: String) {
    FLAGS.write().unwrap().insert(s);
}

pub fn delflag(s: &str) {
    FLAGS.write().unwrap().remove(s);
}

fn flag(s: &str) -> bool {
    FLAGS.read().unwrap().contains(s)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unknown escape: {0}")]
    UnknownEscape(char),

    #[error("corrupt unicode")]
    CorruptUnicode,

    #[error("incomplete literal")]
    IncompleteLiteral,

    #[error("expected {0}, found {1:?}")]
    Expected(&'static str, Option<Token>),

    #[error("function not found: {0}")]
    FnNotFound(String),

    #[error("bad function call")]
    BadFunctionCall,
}
