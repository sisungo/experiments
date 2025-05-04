use crate::{
    AppState,
    database::entity::album,
    error::Error,
    policy::{Condition::MatchUid, ObjectPolicy, PolicyItem},
    user::Uid,
};
use axum::body::Bytes;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, EntityTrait, IntoActiveModel, Set};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use vinioss::ObjectKey;

/// Manager of the album set.
pub struct Albums<'a>(&'a AppState);
impl Albums<'_> {
    /// Resolution of album covers in the HQ quality.
    const COVER_HQ_RESOLUTION: (u32, u32) = (640, 640);

    /// Creates a new album.
    pub async fn create(&self, uploader: Uid, create: Create) -> Result<AlbumId, Error> {
        Ok(AlbumId(
            album::Entity::insert(album::ActiveModel {
                album_id: NotSet,
                title: Set(create.title),
                uploader: Set(uploader.0),
                description: Set(create.description),
                created_at: Set(Utc::now().naive_utc()),
                write_policy: Set(write_policy(uploader).json()),
                cover: Set(None),
            })
            .exec(&*self.0.database.conn)
            .await?
            .last_insert_id,
        ))
    }

    /// Returns the HQ cover of the album.
    pub async fn cover_hq(&self, album_id: AlbumId) -> Result<ObjectKey, Error> {
        album::Entity::find_by_id(album_id.0)
            .one(&*self.0.database.conn)
            .await?
            .ok_or_else(|| Error::not_found())?
            .cover
            .ok_or_else(|| Error::not_found())
            .map(ObjectKey)
    }

    /// Sets the HQ cover of the album.
    pub async fn set_cover(&self, album_id: AlbumId, image: Bytes) -> Result<(), Error> {
        let model = album::Entity::find_by_id(album_id.0)
            .one(&*self.0.database.conn)
            .await?
            .ok_or_else(|| Error::not_found())?;

        let mut buffer = Cursor::new(crate::util::image::recompress(
            image,
            Self::COVER_HQ_RESOLUTION,
        )?);

        if let Some(origin) = &model.cover {
            self.0.objects.remove(ObjectKey(origin.clone())).await?;
        }

        let object_key = ObjectKey(vinutie::random::filename("album_cover", "avif"));
        self.0
            .objects
            .put_stream(object_key.clone(), &mut buffer)
            .await?;

        let mut active_model = model.into_active_model();
        active_model.cover = Set(Some(object_key.0));
        active_model.insert(&*self.0.database.conn).await?;

        Ok(())
    }

    /// Returns the profile of the album.
    pub async fn profile(&self, album_id: AlbumId) -> Result<Profile, Error> {
        let model = album::Entity::find_by_id(album_id.0)
            .one(&*self.0.database.conn)
            .await?
            .ok_or_else(|| Error::not_found())?;

        Ok(Profile {
            title: model.title.clone(),
            uploader: Uid(model.uploader),
            translated_title: model.title,
        })
    }

    /// Returns write policy of the album.
    pub async fn write_policy(&self, album_id: AlbumId) -> Result<ObjectPolicy, Error> {
        let model = album::Entity::find_by_id(album_id.0)
            .one(&*self.0.database.conn)
            .await?
            .ok_or_else(|| Error::not_found())?;

        serde_json::from_value(model.write_policy).map_err(Error::internal)
    }
}

/// Request of album creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Create {
    /// Title of the created album.
    pub title: String,

    /// Description of the created album.
    pub description: Option<String>,
}

/// Representation of a Album ID.
///
/// An Album ID identifies an album uniquely and cannot be changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct AlbumId(pub i64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Title of the album, in its original language.
    pub title: String,

    /// Uploader of this album.
    pub uploader: Uid,

    /// Title of the album, in the requesting user's language.
    ///
    /// If the user didn't specify a language when requesting, or the user's language is same as the album's original language, this field
    /// will contain the same value as `title`.
    pub translated_title: String,
}

fn write_policy(uploader: Uid) -> ObjectPolicy {
    ObjectPolicy {
        class: "AlbumPolicy".into(),
        items: vec![PolicyItem::Allow(MatchUid(uploader))],
    }
}

impl AppState {
    /// Returns a manager of the album set.
    pub fn albums(&self) -> Albums {
        Albums(self)
    }
}
