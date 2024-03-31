use once_cell::sync::Lazy;
use rayon::{iter::ParallelIterator, str::ParallelString};
use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
    path::Path,
    str::FromStr,
    sync::RwLock,
};

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

static FLAGS: Lazy<RwLock<HashSet<String>>> = Lazy::new(RwLock::default);
static STORED_PROCEDURES: Lazy<RwLock<HashMap<String, Procedure>>> = Lazy::new(RwLock::default);
static COUNTERS: Lazy<RwLock<HashMap<String, i32>>> = Lazy::new(RwLock::default);

#[derive(Debug, Clone)]
pub enum Procedure {
    Replace(String, String),
    UnsplitLines,
    OnParaBeginFmt(Vec<String>),
    MakeParaBeginWith(String),
    Trim(Option<String>),
    TrimBegin(Option<String>),
    TrimEnd(Option<String>),
    Append(String),
    AppendLine(String),
    AppendFmt(Vec<String>),
    RepeatN(u32, Box<Procedure>),
    RepeatUntil(Condition, Box<Procedure>),
    If(Condition, Box<Procedure>),
    AddFlag(String),
    DelFlag(String),
    EachLine(Box<Procedure>),
    ParEachLine(Box<Procedure>),
    Lambda(Vec<Procedure>),
    StoreProc(String, Box<Procedure>),
    LoadProc(String),
    InitCounter(String, i32),
    AddCounter(String, String),
    DupCounter(String, String),
}
impl Procedure {
    pub fn run(&self, s: String) -> String {
        match self {
            Self::Replace(from, to) => crate::tools::replace(s, (from, to)),
            Self::UnsplitLines => crate::tools::unsplit_lines(s),
            Self::OnParaBeginFmt(p) => crate::tools::fmt("ob", s, p),
            Self::MakeParaBeginWith(content) => crate::tools::make_para_begin_with(s, content),
            Self::Trim(mat) => crate::tools::trim(s, mat.as_deref()),
            Self::TrimBegin(mat) => crate::tools::trim_begin(s, mat.as_deref()),
            Self::TrimEnd(mat) => crate::tools::trim_end(s, mat.as_deref()),
            Self::RepeatN(n, p) => Self::repeatn(*n, p, s),
            Self::Append(p) => crate::tools::append(s, p),
            Self::AppendLine(p) => crate::tools::append_line(s, p),
            Self::AppendFmt(p) => crate::tools::fmt("append", s, p),
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
            Self::ParEachLine(p) => Self::par_each_line(p, s),
            Self::Lambda(procs) => Self::lambda(procs, s),
            Self::StoreProc(name, proc) => {
                store_procedure(name.clone(), *proc.clone());
                s
            }
            Self::LoadProc(name) => Self::load_procedure(name, s),
            Self::InitCounter(name, val) => {
                init_counter(name.clone(), *val);
                s
            }
            Self::AddCounter(a, b) => Self::add_counter(a, b, s),
            Self::DupCounter(a, b) => Self::dup_counter(a, b, s),
        }
    }

    fn add_counter(a: &str, b: &str, s: String) -> String {
        let a_val = counter(a).unwrap_or_default();
        let b_val = counter(b).unwrap_or_default();
        init_counter(a.into(), a_val + b_val);
        s
    }

    fn dup_counter(a: &str, b: &str, s: String) -> String {
        init_counter(b.into(), counter(a).unwrap_or_default());
        s
    }

    fn load_procedure(name: &str, s: String) -> String {
        match load_procedure(name) {
            Some(proc) => proc.run(s),
            None => s,
        }
    }

