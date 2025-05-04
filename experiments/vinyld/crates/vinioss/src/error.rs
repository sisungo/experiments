use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    ObjectNotFound,
    UnknownVendor(String),
    InvalidConfiguration(Box<dyn std::error::Error + Send + Sync>),
    Unrecognized(Box<dyn std::error::Error + Send + Sync>),
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ObjectNotFound => write!(f, "object not found"),
            Self::UnknownVendor(vendor) => write!(f, "unknown object storage vendor `{vendor}`"),
            Self::InvalidConfiguration(err) => write!(f, "invalid configuration: {err}"),
            Self::Unrecognized(err) => err.fmt(f),
        }
    }
}
impl std::error::Error for Error {}
