use std::{
    fmt::Display,
    path::{Path, PathBuf},
    rc::Rc,
};

#[derive(Debug, Clone)]
pub enum TokenKind {
    Ident(String),
    Comma,
    Literal(String),
    LeftParen,
    RightParen,
    Semicolon,
}

#[derive(Debug, Clone)]
pub struct TokenSpan {
    file: PathBuf,
    line: usize,
    column: usize,
}
impl Display for TokenSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file.display(), self.line, self.column)
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: Rc<TokenKind>,
    pub span: Rc<TokenSpan>,
}
impl Token {
    pub fn new(span: TokenSpan, kind: TokenKind) -> Self {
        Self {
            kind: Rc::new(kind),
            span: Rc::new(span),
        }
    }
}

pub fn tokenize(file: &Path, s: &str) -> eyre::Result<Vec<Token>> {
    let mut tokens = Vec::with_capacity(s.len() / 4);
    let mut in_literal = false;
    let mut in_escape = false;
    let mut this = Vec::with_capacity(8);

    for (line_number, line_data) in s.lines().enumerate() {
        for (column_number, column_byte) in line_data.bytes().enumerate() {
            let span = TokenSpan {
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
                        _ => return Err(eyre::eyre!("unknown escape `\\{}`", column_byte as char)),
                    };
                    in_escape = false;
                } else {
                    match column_byte {
                        b'\\' => in_escape = true,
                        b'"' => {
                            in_literal = false;
                            let token_kind = TokenKind::Literal(
                                String::from_utf8(this.clone())
                                    .map_err(|_| eyre::eyre!("not valid utf-8 string"))?,
                            );
                            tokens.push(Token::new(span, token_kind));
                            this.clear();
                        }
                        _ => this.push(column_byte),
                    }
                }
            } else if b"\"() ,;".contains(&column_byte) {
                if !this.is_empty() {
                    let token_kind = TokenKind::Ident(
                        String::from_utf8(this.clone())
                            .map_err(|_| eyre::eyre!("not valid utf-8 string"))?,
                    );
                    tokens.push(Token::new(span.clone(), token_kind));
                    this.clear();
                }

                match column_byte {
                    b'"' => in_literal = true,
                    b'(' => tokens.push(Token::new(span, TokenKind::LeftParen)),
                    b')' => tokens.push(Token::new(span, TokenKind::RightParen)),
                    b' ' => { /* skip */ }
                    b',' => tokens.push(Token::new(span, TokenKind::Comma)),
                    b';' => tokens.push(Token::new(span, TokenKind::Semicolon)),
                    _ => unreachable!(),
                }
            } else {
                this.push(column_byte);
            }
        }
    }

    if in_literal {
        return Err(eyre::eyre!("incomplete literal"));
    }
    if !this.is_empty() {
        let token_kind = TokenKind::Ident(
            String::from_utf8(this.clone()).map_err(|_| eyre::eyre!("not valid utf-8 string"))?,
        );
        let span = TokenSpan {
            file: file.to_path_buf(),
            line: 0,
            column: 0,
        };
        tokens.push(Token::new(span, token_kind));
    }

    Ok(tokens)
}