    fn lambda(procs: &[Procedure], mut s: String) -> String {
        for p in procs {
            s = p.run(s);
        }
        s
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

    fn par_each_line(p: &Self, s: String) -> String {
        s.par_lines()
            .map(|x| x.to_owned())
            .reduce(
                || String::with_capacity(s.len()),
                |mut acc, line| {
                    writeln!(&mut acc, "{}", p.run(line.to_owned())).unwrap();
                    acc
                },
            )
            .to_string()
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
                "replace" => Self::try_parse_twoliteral("replace", &tokens[1..]),
                "unsplit_lines" => Self::try_parse_unsplit_lines(&tokens[1..]),
                "on_front" => Self::try_parse_fmt("opb", &tokens[1..]),
                "make_para_begin_with" => Self::try_parse_oneliteral("mpbw", &tokens[1..]),
                "trim" => Self::try_parse_trim("", &tokens[1..]),
                "trim_begin" => Self::try_parse_trim("begin", &tokens[1..]),
                "trim_end" => Self::try_parse_trim("end", &tokens[1..]),
                "repeatn" => Self::try_parse_repeatn(&tokens[1..]),
                "repeat_until" => Self::try_parse_ifuntil("until", &tokens[1..]),
                "append" => Self::try_parse_oneliteral("append", &tokens[1..]),
                "append_line" => Self::try_parse_oneliteral("appendline", &tokens[1..]),
                "append_fmt" => Self::try_parse_fmt("append", &tokens[1..]),
                "if" => Self::try_parse_ifuntil("if", &tokens[1..]),
                "addflag" => Self::try_parse_oneliteral("addflag", &tokens[1..]),
                "delflag" => Self::try_parse_oneliteral("delflag", &tokens[1..]),
                "each_line" => Self::try_parse_each_line("", &tokens[1..]),
                "par_each_line" => Self::try_parse_each_line("par", &tokens[1..]),
                "lambda" => Self::try_parse_lambda(&tokens[1..]),
                "storeproc" => Self::try_parse_storeproc(&tokens[1..]),
                "loadproc" => Self::try_parse_oneliteral("loadproc", &tokens[1..]),
                "initctr" => Self::try_parse_init_counter(&tokens[1..]),
                "addctr" => Self::try_parse_twoliteral("add", &tokens[1..]),
                "dupctr" => Self::try_parse_twoliteral("dup", &tokens[1..]),
                _ => Err(Error::FnNotFound(ident.clone())),
            }
        } else {
            Err(Error::Expected("ident", tokens.first().cloned()))
        }
    }

    fn try_parse_twoliteral(var: &'static str, tokens: &[Token]) -> Result<Self, Error> {
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

        Ok(match var {
            "replace" => Self::Replace(from.clone(), to.clone()),
            "add" => Self::AddCounter(from.clone(), to.clone()),
            "dup" => Self::DupCounter(from.clone(), to.clone()),
            _ => unreachable!(),
        })
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

        Ok(match var {
            "append" => Self::Append(content.clone()),
            "appendline" => Self::AppendLine(content.clone()),
            "mpbw" => Self::MakeParaBeginWith(content.clone()),
            "addflag" => Self::AddFlag(content.clone()),
            "delflag" => Self::DelFlag(content.clone()),
            "loadproc" => Self::LoadProc(content.clone()),
            _ => unreachable!(),
        })
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
        let (times, body) = parse_condblk(tokens)?;
        let left_paren = matches!(body.first(), Some(Token::LeftParen));
        let right_paren = matches!(body.last(), Some(Token::RightParen));
        if !(left_paren && right_paren) {
            return Err(Error::BadFunctionCall);
        }
        let Some(Token::Ident(times)) = times.first() else {
            return Err(Error::Expected("ident", times.first().cloned()));
        };
        let Ok(times) = times.parse::<u32>() else {
            return Err(Error::Expected("number", Some(Token::Ident(times.clone()))));
        };
        let mut body = body.to_vec();
        body.insert(0, Token::Ident("lambda".into()));
        let p = Self::parse_tokens(&body)?;
        Ok(Self::RepeatN(times, Box::new(p)))
    }

    fn try_parse_ifuntil(var: &'static str, tokens: &[Token]) -> Result<Self, Error> {
        let (cond, body) = parse_condblk(tokens)?;
        let left_paren = matches!(body.first(), Some(Token::LeftParen));
        let right_paren = matches!(body.last(), Some(Token::RightParen));
        if !(left_paren && right_paren) {
            return Err(Error::BadFunctionCall);
        }
        let mut body = body.to_vec();
        body.insert(0, Token::Ident("lambda".into()));
        let proc = Box::new(Self::parse_tokens(&body)?);
        let cond = Condition::parse_tokens(cond)?;
        Ok(match var {
            "if" => Self::If(cond, proc),
            "until" => Self::RepeatUntil(cond, proc),
            _ => unreachable!(),
        })
    }

    fn try_parse_each_line(var: &'static str, tokens: &[Token]) -> Result<Self, Error> {
        let left_paren = matches!(tokens.first(), Some(Token::LeftParen));
        let right_paren = matches!(tokens.last(), Some(Token::RightParen));
        if !(left_paren && right_paren) {
            return Err(Error::BadFunctionCall);
        }
        let proc = Self::parse_tokens(&tokens[1..tokens.len() - 1])?;
        Ok(match var {
            "" => Self::EachLine(Box::new(proc)),
            "par" => Self::ParEachLine(Box::new(proc)),
            _ => unreachable!(),
        })
    }

    fn try_parse_lambda(tokens: &[Token]) -> Result<Self, Error> {
        let left_paren = matches!(tokens.first(), Some(Token::LeftParen));
        let right_paren = matches!(tokens.last(), Some(Token::RightParen));
        if !(left_paren && right_paren) {
            return Err(Error::BadFunctionCall);
        }
        let procs_tokens = parse_argn(&tokens[1..tokens.len() - 1]);
        let mut procs = Vec::with_capacity(procs_tokens.len());
        for tok in procs_tokens {
            let proc = Self::parse_tokens(tok)?;
            procs.push(proc);
        }
        Ok(Self::Lambda(procs))
    }

    fn try_parse_storeproc(tokens: &[Token]) -> Result<Self, Error> {
        let (name, body) = parse_condblk(tokens)?;
        let left_paren = matches!(body.first(), Some(Token::LeftParen));
        let right_paren = matches!(body.last(), Some(Token::RightParen));
        if !(left_paren && right_paren) {
            return Err(Error::BadFunctionCall);
        }
        let Some(Token::Literal(name)) = name.first() else {
            return Err(Error::Expected("literal", name.first().cloned()));
        };
        let mut body = body.to_vec();
        body.insert(0, Token::Ident("lambda".into()));
        let p = Self::parse_tokens(&body)?;
        Ok(Self::StoreProc(name.clone(), Box::new(p)))
    }

    fn try_parse_init_counter(tokens: &[Token]) -> Result<Self, Error> {
        let left_paren = matches!(tokens.first(), Some(Token::LeftParen));
        let right_paren = matches!(tokens.get(4), Some(Token::RightParen));
        let comma = matches!(tokens.get(2), Some(Token::Comma));
        let valid_len = tokens.len() == 5;
        if !(left_paren && right_paren && comma && valid_len) {
            return Err(Error::BadFunctionCall);
        }
        let Some(Token::Literal(name)) = tokens.get(1) else {
            return Err(Error::Expected("literal", tokens.get(1).cloned()));
        };
        let Some(Token::Ident(val)) = tokens.get(3) else {
            return Err(Error::Expected("ident", tokens.get(3).cloned()));
        };
        let Ok(val) = val.parse::<i32>() else {
            return Err(Error::Expected("number", Some(Token::Ident(val.clone()))));
        };

        Ok(Self::InitCounter(name.clone(), val))
    }

    fn try_parse_fmt(var: &'static str, tokens: &[Token]) -> Result<Self, Error> {
        let left_paren = matches!(tokens.first(), Some(Token::LeftParen));
        let right_paren = matches!(tokens.last(), Some(Token::RightParen));
        if !(left_paren && right_paren) {
            return Err(Error::BadFunctionCall);
        }
        let strs_tokens = parse_argn(&tokens[1..tokens.len() - 1]);
        let mut strs = Vec::with_capacity(strs_tokens.len());
        for tok in strs_tokens {
            if tok.len() != 1 {
                return Err(Error::BadFunctionCall);
            }
            let Some(Token::Literal(s)) = tok.first() else {
                return Err(Error::BadFunctionCall);
            };
            strs.push(s.clone());
        }
        Ok(match var {
            "append" => Self::AppendFmt(strs),
            "opb" => Self::OnParaBeginFmt(strs),
            _ => unreachable!(),
        })
    }
}
impl FromStr for Procedure {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = tokenize(s)?;
        Self::parse_tokens(&tokens)
    }
}

