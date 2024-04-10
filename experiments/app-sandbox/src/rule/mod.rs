mod lexer;

use eyre::OptionExt;
use landlock::{AccessFs, BitFlags, PathBeneath, PathFd, RulesetCreated, RulesetCreatedAttr};
use std::path::Path;

use self::lexer::{Token, TokenKind};

pub fn parse_to(ruleset: &mut RulesetCreated, file: &Path, s: &str) -> eyre::Result<()> {
    let tokens = lexer::tokenize(file, s)?;
    let statements = tokens.split(|x| matches!(*x.kind, TokenKind::Semicolon));

    for statement in statements {
        parse_statement(ruleset, statement)?;
    }

    Ok(())
}

fn parse_statement(ruleset: &mut RulesetCreated, s: &[Token]) -> eyre::Result<()> {
    match s.first() {
        Some(first_token) => match &*first_token.kind {
            TokenKind::Ident(ident) => match ident.as_str() {
                "use" => parse_use(ruleset, &s[1..]),
                "allow" => parse_allow(ruleset, &s[1..]),
                _ => Err(eyre::eyre!("")),
            },
            _ => Err(eyre::eyre!("")),
        },
        None => Ok(()),
    }
}

fn parse_allow(ruleset: &mut RulesetCreated, s: &[Token]) -> eyre::Result<()> {
    let mut splited = s.split(|x| matches!(&*x.kind, TokenKind::Ident(x) if x == "on"));
    let actions = splited.next().unwrap();
    let object = splited.next().ok_or_eyre("")?;
    if splited.next().is_some() {
        Err(eyre::eyre!(""))?;
    }
    let actions = actions.split(|x| matches!(&*x.kind, TokenKind::Comma));

    match &*object.first().unwrap().kind {
        TokenKind::Ident(ident) => match ident.as_str() {
            "path" => match &*object.get(1).ok_or_eyre("")?.kind {
                TokenKind::Literal(path) => {
                    let mut access_fs: BitFlags<AccessFs> = BitFlags::empty();

                    for action in actions {
                        if action.len() != 1 {
                            return Err(eyre::eyre!(""));
                        }

                        match &*action.first().unwrap().kind {
                            TokenKind::Ident(ident) => match ident.as_str() {
                                "read_file" => access_fs |= AccessFs::ReadFile,
                                "write_file" => access_fs |= AccessFs::WriteFile,
                                "execute" => access_fs |= AccessFs::Execute,
                                "read_dir" => access_fs |= AccessFs::ReadDir,
                                "remove_file" => access_fs |= AccessFs::RemoveFile,
                                "remove_dir" => access_fs |= AccessFs::RemoveDir,
                                "make_symlink" => access_fs |= AccessFs::MakeSym,
                                "make_socket" => access_fs |= AccessFs::MakeSock,
                                "make_reg" => access_fs |= AccessFs::MakeReg,
                                "make_fifo" => access_fs |= AccessFs::MakeFifo,
                                "make_dir" => access_fs |= AccessFs::MakeDir,
                                "refer" => access_fs |= AccessFs::Refer,
                                "truncate" => access_fs |= AccessFs::Truncate,
                                "make_char" => access_fs |= AccessFs::MakeChar,
                                "make_block" => access_fs |= AccessFs::MakeBlock,
                                _ => return Err(eyre::eyre!("")),
                            },
                            _ => return Err(eyre::eyre!("")),
                        }
                    }

                    ruleset.add_rule(PathBeneath::new(PathFd::new(path)?, access_fs))?;
                }
                _ => return Err(eyre::eyre!("")),
            },
            _ => return Err(eyre::eyre!("")),
        },
        _ => return Err(eyre::eyre!("")),
    }

    Ok(())
}

fn parse_use(ruleset: &mut RulesetCreated, s: &[Token]) -> eyre::Result<()> {
    match s.first() {
        Some(token) => match &*token.kind {
            TokenKind::Literal(literal) => match literal.as_str() {
                "+set_no_new_privs" => {
                    ruleset.set_no_new_privs(true);
                    Ok(())
                }
                "-set_no_new_privs" => {
                    ruleset.set_no_new_privs(false);
                    Ok(())
                }
                _ => Err(eyre::eyre!("")),
            },
            _ => Err(eyre::eyre!("")),
        },
        None => Err(eyre::eyre!("")),
    }
}
