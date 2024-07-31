mod enter;
mod status;
mod chcategory;
mod chowner;

use std::path::PathBuf;
use clap::Parser;

#[derive(Parser)]
enum Cmdline {
    Status,
    Enter(enter::Cmdline),
    Chcategory(chcategory::Cmdline),
    Chowner(chowner::Cmdline),
}

fn trustedcell_securityfs_path() -> PathBuf {
    match std::env::var("TC_SECFS_DIR") {
        Ok(x) => PathBuf::from(x),
        Err(_) => PathBuf::from("/sys/kernel/security/trustedcell"),
    }
}

fn main() {
    let cmdline = Cmdline::parse();

    let result = match cmdline {
        Cmdline::Status => status::main(),
        Cmdline::Enter(cmdline) => enter::main(cmdline),
        Cmdline::Chcategory(cmdline) => chcategory::main(cmdline),
        Cmdline::Chowner(cmdline) => chowner::main(cmdline),
    };

    if let Err(err) = result {
        eprintln!("tcutil: {err}");
    }
}
