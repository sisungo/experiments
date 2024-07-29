use std::path::Path;
use rusqlite::params;

pub struct AccessDb {
    conn: rusqlite::Connection,
}
impl AccessDb {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let conn = rusqlite::Connection::open(path)?;

        conn.execute(
            r#"CREATE TABLE IF NOT EXISTS acl(
                    subject_uid INTEGER NOT NULL,
                    subject_cell TEXT NOT NULL,
                    object_category TEXT NOT NULL,
                    object_owner TEXT,
                    action TEXT NOT NULL,
                    allowed BOOLEAN NOT NULL,
            )"#,
            params![]
        )?;
        
        Ok(Self {
            conn,
        })
    }
}
