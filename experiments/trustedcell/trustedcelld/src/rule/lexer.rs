use std::path::Path;

pub struct Token {
    kind: Kind,
    span: Span,
}

pub enum Kind {}

pub struct Span {}

pub fn process_file(path: &Path) -> Vec<Token> {
    todo!()
}