pub fn parse(path: &Path) -> Result<Vec<Procedure>, Error> {
    let s = std::fs::read_to_string(path).map_err(|_| Error::Io)?;
    let mut procedures = Vec::with_capacity(s.len() / 8);

    let s =
        s.lines()
            .map(|line| line.trim())
            .fold(String::with_capacity(s.len()), |mut acc, line| {
                if line.as_bytes().last().copied() == Some(b'\\') {
                    write!(&mut acc, "{}", &line[..line.len() - 1]).unwrap();
                } else {
                    writeln!(&mut acc, "{line}").unwrap();
                }
                acc
            });

    for l in s.lines() {
        if l.starts_with("//") || l.is_empty() {
            continue;
        }
        if let Some(x) = l.strip_prefix("#include :: ") {
            let mut path = path.to_path_buf();
            path.pop();
            let path = path.join(x);
            let mut procs = parse(&path).map_err(|_| Error::BadInclude)?;
            procedures.append(&mut procs);
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
    CounterEq(String, String),
    CounterLt(String, String),
    CounterMt(String, String),
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
            Self::CounterEq(a, b) => {
                counter(a).unwrap_or_default() == counter(b).unwrap_or_default()
            }
            Self::CounterLt(a, b) => {
                counter(a).unwrap_or_default() < counter(b).unwrap_or_default()
            }
            Self::CounterMt(a, b) => {
                counter(a).unwrap_or_default() > counter(b).unwrap_or_default()
            }
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
                "ctreq" => Self::try_parse_ctrcmp("eq", &tokens[1..]),
                "ctrlt" => Self::try_parse_ctrcmp("lt", &tokens[1..]),
                "ctrmt" => Self::try_parse_ctrcmp("mt", &tokens[1..]),
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
        Ok(match var {
            "contains" => Self::Contains(pat.clone()),
            "content_eq" => Self::ContentEq(pat.clone()),
            "flag" => Self::Flag(pat.clone()),
            _ => unreachable!(),
        })
    }

    fn try_parse_allany(var: &'static str, tokens: &[Token]) -> Result<Self, Error> {
        let left_paren = matches!(tokens.first(), Some(Token::LeftParen));
        let right_paren = matches!(tokens.last(), Some(Token::RightParen));
        if !(left_paren && right_paren) {
            return Err(Error::BadFunctionCall);
        }
        let (a, b) = parse_arg2(&tokens[1..tokens.len() - 1])?;
        let a = Box::new(Self::parse_tokens(a)?);
        let b = Box::new(Self::parse_tokens(b)?);
        Ok(match var {
            "all" => Self::All(a, b),
            "any" => Self::Any(a, b),
            _ => unreachable!(),
        })
    }

    fn try_parse_ctrcmp(var: &'static str, tokens: &[Token]) -> Result<Self, Error> {
        let left_paren = matches!(tokens.first(), Some(Token::LeftParen));
        let right_paren = matches!(tokens.get(4), Some(Token::RightParen));
        let comma = matches!(tokens.get(2), Some(Token::Comma));
        if !(left_paren && right_paren && comma) {
            return Err(Error::BadFunctionCall);
        }
        let Some(Token::Literal(a)) = tokens.get(1) else {
            return Err(Error::BadFunctionCall);
        };
        let Some(Token::Literal(b)) = tokens.get(3) else {
            return Err(Error::BadFunctionCall);
        };
        Ok(match var {
            "eq" => Self::CounterEq(a.clone(), b.clone()),
            "lt" => Self::CounterLt(a.clone(), b.clone()),
            "mt" => Self::CounterMt(a.clone(), b.clone()),
            _ => unreachable!(),
        })
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
                    b'0' => this.push(b'\0'),
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
                    break;
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

fn parse_condblk(tokens: &[Token]) -> Result<(&[Token], &[Token]), Error> {
    // Input: (do_b()) ( ... )
    // Output I: do_b()
    // Output II: ( ... )
    let mut current_parens = 0;
    let mut stop = None;
    for (pos, token) in tokens.iter().enumerate() {
        match token {
            Token::LeftParen => current_parens += 1,
            Token::RightParen => {
                current_parens -= 1;
                if current_parens == 0 {
                    stop = Some(pos);
                    break;
                }
            }
            _ => (),
        }
    }
    if current_parens != 0 {
        return Err(Error::BadFunctionCall);
    }
    let Some(stop) = stop else {
        return Err(Error::BadFunctionCall);
    };
    if tokens.len() == stop {
        return Err(Error::BadFunctionCall);
    }
    Ok((&tokens[1..stop], &tokens[stop + 1..]))
}

fn parse_argn(mut remaining: &[Token]) -> Vec<&[Token]> {
    let mut result = Vec::with_capacity(remaining.len() / 8);

    loop {
        match parse_arg2(remaining) {
            Ok((a, b)) => {
                result.push(a);
                remaining = b;
            }
            Err(_) => {
                result.push(remaining);
                break;
            }
        }
    }

    result
}

fn store_procedure(name: String, proc: Procedure) {
    STORED_PROCEDURES.write().unwrap().insert(name, proc);
}

fn load_procedure(name: &str) -> Option<Procedure> {
    STORED_PROCEDURES.read().unwrap().get(name).cloned()
}

pub fn addflag(s: String) {
    FLAGS.write().unwrap().insert(s);
}

fn delflag(s: &str) {
    FLAGS.write().unwrap().remove(s);
}

fn flag(s: &str) -> bool {
    FLAGS.read().unwrap().contains(s)
}

fn init_counter(name: String, val: i32) {
    COUNTERS.write().unwrap().insert(name, val);
}

pub fn counter(name: &str) -> Option<i32> {
    COUNTERS.read().unwrap().get(name).copied()
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

    #[error("bad include")]
    BadInclude,

    #[error("I/O Error")]
    Io,
}
