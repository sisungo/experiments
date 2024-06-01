use std::{
    fs::File,
    io::{ErrorKind, Read, Seek, Write},
    path::PathBuf,
};

pub struct LargeFile {
    underlying_dir: PathBuf,
    current_pos: u32,
    current_file: File,
    read_access: bool,
    write_access: bool,
    append_mode: bool,
    max_size: u64,
}
impl Read for LargeFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self.current_file.read(buf) {
            Ok(0) => {
                let next = self.current_pos + 1;
                let next_file = File::options()
                    .read(self.read_access)
                    .write(self.write_access)
                    .create(false)
                    .append(self.append_mode)
                    .open(self.underlying_dir.join(next.to_string()));
                match next_file {
                    Ok(file) => {
                        self.current_file = file;
                        self.current_pos += 1;
                        self.read(buf)
                    }
                    Err(err) => match err.kind() {
                        ErrorKind::NotFound => Ok(0),
                        _ => Err(err),
                    },
                }
            }
            Ok(n) => Ok(n),
            Err(err) => Err(err),
        }
    }
}
impl Write for LargeFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let current_pos = self.current_file.stream_position()?;
        if current_pos + buf.len() as u64 > self.max_size {
            self.current_file
                .write_all(&buf[..(self.max_size - current_pos) as usize])?;
            let next = self.current_pos + 1;
            let next_file = File::options()
                .read(self.read_access)
                .write(self.write_access)
                .create(true)
                .append(self.append_mode)
                .open(self.underlying_dir.join(next.to_string()));
            match next_file {
                Ok(file) => {
                    self.current_file = file;
                    self.current_pos += 1;
                    Ok(self.max_size as usize - current_pos as usize)
                }
                Err(err) => Err(err),
            }
        } else {
            match self.current_file.write(buf) {
                Ok(n) => Ok(n),
                Err(err) => Err(err),
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.current_file.flush()
    }
}
impl Seek for LargeFile {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        todo!()
    }
}

#[test]
#[ignore]
fn test_read() {
    let mut lf = LargeFile {
        underlying_dir: "./test".into(),
        current_pos: 0,
        current_file: File::open("./test/0").unwrap(),
        read_access: true,
        write_access: false,
        append_mode: false,
        max_size: 1024,
    };
    let mut buf = vec![];
    lf.read_to_end(&mut buf).unwrap();
    std::fs::write("./test/read.bin", &buf).unwrap();
}

#[test]
#[ignore]
fn test_write() {
    let mut lf = LargeFile {
        underlying_dir: "./test".into(),
        current_pos: 0,
        current_file: File::options()
            .write(true)
            .create(true)
            .open("./test/0")
            .unwrap(),
        read_access: true,
        write_access: true,
        append_mode: true,
        max_size: 65536,
    };
    lf.write_all(&std::fs::read("/bin/ls").unwrap()).unwrap();
}
