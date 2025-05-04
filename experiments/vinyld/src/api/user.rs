use crate::{
    AppState,
    error::Error,
    user::{
        Profile, Uid,
        login::{LoginResponse, UserProber},
        register::RegisterRequest,
        session::{AccessToken, RefreshToken},
    },
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::Redirect,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login_via_passwd", post(login_via_passwd))
        .route("/refresh_token", post(refresh_token))
        .route("/register", post(register))
        .route("/{uid}/profile/profile.json", get(profile))
        .route("/{uid}/avatar/hq.avif", get(avatar_hq))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LoginViaPasswdRequest {
    #[serde(flatten)]
    prober: UserProber,
    password: String,

    source: String,
}

async fn login_via_passwd(
    State(app_state): State<Arc<AppState>>,
    Query(args): Query<LoginViaPasswdRequest>,
) -> Result<Json<LoginResponse>, Error> {
    app_state
        .users()
        .login_via_password(&args.prober, &args.password, args.source)
        .await
        .map(Json)
}

#[derive(Serialize, Deserialize)]
struct RefreshTokenRequest {
    refresh_token: RefreshToken,
}

#[derive(Serialize, Deserialize)]
struct RefreshTokenResponse {
    refresh_token: RefreshToken,
    access_token: AccessToken,
}

async fn refresh_token(
    State(app_state): State<Arc<AppState>>,
    Query(refresh_token): Query<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>, Error> {
    let (refresh_token, access_token) = app_state
        .users()
        .refresh_token(&refresh_token.refresh_token)
        .await?;

    Ok(Json(RefreshTokenResponse {
        refresh_token,
        access_token,
    }))
}

async fn register(
    State(app_state): State<Arc<AppState>>,
    Query(request): Query<RegisterRequest>,
) -> Result<Json<LoginResponse>, Error> {
    app_state.users().register(request).await.map(Json)
}

async fn profile(
    State(app_state): State<Arc<AppState>>,
    Path(uid): Path<Uid>,
) -> Result<Json<Profile>, Error> {
    app_state
        .users()
        .find_by_uid(uid)
        .await
        .map(|x| Json(Profile::from(x)))
}

async fn avatar_hq(
    State(state): State<Arc<AppState>>,
    Path(uid): Path<Uid>,
) -> Result<Redirect, Error> {
    Ok(Redirect::to(&state.users().avatar_hq(uid).await?.0))
}
