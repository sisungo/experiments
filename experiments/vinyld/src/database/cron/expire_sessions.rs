use crate::database::entity::session;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::{sync::Arc, time::Duration};
use tokio::time::{MissedTickBehavior, interval};

pub fn spawn(conn: Arc<DatabaseConnection>) {
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(24 * 60 * 60 * 60));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            interval.tick().await;

            tracing::info!("Running database cron task `expire_sessions`...");
            _ = session::Entity::delete_many()
                .filter(session::Column::RefreshExpiry.lt(chrono::Utc::now().naive_utc()))
                .exec(&*conn)
                .await;
        }
    });
}
