mod access;
mod access_conductor;
mod database;
mod helper;
mod host_gate;
mod rule;

use access_conductor::AccessConductor;
use clap::Parser;
use database::AccessDb;
use helper::HelperHub;
use host_gate::{HostReader, HostWriter, Request, Response};
use rule::Ruleset;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

struct Context {
    cmdline: Cmdline,
    host_reader: Mutex<HostReader>,
    host_writer: HostWriter,
    access_conductor: AccessConductor,
}
impl Context {
    async fn run(self: Arc<Self>) -> anyhow::Result<()> {
        let mut host_reader = self.host_reader.lock().await;

        loop {
            let request = host_reader.recv().await?;
        }
    }
}

#[derive(Parser)]
struct Cmdline {
    #[arg(short, long, default_value = "/sys/kernel/security/trustedcell/host")]
    host_path: PathBuf,

    #[arg(short, long, default_value = "/var/lib/trustedcelld")]
    data_dir: PathBuf,

    #[arg(short, long, default_value = "/var/run/trustedcelld")]
    runtime_dir: PathBuf,

    #[arg(short, long, default_value = "/usr/share/trustedcelld")]
    resource_dir: PathBuf,
}
impl Cmdline {
    fn db_path(&self) -> PathBuf {
        self.data_dir.join("main.db")
    }

    fn ruleset_path(&self) -> PathBuf {
        self.resource_dir.join("builtin-rules.sb")
    }

    fn helper_hub_sock_path(&self) -> PathBuf {
        self.runtime_dir.join("helper_hub.db")
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cmdline = Cmdline::parse();

    let host_reader = Mutex::new(HostReader::open(&cmdline.host_path)?);
    let host_writer = HostWriter::open(&cmdline.host_path)?;

    let access_db = AccessDb::open(&cmdline.db_path())?;
    let ruleset = Ruleset::compile_file(&cmdline.ruleset_path())?;
    let helper_hub = HelperHub::listen(&cmdline.helper_hub_sock_path())?;

    Arc::new(Context {
        cmdline,
        host_reader,
        host_writer,
        access_conductor: AccessConductor::new(access_db, ruleset, helper_hub),
    })
    .run()
    .await
}
