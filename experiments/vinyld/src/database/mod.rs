mod cron;
pub mod entity;
mod migrator;

use anyhow::anyhow;
use migrator::Migrator;
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigratorTrait;
use std::sync::Arc;

#[derive(Debug)]
pub struct Database {
    pub conn: Arc<DatabaseConnection>,
}
impl Database {
    /// Connects to a database via URL.
    pub async fn connect(url: &str) -> anyhow::Result<Self> {
        let conn = Arc::new(
            sea_orm::Database::connect(url)
                .await
                .map_err(|err| anyhow!("failed to connect to database \"{}\": {}", url, err))?,
        );

        Ok(Self { conn })
    }

    /// Runs all migrations on this database.
    pub async fn migrate_up(&self) -> anyhow::Result<()> {
        Migrator::up(&*self.conn, None)
            .await
            .map_err(|err| anyhow!("database migration failed: {err}"))
    }

    /// Refreshes all migrations on this database.
    pub async fn migrate_fresh(&self) -> anyhow::Result<()> {
        Migrator::fresh(&*self.conn)
            .await
            .map_err(|err| anyhow!("database migration failed: {err}"))
    }

    /// Starts the task scheduler on this database.
    pub fn start_crond(&self) {
        cron::spawn(self.conn.clone());
    }
}
