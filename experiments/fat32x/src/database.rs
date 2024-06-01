use rusqlite::OpenFlags;
use std::path::Path;

pub struct Fat32XDb {
    db: rusqlite::Connection,
}
impl Fat32XDb {
    pub fn open(fs_root: &Path) -> rusqlite::Result<Self> {
        let db = rusqlite::Connection::open_with_flags(
            fs_root.join(".fat32x/database/master.db"),
            OpenFlags::default() | OpenFlags::SQLITE_OPEN_CREATE,
        )?;

        db.execute(
            "CREATE TABLE IF NOT EXISTS unix_permissions(path TEXT PRIMARY KEY, owner INTEGER NOT NULL, group INTEGER NOT NULL, permbits INTEGER NOT NULL)",
            []
        )?;

        Ok(Self { db })
    }
}
