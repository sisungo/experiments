//! Object Storage abstractions of Vinyl.

mod error;
mod s3;

pub use error::Error;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, time::Duration};
use tokio::io::AsyncRead;
use url::Url;

/// An object key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ObjectKey(pub String);

/// An OSS client.
#[async_trait]
pub trait Objects: Debug + Send + Sync {
    /// Returns a presigned URL for the given object key.
    async fn get_url(&self, key: ObjectKey, expires_in: Duration) -> Result<Url, Error>;

    /// Uploads a stream to the object storage.
    async fn put_stream(
        &self,
        key: ObjectKey,
        stream: &mut (dyn AsyncRead + Send + Unpin),
    ) -> Result<(), Error>;

    /// Removes an object from the object storage.
    async fn remove(&self, key: ObjectKey) -> Result<(), Error>;
}

/// Connects to the object storage.
pub async fn connect(vendor: &str) -> Result<Box<dyn Objects>, Error> {
    match vendor {
        "s3" => {
            let aws_config = aws_config::load_from_env().await;
            let s3 = aws_sdk_s3::Client::new(&aws_config);
            Ok(Box::new(s3::S3(s3)))
        }
        vendor => Err(Error::UnknownVendor(vendor.into())),
    }
}
