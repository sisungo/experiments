use std::path::PathBuf;
use anyhow::anyhow;
use clap::Parser;

#[derive(Parser)]
pub struct Cmdline {
    #[arg(short, long)]
    recursive: bool,

    path: PathBuf,
}

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    todo!()
}