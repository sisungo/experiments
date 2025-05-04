mod expire_sessions;

use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub fn spawn(conn: Arc<DatabaseConnection>) {
    expire_sessions::spawn(conn.clone());
}
