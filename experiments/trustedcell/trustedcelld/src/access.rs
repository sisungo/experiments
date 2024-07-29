#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Subject {
    pub uid: libc::uid_t,
    pub cell: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Object {
    pub category: String,
    pub owner: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AccessVector {
    pub subject: Subject,
    pub object: Object,
    pub action: String,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Decision {
    Allow,
    AllowOnce,
    Deny,
    DenyOnce,
}
