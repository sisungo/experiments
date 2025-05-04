mod format_convert;

use crate::{
    AppState,
    album::AlbumId,
    database::entity::song,
    error::{Error, ErrorCode},
    local_data::temp::TempFile,
    policy::{ObjectPolicy, Subject},
    user::{Uid, User},
};
use sea_orm::{ActiveValue::NotSet, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use vinioss::ObjectKey;

/// Manager of the song set.
pub struct Songs<'a>(&'a AppState);
impl Songs<'_> {
    /// Uploads a song.
    pub async fn upload(
        &self,
        uploader: Uid,
        upload: Upload,
        unprocessed: TempFile,
    ) -> Result<SongId, Error> {
        if upload.quality == AudioQuality::Origin {
            return Err(Error::bad_request(
                "audio quality `origin` cannot be used in uploading",
            ));
        }

        let uploader = self.0.users().find_by_uid(uploader).await?;
        let album_write_policy =
            self.0
                .albums()
                .write_policy(upload.album)
                .await
                .map_err(|err| match err.code {
                    ErrorCode::NOT_FOUND => Error::non_existent_album(),
                    _ => err,
                })?;
        if album_write_policy.denies(&Subject::from_user(&uploader)) {
            return Err(Error::denied_by_policy(&album_write_policy.class));
        }

        let recompressed = self
            .0
            .local_data
            .temp
            .open()
            .await
            .map_err(|err| Error::internal(err))?;
        format_convert::Conversion::new(unprocessed.reader().await.map_err(Error::internal)?)
            .codec(upload.quality.codec())
            .lossy_quality(upload.quality.lossy_quality())
            .discard_metadata()
            .container(upload.quality.container())
            .perform(recompressed.appender().await.map_err(Error::internal)?)
            .await
            .map_err(|err| Error::bad_request(format!("failed to recompress audio: {err}")))?;
        drop(unprocessed);

        let object_id = ObjectKey(vinutie::random::filename(
            "song_audio",
            upload.quality.filename_extension(),
        ));

        let song = song::Entity::insert(song::ActiveModel {
            song_id: NotSet,
            title: Set(upload.title),
            album: Set(upload.album.0),
            uploader: Set(uploader.uid.0),
            origin_audio: Set(object_id.0.clone()),
            listen_policy: Set(None),
            created_at: Set(chrono::Utc::now().naive_utc()),
        })
        .exec(&*self.0.database.conn)
        .await
        .map_err(|err| match err.sql_err() {
            Some(sea_orm::SqlErr::ForeignKeyConstraintViolation(_)) => Error::non_existent_album(),
            _ => Error::internal(err),
        })?;

        self.0
            .objects
            .put_stream(
                object_id.clone(),
                &mut recompressed.reader().await.map_err(Error::internal)?,
            )
            .await?;

        Ok(SongId(song.last_insert_id))
    }

    /// Gets audio file of given song ID and audio quality.
    pub fn audio(&self, song_id: SongId) -> AudioCall {
        AudioCall {
            state: self.0,
            user: None,
            song_id,
            quality: AudioQuality::Origin,
        }
    }

    /// Returns the profile of the song.
    pub async fn profile(&self, song_id: SongId) -> Result<Profile, Error> {
        let mut listen_policy_class = None;

        let model = song::Entity::find_by_id(song_id.0)
            .one(&*self.0.database.conn)
            .await?
            .ok_or(Error::not_found())?;

        let origin_quality = model.origin_quality()?;
        if let Some(listen_policy) = model.listen_policy {
            let listen_policy: ObjectPolicy =
                serde_json::from_value(listen_policy).map_err(Error::internal)?;
            listen_policy_class = Some(listen_policy.class);
        }

        Ok(Profile {
            song_id: SongId(model.song_id),
            title: model.title.clone(),
            translated_title: model.title,
            uploader: Uid(model.uploader),
            album: AlbumId(model.album),
            listen_policy_class,
            origin_quality,
        })
    }
}

