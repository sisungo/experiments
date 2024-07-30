mod database;
mod fuse_server;
mod large_file;

use database::Fat32XDb;
use large_file::LargeFile;
use std::{
    io::{ErrorKind, Read, Write},
    path::PathBuf,
};

enum OpenedFile {
    Normal(std::fs::File),
    Large(LargeFile),
}
impl Read for OpenedFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Normal(f) => f.read(buf),
            Self::Large(f) => f.read(buf),
        }
    }
}
impl Write for OpenedFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Normal(f) => f.write(buf),
            Self::Large(f) => f.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Normal(f) => f.flush(),
            Self::Large(f) => f.flush(),
        }
    }
}

struct Fat32X {
    backend: PathBuf,
    large_files_path: PathBuf,
    db: Fat32XDb,
}
impl Fat32X {
    const MAX_SIZE: u64 = 4 * 1024 * 1024;

    fn new(backend: PathBuf) -> anyhow::Result<Self> {
        let internal_path = backend.join(".fat32x");
        let large_files_path = internal_path.join("lrgfiles");

        std::fs::create_dir_all(&internal_path)?;
        std::fs::create_dir_all(&large_files_path)?;
        std::fs::create_dir_all(internal_path.join("database"))?;

        let db = Fat32XDb::open(&backend)?;

        Ok(Self {
            backend,
            large_files_path,
            db,
        })
    }

    fn open(
        &self,
        path: &str,
        read: bool,
        write: bool,
        append: bool,
        create: bool,
    ) -> std::io::Result<OpenedFile> {
        if path.starts_with(".fat32x/") {
            return Err(ErrorKind::NotFound.into()); // protect internal directory .fat32x
        }
        let path_as_large_file = self.large_files_path.join(path);
        if path_as_large_file.exists() {
            Ok(OpenedFile::Large(LargeFile::new(
                path_as_large_file,
                read,
                write,
                append,
                Self::MAX_SIZE,
                create,
            )?))
        } else {
            Ok(OpenedFile::Normal(
                std::fs::File::options()
                    .read(read)
                    .write(write)
                    .create(create)
                    .append(append)
                    .open(self.backend.join(path))?,
            ))
        }
    }
}

fn main() {
    todo!()
}
