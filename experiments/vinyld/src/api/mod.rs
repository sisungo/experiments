mod admin;
mod album;
mod personalized;
mod song;
mod user;

use crate::{
    AppState,
    app_settings::{LicenseHTML, SiteName},
};
use axum::{Json, Router, extract::State, response::Html, routing::get};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Indicates the version of the implemented protocol.
///
/// **NOTE**: This is NOT version of the server software, [`vinyld`].
pub const VERSION: &str = "9999";

/// Root routes.
pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .nest("/user", user::router())
        .nest("/admin", admin::router(state.clone()))
        .nest("/song", song::router())
        .nest("/album", album::router())
        .nest("/personalized", personalized::router())
        .route("/version.txt", get(|| async { VERSION }))
        .route("/site_info.json", get(site_info))
        .route("/license.html", get(license))
}

/// Information of the site server.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SiteInfo {
    /// Name of the site.
    name: String,

    /// Name of the software.
    software: String,
}

async fn site_info(State(state): State<Arc<AppState>>) -> Json<SiteInfo> {
    Json(SiteInfo {
        name: state.app_settings().get::<SiteName>().await,
        software: concat!("Vinyld v", env!("CARGO_PKG_VERSION")).into(),
    })
}

async fn license(State(state): State<Arc<AppState>>) -> Html<String> {
    Html(state.app_settings().get::<LicenseHTML>().await)
}
