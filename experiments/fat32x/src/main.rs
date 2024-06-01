mod database;
mod fuse_server;
mod large_file;

use database::Fat32XDb;
use std::path::PathBuf;

struct Fat32X {
    backend: PathBuf,
    db: Fat32XDb,
}
impl Fat32X {}

fn main() {
    todo!()
}
