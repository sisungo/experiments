use anyhow::anyhow;
use smallvec::SmallVec;
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};
use tokio::sync::mpsc;

pub struct HostReader {
    rx: mpsc::Receiver<Request>,
}
impl HostReader {
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let (tx, rx) = mpsc::channel(16);
        std::thread::spawn(move || HostReaderImpl { file, tx }.run());
        Ok(Self { rx })
    }

    pub async fn recv(&mut self) -> anyhow::Result<Request> {
        self.rx
            .recv()
            .await
            .ok_or_else(|| anyhow!("background thread died"))
    }
}

#[derive(Clone)]
pub struct HostWriter {
    tx: mpsc::Sender<WriteCommand>,
}
impl HostWriter {
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let (tx, rx) = mpsc::channel(16);
        std::thread::spawn(move || HostWriterImpl { file, rx }.run());
        Ok(Self { tx })
    }

    pub async fn send_response(&self, response: Response) -> anyhow::Result<()> {
        self.tx.send(WriteCommand::Response(response)).await?;
        Ok(())
    }
}

struct HostReaderImpl {
    file: File,
    tx: mpsc::Sender<Request>,
}
impl HostReaderImpl {
    fn run(mut self) -> anyhow::Result<()> {
        let mut buf = [0u8; 512];
        loop {
            let n = self.file.read(&mut buf)?;
            let req = Request::deserialize_from(&buf[..n])?;
            self.tx.blocking_send(req)?;
        }
    }
}

struct HostWriterImpl {
    file: File,
    rx: mpsc::Receiver<WriteCommand>,
}
impl HostWriterImpl {
    fn run(mut self) {
        while let Some(cmd) = self.rx.blocking_recv() {
            match cmd {
                WriteCommand::Response(resp) => self.write_response(resp),
            }
        }
    }

    fn write_response(&mut self, resp: Response) {
        let mut buf: SmallVec<[u8; 80]> = SmallVec::new();
        if resp.serialize_to(&mut buf).is_err() {
            unreachable!();
        }
        let _ = self.file.write(&buf);
    }
}

pub struct Request {
    request_id: i64,
    subject_uid: libc::uid_t,
    subject_cell: String,
    object_category: String,
    object_owner: String,
}
impl Request {
    fn deserialize_from(buf: &[u8]) -> anyhow::Result<Self> {
        let buf = String::from_utf8_lossy(buf).as_ref();
        todo!()
    }
}

pub struct Response {
    request_id: i64,
    allowed: bool,
    cachable: bool,
}
impl Response {
    fn serialize_to<const N: usize>(&self, buf: &mut SmallVec<[u8; N]>) -> std::io::Result<()> {
        write!(
            buf,
            "{} {} {}",
            self.request_id, self.allowed as i32, self.cachable as i32
        )
    }
}

enum WriteCommand {
    Response(Response),
}
