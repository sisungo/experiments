use anyhow::anyhow;
use clap::Parser;

#[derive(Parser)]
pub struct Cmdline {
    cell: String,
}

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    std::fs::write(crate::trustedcell_securityfs_path().join("me"), &cmdline.cell)
        .map_err(|err| anyhow!("failed to enter cell `{}`: {}", &cmdline.cell, err))?;
    Ok(())
}