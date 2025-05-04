use crate::{ObjectKey, Objects, error::Error};
use async_trait::async_trait;
use aws_sdk_s3::{
    operation::get_object::GetObjectError,
    presigning::PresigningConfig,
    primitives::ByteStream,
    types::{CompletedMultipartUpload, CompletedPart},
};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt};
use url::Url;

/// An S3 client.
#[derive(Debug)]
pub struct S3(pub aws_sdk_s3::Client);
impl S3 {
    /// Returns a presigning config that expires in given duration.
    fn presigning_config(expires_in: Duration) -> PresigningConfig {
        PresigningConfig::builder()
            .expires_in(expires_in)
            .build()
            .unwrap()
    }

    /// Uploads a stream to the object storage by parts.
    async fn upload_by_parts(
        &self,
        upload_id: String,
        key: ObjectKey,
        stream: &mut (dyn AsyncRead + Send + Unpin),
    ) -> Result<Vec<CompletedPart>, Error> {
        const BUF_SIZE: usize = 6 * 1024 * 1024;

        let mut part_number = 1;
        let mut completed_parts = Vec::new();
        let mut buf = Vec::new();

        loop {
            let n = stream
                .take(BUF_SIZE as _)
                .read_to_end(&mut buf)
                .await
                .map_err(|err| Error::Unrecognized(err.into()))?;

            if n == 0 {
                break;
            }

            let part = self
                .0
                .upload_part()
                .bucket(s3_bucket()?)
                .key(key.0.clone())
                .upload_id(upload_id.clone())
                .part_number(part_number)
                .body(ByteStream::from(buf.clone()))
                .send()
                .await
                .map_err(|err| Error::Unrecognized(err.into()))?;

            completed_parts.push(
                CompletedPart::builder()
                    .part_number(part_number)
                    .e_tag(
                        part.e_tag.ok_or_else(|| {
                            Error::Unrecognized("no etag from uploaded part".into())
                        })?,
                    )
                    .build(),
            );

            part_number += 1;
            buf.clear();
        }

        Ok(completed_parts)
    }
}
#[async_trait]
impl Objects for S3 {
    async fn get_url(&self, key: ObjectKey, expires_in: Duration) -> Result<Url, Error> {
        self.0
            .get_object()
            .bucket(s3_bucket()?)
            .key(key.0)
            .presigned(Self::presigning_config(expires_in))
            .await
            .map_err(|err| match err.into_service_error() {
                GetObjectError::NoSuchKey(_) => Error::ObjectNotFound,
                err => Error::Unrecognized(err.into()),
            })?
            .uri()
            .parse::<Url>()
            .map_err(|err| Error::Unrecognized(Box::new(err)))
    }

    async fn put_stream(
        &self,
        key: ObjectKey,
        stream: &mut (dyn AsyncRead + Send + Unpin),
    ) -> Result<(), Error> {
        let upload_id = self
            .0
            .create_multipart_upload()
            .bucket(s3_bucket()?)
            .key(key.0.clone())
            .send()
            .await
            .map_err(|err| Error::Unrecognized(err.into()))?
            .upload_id
            .ok_or_else(|| Error::Unrecognized("no upload id from s3 server".into()))?;

        let completed_parts = match self
            .upload_by_parts(upload_id.clone(), key.clone(), stream)
            .await
        {
            Ok(x) => x,
            Err(err) => {
                self.0
                    .abort_multipart_upload()
                    .bucket(s3_bucket()?)
                    .key(key.0)
                    .upload_id(upload_id)
                    .send()
                    .await
                    .map_err(|err| Error::Unrecognized(err.into()))?;

                return Err(err);
            }
        };

        self.0
            .complete_multipart_upload()
            .bucket(s3_bucket()?)
            .key(key.0)
            .upload_id(upload_id)
            .multipart_upload(
                CompletedMultipartUpload::builder()
                    .set_parts(Some(completed_parts))
                    .build(),
            )
            .send()
            .await
            .map_err(|err| Error::Unrecognized(err.into()))?;

        Ok(())
    }

    async fn remove(&self, key: ObjectKey) -> Result<(), Error> {
        self.0
            .delete_object()
            .bucket(s3_bucket()?)
            .key(key.0)
            .send()
            .await
            .map_err(|err| Error::Unrecognized(err.into()))?;

        Ok(())
    }
}

fn s3_bucket() -> Result<String, Error> {
    std::env::var("S3_BUCKET")
        .map_err(|_| Error::InvalidConfiguration("S3_BUCKET environment variable not set".into()))
}
