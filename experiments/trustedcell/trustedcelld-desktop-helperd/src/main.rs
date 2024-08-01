mod dialog;
mod translations;
mod daemon_gate;
mod res;

use daemon_gate::MessageProto;
use std::{
    convert::Infallible,
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
    ReadDir,
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
            "posix.read_dir" => Self::ReadDir,
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
    let mut sock = MessageProto::from(UnixStream::connect(&sock_path)?);
    loop {
        let buf = String::from_utf8(sock.recv()?)?;
        let av = daemon_gate::access_vector_of(&buf)?;
        let decision = dialog::ask_for_permission(&av)?;
        sock.send(daemon_gate::of_decision(decision))?;
    }
}
