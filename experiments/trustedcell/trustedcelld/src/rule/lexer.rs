use anyhow::anyhow;
use std::{
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Kind {
    Ident(String),
    Comma,
    Literal(String),
    LeftParen,
    RightParen,
    Semicolon,
}

#[derive(Debug, Clone)]
pub struct Span {
    file: PathBuf,
    line: usize,
    column: usize,
}
impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file.display(), self.line, self.column)
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: Kind,
    pub span: Span,
}
impl Token {
    pub fn new(span: Span, kind: Kind) -> Self {
        Self { kind, span }
    }
}

pub fn process_file(file: &Path) -> anyhow::Result<Vec<Token>> {
    let mut s = String::new();
    for line in BufReader::new(File::open(file)?).lines() {
        let line = line?;
        if line.starts_with("// ") || line.starts_with("# ") {
            continue;
        }
        s.push_str(&line);
        s.push('\n');
    }
    let mut tokens = Vec::with_capacity(s.len() / 4);
    let mut in_literal = false;
    let mut in_escape = false;
    let mut this = Vec::with_capacity(8);

    for (line_number, line_data) in s.lines().enumerate() {
        for (column_number, column_byte) in line_data.bytes().enumerate() {
            let span = Span {
                file: file.to_path_buf(),
                line: line_number + 1,
                column: column_number + 1,
            };
            if in_literal {
                if in_escape {
                    match column_byte {
                        b'n' => this.push(b'\n'),
                        b'r' => this.push(b'\r'),
                        b'"' => this.push(b'"'),
                        b'\\' => this.push(b'\\'),
                        b'0' => this.push(b'\0'),
                        _ => return Err(anyhow!("unknown escape `\\{}`", column_byte as char)),
                    };
                    in_escape = false;
                } else {
                    match column_byte {
                        b'\\' => in_escape = true,
                        b'"' => {
                            in_literal = false;
                            let token_kind = Kind::Literal(
                                String::from_utf8(this.clone())
                                    .map_err(|_| anyhow!("not valid utf-8 string"))?,
                            );
                            tokens.push(Token::new(span, token_kind));
                            this.clear();
                        }
                        _ => this.push(column_byte),
                    }
                }
            } else if b"\"() ,;".contains(&column_byte) {
                if !this.is_empty() {
                    let token_kind = Kind::Ident(
                        String::from_utf8(this.clone())
                            .map_err(|_| anyhow!("not valid utf-8 string"))?,
                    );
                    tokens.push(Token::new(span.clone(), token_kind));
                    this.clear();
                }

                match column_byte {
                    b'"' => in_literal = true,
                    b'(' => tokens.push(Token::new(span, Kind::LeftParen)),
                    b')' => tokens.push(Token::new(span, Kind::RightParen)),
                    b' ' => { /* skip */ }
                    b',' => tokens.push(Token::new(span, Kind::Comma)),
                    b';' => tokens.push(Token::new(span, Kind::Semicolon)),
                    _ => unreachable!(),
                }
            } else {
                this.push(column_byte);
            }
        }
    }

    if in_literal {
        return Err(anyhow!("incomplete literal"));
    }
    if !this.is_empty() {
        let token_kind = Kind::Ident(
            String::from_utf8(this.clone()).map_err(|_| anyhow!("not valid utf-8 string"))?,
        );
        let span = Span {
            file: file.to_path_buf(),
            line: 0,
            column: 0,
        };
        tokens.push(Token::new(span, token_kind));
    }

    Ok(tokens)
}
