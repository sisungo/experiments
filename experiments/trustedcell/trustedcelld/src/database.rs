use crate::access::{AccessVector, Decision};
use rusqlite::params;
use std::path::Path;

pub struct AccessDb {
    conn: rusqlite::Connection,
}
impl AccessDb {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let conn = rusqlite::Connection::open(path)?;

        conn.execute(
            r#"CREATE TABLE IF NOT EXISTS access(
                    subject_uid INTEGER NOT NULL,
                    subject_cell TEXT NOT NULL,
                    object_category TEXT NOT NULL,
                    object_owner TEXT,
                    action TEXT NOT NULL,
                    allowed BOOLEAN NOT NULL,
            )"#,
            params![],
        )?;

        Ok(Self { conn })
    }

    pub fn decide(&self, access_vector: &AccessVector) -> rusqlite::Result<Option<Decision>> {
        let result = self.conn.query_row(
            r#"SELECT allowed FROM access WHERE
                subject_uid=?1 AND
                subject_cell=?2 AND
                object_category=?3 AND
                object_owner=?4 AND
                action=?5
            "#,
            params![
                access_vector.subject.uid,
                access_vector.subject.cell,
                access_vector.object.category,
                access_vector.object.owner,
                access_vector.action
            ],
            |row| {
                Ok(match row.get("allowed")? {
                    true => Decision::Allow,
                    false => Decision::Deny,
                })
            },
        );
        match result {
            Ok(x) => Ok(Some(x)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err),
        }
    }
}
