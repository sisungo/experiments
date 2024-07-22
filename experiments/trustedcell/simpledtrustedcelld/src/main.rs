//! # simpletrustedcelld
//! Very simple TrustedCell daemon. For demonstration and testing usage.

use std::{collections::HashMap, fs::File, io::{Read, Write}};

fn main() -> std::io::Result<()> {
    let mut file = File::options()
        .read(true)
        .write(true)
        .open("/sys/kernel/security/trustedcell/host")?;
    let mut buffer = [0u8; 512];

    loop {
        file.read(&mut buffer)?;
        // TODO: Resolve message
        file.write(&buffer)?;
    }
}
