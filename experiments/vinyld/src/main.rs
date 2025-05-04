mod album;
mod api;
mod app_settings;
mod database;
mod error;
mod local_data;
mod personalized;
mod policy;
mod setup_wizard;
mod song;
mod user;
mod util;

use anyhow::anyhow;
use axum::Router;
use clap::Parser;
use database::Database;
use local_data::{LocalData, dotenv::*};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

/// Application state.
#[derive(Debug)]
struct AppState {
    local_data: LocalData,
    database: Database,
    objects: Box<dyn vinioss::Objects>,
}
impl AppState {
    /// Creates a new application state.
    async fn new(cli: Cli) -> anyhow::Result<Arc<Self>> {
        if let Some(working_dir) = cli.workdir {
            if let Err(err) = std::env::set_current_dir(&working_dir) {
                return Err(anyhow!(
                    "failed to set working directory to \"{}\": {}",
                    working_dir.display(),
                    err
                ));
            }
        }

        if !Path::new(".setup_done").exists() {
            if cli.no_setup_wizard {
                return Err(anyhow!("the working directory is not set up"));
            } else {
                setup_wizard::setup_wizard()?;
            }
        }

        dotenvy::from_path("env").map_err(|err| {
            anyhow!("failed to read dotenv file `env` in the working directory: {err}")
        })?;
        check_required()?;

        let local_data = LocalData::new()?;

        let database = Database::connect(&fetch_env::<String>(DATABASE_URL)?).await?;
        if let Ok(true) = fetch_env::<bool>(DATABASE_DANGEROUS_FRESH_MIGRATIONS) {
            database.migrate_fresh().await?;
        } else {
            database.migrate_up().await?;
        }
        if fetch_env::<bool>(DATABASE_CRON_ENABLED)? {
            database.start_crond();
        }

        let objects = vinioss::connect(&fetch_env::<String>(OBJECT_STORAGE)?).await?;

        Ok(Arc::new(Self {
            local_data,
            database,
            objects,
        }))
    }
}

#[derive(Debug, Parser)]
#[command(name = "vinyld")]
struct Cli {
    /// Working directory of the service
    #[arg(short = 'w', long = "workdir")]
    workdir: Option<PathBuf>,

    /// Don't start the setup wizard
    #[arg(long = "no-setup-wizard")]
    no_setup_wizard: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let app_state = match AppState::new(cli).await {
        Ok(x) => x,
        Err(err) => {
            tracing::error!("{err}");
            std::process::exit(1);
        }
    };

    if let Err(err) = start_axum_app(app_state.clone()).await {
        tracing::error!("{err}");
        std::process::exit(1);
    }

    while tokio::signal::ctrl_c().await.is_err() {}
}

/// Starts the axum application.
async fn start_axum_app(state: Arc<AppState>) -> anyhow::Result<()> {
    let axum_app = Router::new()
        .nest("/api", api::router(state.clone()))
        .with_state(state);

    let listener = util::listener::listener().await?;

    tokio::spawn(async move {
        if let Err(err) = axum::serve(listener, axum_app).await {
            tracing::error!("failed to initialize application: {err}");
            std::process::exit(1);
        }
    });

    Ok(())
}
