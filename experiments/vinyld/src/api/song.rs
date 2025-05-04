use crate::{
    AppState,
    error::Error,
    song::{AudioQuality, Profile, SongId, Upload},
    user::{extract::Authorization, session::Session},
};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    response::Redirect,
    routing::{get, post},
};
use std::{sync::Arc, time::Duration};
use tokio::io::AsyncWriteExt;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/upload",
            post(upload).layer(DefaultBodyLimit::max(256 * 1024 * 1024)),
        )
        .route("/{id}/audio/{quality}", get(audio))
        .route("/{id}/audio/profile.json", get(profile))
}

async fn upload(
    State(app_state): State<Arc<AppState>>,
    Authorization(session): Authorization<Session>,
    mut multipart: Multipart,
) -> Result<Json<SongId>, Error> {
    session.will_upload_song()?;

    let temp = app_state
        .local_data
        .temp
        .open()
        .await
        .map_err(|err| Error::internal(err))?;
    let mut info = None;
    let mut audio_written = false;

    while let Some(mut field) = multipart.next_field().await? {
        if field.name() == Some("info") {
            info = Some(
                serde_json::from_slice::<Upload>(&*field.bytes().await?)
                    .map_err(|err| Error::bad_request(err))?,
            );
        } else if field.name() == Some("audio") {
            let mut file = temp.appender().await.map_err(Error::internal)?;
            while let Some(chunk) = field.chunk().await? {
                file.write_all(&*chunk).await.map_err(Error::internal)?;
            }
            audio_written = true;
        }
    }

    if !audio_written {
        return Err(Error::bad_request("Form entry `audio` was not written."));
    }

    if let Some(info) = info {
        let song_id = app_state
            .songs()
            .upload(session.user.uid, info, temp)
            .await?;
        Ok(Json(song_id))
    } else {
        Err(Error::bad_request("Form entry `info` was not written."))
    }
}

async fn audio(
    State(app_state): State<Arc<AppState>>,
    Authorization(session): Authorization<Option<Session>>,
    Path((song_id, quality)): Path<(SongId, AudioQuality)>,
) -> Result<Redirect, Error> {
    let object_key = app_state
        .songs()
        .audio(song_id)
        .user(session.map(|x| x.user).as_ref())
        .quality(quality)
        .invoke()
        .await?;

    Ok(Redirect::to(
        app_state
            .objects
            .get_url(object_key, Duration::from_secs(10 * 60))
            .await?
            .as_str(),
    ))
}

async fn profile(
    State(app_state): State<Arc<AppState>>,
    Path(song_id): Path<SongId>,
) -> Result<Json<Profile>, Error> {
    app_state.songs().profile(song_id).await.map(Json)
}
