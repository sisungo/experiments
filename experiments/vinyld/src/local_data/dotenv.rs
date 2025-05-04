use anyhow::anyhow;
use std::str::FromStr;
use url::Url;

pub const DATABASE_URL: &str = "DATABASE_URL";
pub const DATABASE_CRON_ENABLED: &str = "DATABASE_CRON_ENABLED";
pub const DATABASE_DANGEROUS_FRESH_MIGRATIONS: &str = "DATABASE_DANGEROUS_FRESH_MIGRATIONS";
pub const NETWORK_LISTEN_URL: &str = "NETWORK_LISTEN_URL";
pub const S3_BUCKET: &str = "S3_BUCKET";
pub const OBJECT_STORAGE: &str = "OBJECT_STORAGE";

/// Fetches an environment variable and parses it into a type.
pub fn fetch_env<T>(key: &str) -> anyhow::Result<T>
where
    T: FromStr,
    T::Err: std::error::Error,
{
    std::env::var(key)
        .map_err(|err| anyhow!("failed to get environment variable `{}`: {}", key, err))?
        .parse()
        .map_err(|err| anyhow!("failed to parse environment variable `{}`: {}", key, err))
}

/// Checks that all required environment variables are set.
pub fn check_required() -> anyhow::Result<()> {
    fetch_env::<String>(DATABASE_URL)?;
    fetch_env::<bool>(DATABASE_CRON_ENABLED)?;
    fetch_env::<Url>(NETWORK_LISTEN_URL)?;

    let object_storage = fetch_env::<String>(OBJECT_STORAGE)?;
    if object_storage == "s3" {
        fetch_env::<String>(S3_BUCKET)?;
    }

    Ok(())
}
