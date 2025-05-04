use crate::{
    AppState,
    app_settings::{self, *},
    error::Error,
    user::{extract::Authorization, session::Session},
};
use axum::{
    Json, Router,
    extract::{Request, State},
    middleware::Next,
    response::Response,
    routing::get,
};
use std::sync::Arc;

/// Macro for creating a route for an app setting.
macro_rules! route_entry {
    ($t:ty) => {
        get(async |State(state): State<Arc<AppState>>| Json(state.app_settings().get::<$t>().await))
            .put(
                async |State(state): State<Arc<AppState>>,
                       data: Json<<$t as app_settings::Entry>::Ty>| {
                    state.app_settings().set::<$t>(&*data).await
                },
            )
    };
}

/// Middleware for checking if the user has admin permissions.
async fn requires_admin(
    Authorization(session): Authorization<Session>,
    request: Request,
    next: Next,
) -> Result<Response, Error> {
    session.will_administrate()?;
    Ok(next.run(request).await)
}

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/mandatory_album_censorship",
            route_entry!(MandatoryAlbumCensorship),
        )
        .route(
            "/mandatory_song_censorship",
            route_entry!(MandatorySongCensorship),
        )
        .route(
            "/mandatory_comment_censorship",
            route_entry!(MandatoryCommentCensorship),
        )
        .route("/license_html", route_entry!(LicenseHTML))
        .layer(axum::middleware::from_fn_with_state(state, requires_admin))
}
