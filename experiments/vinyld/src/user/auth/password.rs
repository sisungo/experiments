use crate::error::Error;
use argon2::Argon2;
use vinutie::def_verify;

def_verify!(pub PasswordLike<str>(err: Error = Error::invalid_password()) = |x: &str| x.len() <= 128);

pub fn hasher() -> Argon2<'static> {
    Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::default(),
    )
}
