mod dialog;
mod translations;

use anyhow::anyhow;
use std::{
    convert::Infallible,
    io::{BufRead, BufReader},
    os::unix::net::UnixStream,
    path::PathBuf,
    str::FromStr,
};
use translations::I18NToString;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Decision {
    Allow,
    AllowOnce,
    Deny,
    DenyOnce,
}
impl Decision {
    fn serialize_into(self, buf: &mut dyn std::io::Write) -> std::io::Result<()> {
        match self {
            Self::Allow => buf.write_all(b"1 1\n"),
            Self::AllowOnce => buf.write_all(b"1 0\n"),
            Self::Deny => buf.write_all(b"0 1\n"),
            Self::DenyOnce => buf.write_all(b"0 0\n"),
        }
    }
}

pub struct AccessVector {
    subject_cell: String,
    object: Object,
    action: Action,
}
impl I18NToString for AccessVector {
    fn i18n_to_string(&self, lang: &dyn translations::Translation) -> String {
        lang.translate_access_vector(self)
    }
}
impl FromStr for AccessVector {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splited = s.split(' ');
        let subject_cell = splited
            .next()
            .ok_or_else(|| anyhow!("incomplete access vector"))?;
        let object_category = splited
            .next()
            .ok_or_else(|| anyhow!("incomplete access vector"))?;
        let object_owner = splited
            .next()
            .ok_or_else(|| anyhow!("incomplete access vector"))?;
        let action = splited
            .next()
            .ok_or_else(|| anyhow!("incomplete access vector"))?;
        Ok(Self {
            subject_cell: subject_cell.into(),
            object: Object {
                category: object_category.into(),
                owner: object_owner.into(),
            },
            action: action.parse().unwrap(),
        })
    }
}

pub struct Object {
    category: String,
    owner: String,
}
impl Object {
    pub fn owner_mode(&self) -> bool {
        self.category.as_bytes()[0] == b'~'
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Action {
    OpenRo,
    OpenWo,
    OpenRw,
    Mkdir,
    Mknod,
    CreateReg,
    Unlink,
    Rmdir,
    Transform,
    Other(String),
}
impl FromStr for Action {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "posix.open_ro" => Self::OpenRo,
            "posix.open_wo" => Self::OpenWo,
            "posix.open_rw" => Self::OpenRw,
            "posix.mkdir" => Self::Mkdir,
            "posix.mknod" => Self::Mknod,
            "posix.create_reg" => Self::CreateReg,
            "posix.unlink" => Self::Unlink,
            "posix.rmdir" => Self::Rmdir,
            "trustedcell.change_cell" => Self::Transform,
            x => Self::Other(x.into()),
        })
    }
}

fn main() -> anyhow::Result<()> {
    let sock_path = std::env::var("TRUSTEDCELLD_SOCK")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/var/run/trustedcelld/helper_hub.sock"));
    let mut sock = BufReader::new(UnixStream::connect(&sock_path)?);
    let mut buf = String::new();
    loop {
        buf.clear();
        sock.read_line(&mut buf)?;
        let av = AccessVector::from_str(&buf)?;
        let decision = dialog::ask_for_permission(&av)?;
        decision.serialize_into(sock.get_mut())?;
    }
}
