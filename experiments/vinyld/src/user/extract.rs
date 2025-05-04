use super::session::{AccessToken, Session};
use crate::{AppState, error::Error};
use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, HeaderValue, request},
};
use std::{convert::Infallible, sync::Arc};

/// Extracts the authorization token from the request headers.
pub struct Authorization<T>(pub T);
impl FromRequestParts<Arc<AppState>> for Authorization<Session> {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let access_token = access_token(&parts.headers)?;
        Ok(Self(state.users().get_session(&access_token).await?))
    }
}
impl FromRequestParts<Arc<AppState>> for Authorization<Option<Session>> {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let Ok(access_token) = access_token(&parts.headers) else {
            return Ok(Self(None));
        };
        let Ok(session) = state.users().get_session(&access_token).await else {
            return Ok(Self(None));
        };
        Ok(Self(Some(session)))
    }
}

fn access_token(headers: &HeaderMap<HeaderValue>) -> Result<AccessToken, Error> {
    Ok(AccessToken(
        headers
            .get("Authorization")
            .ok_or_else(|| Error::unauthorized())?
            .to_str()
            .map_err(|err| Error::bad_request(err))?
            .strip_prefix("Vinyl-Token ")
            .ok_or_else(|| Error::unauthorized())?
            .to_string(),
    ))
}
