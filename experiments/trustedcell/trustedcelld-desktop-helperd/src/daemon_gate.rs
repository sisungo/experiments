use std::{os::unix::net::UnixStream, path::Path};

pub struct DaemonGate {
    inner: UnixStream,
}
impl DaemonGate {
    pub fn connect(path: &Path) -> std::io::Result<Self> {
        Ok(Self {
            inner: UnixStream::connect(path)?,
        })
    }
}
