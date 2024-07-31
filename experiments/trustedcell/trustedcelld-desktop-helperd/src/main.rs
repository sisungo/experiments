mod daemon_gate;
mod dialog;
mod translations;

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
    CreateFile,
    Unlink,
    Rmdir,
    Transform,
    Other(String),
}

fn main() {}