#[derive(Debug)]
pub struct AudioCall<'a, 'b> {
    state: &'a AppState,
    user: Option<&'b User>,
    song_id: SongId,
    quality: AudioQuality,
}
impl<'b> AudioCall<'_, 'b> {
    pub fn user(mut self, user: Option<&'b User>) -> Self {
        self.user = user;
        self
    }

    pub fn quality(mut self, quality: AudioQuality) -> Self {
        self.quality = quality;
        self
    }

    pub async fn invoke(self) -> Result<ObjectKey, Error> {
        let model = song::Entity::find_by_id(self.song_id.0)
            .one(&*self.state.database.conn)
            .await?
            .ok_or(Error::not_found())?;

        let origin_quality = model.origin_quality()?;

        if let Some(listen_policy) = model.listen_policy {
            let listen_policy: ObjectPolicy =
                serde_json::from_value(listen_policy).map_err(Error::internal)?;
            let subject = self
                .user
                .map(Subject::from_user)
                .unwrap_or_else(|| Subject::anon());
            if listen_policy.denies(&subject) {
                return Err(Error::denied_by_policy(&listen_policy.class));
            }
        }

        if self.quality == AudioQuality::Origin || self.quality == origin_quality {
            Ok(ObjectKey(model.origin_audio))
        } else if self.quality > origin_quality {
            Err(Error::not_found())
        } else {
            todo!()
        }
    }
}

impl song::Model {
    /// Returns the quality of the original audio file.
    fn origin_quality(&self) -> Result<AudioQuality, Error> {
        AudioQuality::from_filename(&self.origin_audio)
    }
}

/// Request of uploading a song.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upload {
    /// Title of the song, in the native language to the song.
    pub title: String,

    /// Album that the song belongs to.
    pub album: AlbumId,

    /// Expected quality of the original audio file.
    ///
    /// Note that you cannot fill [`AudioQuality::Origin`] here, since it's an ambigious quality.
    pub quality: AudioQuality,
}

/// Profile of a song.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// ID of the song.
    pub song_id: SongId,

    /// Title of the song, in its original language.
    pub title: String,

    /// Title of the song, in the requesting user's language.
    ///
    /// If the user didn't specify a language when requesting, or the user's language is same as the song's original language, this field
    /// will contain the same value as `title`.
    pub translated_title: String,

    /// Uploader of this song.
    pub uploader: Uid,

    /// Album that this song belongs to.
    pub album: AlbumId,

    /// Listen policy class of this song.
    pub listen_policy_class: Option<String>,

    /// Quality of the "origin" quality.
    pub origin_quality: AudioQuality,
}

/// Representation of a Song ID.
///
/// A Song ID identifies a song uniquely and cannot be changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct SongId(pub i64);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum AudioQuality {
    /// Low-quality lossy audio.
    Low,

    /// Medium-quality lossy audio.
    Medium,

    /// High-quality lossy audio.
    High,

    /// Lossless audio quality.
    Lossless,

    /// Quality of the originally uploaded audio file. This is the highest quality, and may be an alias
    /// of another quality.
    Origin,
}
impl AudioQuality {
    pub fn lossy_quality(&self) -> format_convert::LossyQuality {
        match self {
            Self::Low => format_convert::LossyQuality::Low,
            Self::Medium => format_convert::LossyQuality::Medium,
            Self::High => format_convert::LossyQuality::High,
            _ => format_convert::LossyQuality::High,
        }
    }

    pub fn container(&self) -> format_convert::Container {
        match self {
            Self::Lossless => format_convert::Container::Flac,
            _ => format_convert::Container::Ogg,
        }
    }

    pub fn codec(&self) -> format_convert::Codec {
        match self {
            Self::Lossless => format_convert::Codec::Flac,
            _ => format_convert::Codec::Opus,
        }
    }

    pub fn filename_extension(&self) -> &'static str {
        match self {
            Self::Origin => panic!("invalid use of file_extension"),
            Self::Lossless => "flac",
            Self::High => "high.opus",
            Self::Medium => "medium.opus",
            Self::Low => "low.opus",
        }
    }

    pub fn from_filename(name: &str) -> Result<Self, Error> {
        if name.ends_with(".flac") {
            Ok(Self::Lossless)
        } else if name.ends_with(".high.opus") {
            Ok(Self::High)
        } else if name.ends_with(".medium.opus") {
            Ok(Self::Medium)
        } else if name.ends_with(".low.opus") {
            Ok(Self::Low)
        } else {
            Err(Error::internal("unrecognized audio file"))
        }
    }
}

impl AppState {
    /// Returns a manager to the song set.
    pub fn songs(&self) -> Songs {
        Songs(self)
    }
}
