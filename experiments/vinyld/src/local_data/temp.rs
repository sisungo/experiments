//! Temporary file storage.

use anyhow::anyhow;
use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::io::{AsyncRead, AsyncWrite};

const PATH: &str = "temp";

/// Temporary file storage.
#[derive(Debug)]
pub struct Temp(());
impl Temp {
    /// Returns a new temporary file storage.
    pub fn new() -> anyhow::Result<Self> {
        tracing::info!("Clearing the temporary file directory...");
        _ = std::fs::remove_dir_all(PATH);

        std::fs::create_dir_all(PATH)
            .map_err(|err| anyhow!("failed to create the temp directory: {err}"))?;

        Ok(Self(()))
    }

    /// Opens a new temporary file.
    pub async fn open(&self) -> std::io::Result<TempFile> {
        let mut options = tokio::fs::File::options();
        options.create_new(true).write(true);

        loop {
            let filename = vinutie::random::filename("temp", "tmp");
            let path = Path::new(PATH).join(filename);
            break match options.open(&path).await {
                Ok(_) => Ok(TempFile(Arc::new(TempFileInner { path }))),
                Err(err) if err.kind() == ErrorKind::AlreadyExists => continue,
                Err(x) => Err(x),
            };
        }
    }
}

/// A temporary file.
#[derive(Debug, Clone)]
pub struct TempFile(Arc<TempFileInner>);
impl TempFile {
    /// Returns a reader for the temporary file.
    pub async fn reader(&self) -> std::io::Result<TempFileReader> {
        let file = tokio::fs::File::open(&self.0.path).await?;
        Ok(TempFileReader {
            file,
            _guard: self.0.clone(),
        })
    }

    /// Returns a writer for the temporary file, in append mode.
    pub async fn appender(&self) -> std::io::Result<TempFileWriter> {
        let file = tokio::fs::File::options()
            .append(true)
            .open(&self.0.path)
            .await?;
        Ok(TempFileWriter {
            file,
            _guard: self.0.clone(),
        })
    }
}

/// A reader for a temporary file.
#[derive(Debug)]
pub struct TempFileReader {
    file: tokio::fs::File,
    _guard: Arc<TempFileInner>,
}
impl AsyncRead for TempFileReader {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        unsafe { std::pin::Pin::map_unchecked_mut(self, |x| &mut x.file).poll_read(cx, buf) }
    }
}

/// A writer for a temporary file.
#[derive(Debug)]
pub struct TempFileWriter {
    file: tokio::fs::File,
    _guard: Arc<TempFileInner>,
}
impl AsyncWrite for TempFileWriter {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        unsafe { std::pin::Pin::map_unchecked_mut(self, |x| &mut x.file).poll_write(cx, buf) }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        unsafe { std::pin::Pin::map_unchecked_mut(self, |x| &mut x.file).poll_flush(cx) }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        unsafe { std::pin::Pin::map_unchecked_mut(self, |x| &mut x.file).poll_shutdown(cx) }
    }
}

/// Inner representation of a temporary file.
#[derive(Debug)]
struct TempFileInner {
    path: PathBuf,
}
impl Drop for TempFileInner {
    fn drop(&mut self) {
        _ = std::fs::remove_file(&self.path);
    }
}
