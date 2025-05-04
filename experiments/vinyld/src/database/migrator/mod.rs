mod m20250216_000001_create_user_table;
mod m20250223_000001_create_user_auth_password_table;
mod m20250302_000001_create_album_table;
mod m20250302_000002_create_song_table;
mod m20250316_000001_create_session_table;
mod m20250318_000001_create_app_settings_table;
mod m20250406_000001_create_song_comment_table;
mod m20250501_000001_create_lyrics_table;

use async_trait::async_trait;
use sea_orm_migration::prelude::*;

pub struct Migrator;
#[async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250216_000001_create_user_table::Migration),
            Box::new(m20250223_000001_create_user_auth_password_table::Migration),
            Box::new(m20250302_000001_create_album_table::Migration),
            Box::new(m20250302_000002_create_song_table::Migration),
            Box::new(m20250316_000001_create_session_table::Migration),
            Box::new(m20250318_000001_create_app_settings_table::Migration),
            Box::new(m20250406_000001_create_song_comment_table::Migration),
            Box::new(m20250501_000001_create_lyrics_table::Migration),
        ]
    }
}
