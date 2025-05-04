//! Audio format converter.
//!
//! Currently, the implementation is based on FFMpeg command line utility. We are planning to migrate to
//! [Symphonia](https://github.com/pdeljanov/Symphonia) later after its Opus codec is complete, for better archotectural
//! design.

use anyhow::anyhow;
use std::process::Stdio;
use tokio::io::{AsyncRead, AsyncWrite};

#[derive(Debug, Clone, Copy)]
pub enum Container {
    Flac,
    Ogg,
}

#[derive(Debug, Clone, Copy)]
pub enum Codec {
    Flac,
    Opus,
}

#[derive(Debug, Clone, Copy)]
pub enum LossyQuality {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub title: String,
    pub album: String,
    pub author: String,
    pub cover: Vec<u8>,
}

#[derive(Debug, Clone)]
enum MetadataOps {
    Keep,
    Discard,
    Set(Metadata),
}

#[derive(Debug)]
pub struct Conversion<I> {
    input: I,
    out_container: Option<Container>,
    out_codec: Option<Codec>,
    out_quality: Option<LossyQuality>,
    out_metadata: MetadataOps,
}
impl<I: AsyncRead + Send + Unpin + 'static> Conversion<I> {
    pub fn new(input: I) -> Self {
        Self {
            input,
            out_container: None,
            out_codec: None,
            out_quality: None,
            out_metadata: MetadataOps::Keep,
        }
    }

    pub fn container(mut self, container: Container) -> Self {
        self.out_container = Some(container);
        self
    }

    pub fn codec(mut self, codec: Codec) -> Self {
        self.out_codec = Some(codec);
        self
    }

    pub fn discard_metadata(mut self) -> Self {
        self.out_metadata = MetadataOps::Discard;
        self
    }

    pub fn lossy_quality(mut self, quality: LossyQuality) -> Self {
        self.out_quality = Some(quality);
        self
    }

    pub async fn perform<O: AsyncWrite + Send + Unpin + 'static>(
        self,
        out: O,
    ) -> anyhow::Result<()> {
        convert_using_ffmpeg_cli(self, out).await
    }
}

async fn convert_using_ffmpeg_cli<
    I: AsyncRead + Send + Unpin + 'static,
    O: AsyncWrite + Send + Unpin + 'static,
>(
    mut conversion: Conversion<I>,
    mut out: O,
) -> anyhow::Result<()> {
    let mut command = tokio::process::Command::new("ffmpeg");

    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .arg("-loglevel")
        .arg("fatal")
        .arg("-i")
        .arg("pipe:");

    match conversion.out_codec {
        Some(Codec::Flac) => {
            command.arg("-acodec").arg("flac");
        }
        Some(Codec::Opus) => {
            command.arg("-acodec").arg("libopus");
        }
        None => {}
    }

    match conversion.out_container {
        Some(Container::Flac) => {
            command.arg("-f").arg("flac");
        }
        Some(Container::Ogg) => {
            command.arg("-f").arg("ogg");
        }
        None => {}
    }

    match conversion.out_metadata {
        MetadataOps::Keep => {}
        MetadataOps::Discard => {
            command.arg("-map_metadata").arg("-1").arg("-vn");
        }
        MetadataOps::Set(_) => {
            todo!();
        }
    }

    match conversion.out_quality {
        Some(LossyQuality::High) => {
            command.arg("-b:a").arg("320k");
        }
        Some(LossyQuality::Medium) => {
            command.arg("-b:a").arg("256k");
        }
        Some(LossyQuality::Low) => {
            command.arg("-b:a").arg("128k");
        }
        None => {}
    }

    command.arg("-");

    tracing::trace!("Running `ffmpeg` CLI with command: {command:?}");

    let mut child = command
        .spawn()
        .map_err(|err| anyhow!("failed to spawn ffmpeg process: {err}"))?;

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();

    tokio::spawn(async move { tokio::io::copy(&mut conversion.input, &mut stdin).await });
    tokio::spawn(async move { tokio::io::copy(&mut stdout, &mut out).await });
    let exit_status = child
        .wait()
        .await
        .map_err(|err| anyhow!("failed to probe ffmpeg process: {err}"))?;

    if exit_status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "conversion failed: ffmpeg exited with code {}",
            exit_status.code().unwrap_or(101)
        ))
    }
}
