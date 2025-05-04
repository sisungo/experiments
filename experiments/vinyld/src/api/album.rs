use crate::{
    AppState,
    album::{AlbumId, Create, Profile},
    error::Error,
    user::{extract::Authorization, session::Session},
};
use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, State},
    response::Redirect,
    routing::{get, post},
};
use std::{sync::Arc, time::Duration};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create", post(create))
        .route(
            "/{id}/cover/hq.avif",
            get(cover_hq)
                .put(set_cover)
                .layer(DefaultBodyLimit::max(512 * 1024)),
        )
        .route("/{id}/profile.json", get(profile))
}

async fn create(
    State(state): State<Arc<AppState>>,
    Authorization(session): Authorization<Session>,
    Json(create): Json<Create>,
) -> Result<Json<AlbumId>, Error> {
    session.will_create_album()?;
    state
        .albums()
        .create(session.user.uid, create.into())
        .await
        .map(Json)
}

async fn cover_hq(
    State(state): State<Arc<AppState>>,
    Path(album_id): Path<AlbumId>,
) -> Result<Redirect, Error> {
    state
        .objects
        .get_url(
            state.albums().cover_hq(album_id).await?,
            Duration::from_secs(10 * 60),
        )
        .await
        .map(|url| Redirect::to(url.as_str()))
        .map_err(Into::into)
}

async fn set_cover(
    State(state): State<Arc<AppState>>,
    Authorization(session): Authorization<Session>,
    Path(album_id): Path<AlbumId>,
    image: Bytes,
) -> Result<(), Error> {
    state.albums().set_cover(album_id, image).await
}

async fn profile(
    State(state): State<Arc<AppState>>,
    Path(album_id): Path<AlbumId>,
) -> Result<Json<Profile>, Error> {
    state.albums().profile(album_id).await.map(Json)
}
