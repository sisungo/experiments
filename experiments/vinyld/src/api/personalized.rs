use crate::{
    AppState,
    personalized::Homepage,
    user::{extract::Authorization, session::Session},
};
use axum::{Json, Router, extract::State, routing::get};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/homepage", get(homepage))
        .route("/commit", post(commit))
}

async fn homepage(
    State(state): State<Arc<AppState>>,
    Authorization(session): Authorization<Option<Session>>,
) -> Json<Homepage> {
    state
        .personalized()
        .homepage(session.map(|x| x.user.uid))
        .await
        .into()
}

async fn commit(
    State(state): State<Arc<AppState>>,
    Authorization(session): Authorization<Session>,
) {
}
