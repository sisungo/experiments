use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Procedure {
    Replace(String, String),
    UnsplitLines,
    OnParaBegin(String),
    Trim(Option<String>),
    TrimBegin(Option<String>),
}
impl Procedure {
    pub fn run(&self, s: String) -> String {
        match self {
            Self::Replace(from, to) => crate::tools::replace(s, (from, to)),
            Self::UnsplitLines => crate::tools::unsplit_lines(s),
            Self::OnParaBegin(content) => crate::tools::on_para_begin(s, content),
            Self::Trim(mat) => crate::tools::trim(s, mat.as_deref()),
            Self::TrimBegin(mat) => crate::tools::trim_begin(s, mat.as_deref()),
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

    fn try_parse_on_para_begin(tokens: &[Token]) -> Result<Self, Error> {
        let left_paren = matches!(tokens.first(), Some(Token::LeftParen));
        let right_paren = matches!(tokens.get(2), Some(Token::RightParen));
        let valid_len = tokens.len() == 3;
        if !(left_paren && right_paren && valid_len) {
            return Err(Error::BadFunctionCall);
        }
        let Some(Token::Literal(content)) = tokens.get(1) else {
            return Err(Error::Expected("literal", tokens.get(1).cloned()));
        };

        Ok(Self::OnParaBegin(content.clone()))
    }

    fn try_parse_trim(var: &'static str, tokens: &[Token]) -> Result<Self, Error> {
        let left_paren = matches!(tokens.first(), Some(Token::LeftParen));
        match tokens.len() {
            2 => {
                let right_paren = matches!(tokens.get(2), Some(Token::RightParen));
                if !(left_paren && right_paren) {
                    return Err(Error::BadFunctionCall);
                }
                Ok(match var {
                    "" => Self::Trim(None),
                    "begin" => Self::TrimBegin(None),
                    _ => unreachable!(),
                })
            }
            3 => {
                let right_paren = matches!(tokens.get(3), Some(Token::RightParen));
                if !(left_paren && right_paren) {
                    return Err(Error::BadFunctionCall);
                }
                let Some(Token::Literal(mat)) = tokens.get(1) else {
                    return Err(Error::Expected("literal", tokens.get(1).cloned()));
                };
                Ok(match var {
                    "" => Self::Trim(Some(mat.to_owned())),
                    "begin" => Self::TrimBegin(Some(mat.to_owned())),
                    _ => unreachable!(),
                })
            }
            _ => Err(Error::BadFunctionCall),
        }
    }
}
impl FromStr for Procedure {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = tokenize(s)?;
        if let Some(Token::Ident(ident)) = tokens.first() {
            match &ident[..] {
                "replace" => Self::try_parse_replace(&tokens[1..]),
                "unsplit_lines" => Self::try_parse_unsplit_lines(&tokens[1..]),
                "on_para_begin" => Self::try_parse_on_para_begin(&tokens[1..]),
                "trim" => Self::try_parse_trim("", &tokens[1..]),
                "trim_begin" => Self::try_parse_trim("begin", &tokens[1..]),
                _ => Err(Error::FnNotFound(ident.clone())),
            }
        } else {
            Err(Error::Expected("ident", tokens.first().cloned()))
        }
    }
}

pub fn parse(s: &str) -> Result<Vec<Procedure>, Error> {
    let mut procedures = Vec::with_capacity(s.len() / 8);

    for l in s.lines() {
        if l.starts_with("//") {
            continue;
        }
        if l.trim().is_empty() {
            continue;
        }
        procedures.push(l.parse()?);
    }

    Ok(procedures)
}

#[derive(Debug, Clone)]
pub enum Token {
    Ident(String),
    Comma,
    Literal(String),
    LeftParen,
    RightParen,
}

pub fn tokenize(s: &str) -> Result<Vec<Token>, Error> {
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
