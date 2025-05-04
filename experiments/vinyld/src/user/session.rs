use super::{Ban, Uid, Users};
use crate::{
    database::entity::{session, user},
    error::Error,
};
use base64::{Engine, prelude::BASE64_URL_SAFE};
use bitflags::bitflags;
use chrono::Utc;
use rand::RngCore;
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
    Set,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

bitflags! {
    #[derive(Debug)]
    pub struct Permissions: u32 {
        /// Indicates that the session may be renewed.
        const REFRESH = 2 << 0;

        /// Indicates that the session may manage other sessions associated to this account.
        const MANAGE_SESSIONS = 2 << 1;

        /// Indicates that the session may do server administration.
        const ADMINISTRATION = 2 << 2;

        /// Indicates that the session may upload songs.
        const UPLOAD_SONG = 2 << 3;

        /// Indicates that the session may create albums.
        const CREATE_ALBUM = 2 << 4;
    }
}

impl Users<'_> {
    pub async fn get_session(&self, access_token: &AccessToken) -> Result<Session, Error> {
        let session_model = session::Entity::find()
            .filter(session::Column::AccessToken.eq(&access_token.0))
            .one(&*self.0.database.conn)
            .await?
            .ok_or_else(|| Error::unauthorized())?;
        let user_model = user::Entity::find_by_id(session_model.uid)
            .one(&*self.0.database.conn)
            .await?
            .ok_or_else(|| Error::unauthorized())?;

        if let Some(payload) = user_model.banned.clone() {
            let payload: Ban = serde_json::from_value(payload).map_err(Error::internal)?;
            if payload.end_time > Utc::now() {
                return Err(Error::banned_user(serde_json::to_value(payload).unwrap()));
            } else {
                let mut active_model = user_model.clone().into_active_model();
                active_model.banned = Set(None);
                active_model.update(&*self.0.database.conn).await?;
            }
        }

        Ok(Session {
            user: user_model.into(),
            permissions: Permissions::from_bits_retain(session_model.permissions as _),
        })
    }

    pub async fn put_session(
        &self,
        uid: Uid,
        permissions: Permissions,
        source: String,
    ) -> Result<(RefreshToken, AccessToken), Error> {
        let model = session::ActiveModel {
            numeral: NotSet,
            uid: Set(uid.0),
            refresh_token: Set(RefreshToken::generate(uid).0),
            access_token: Set(AccessToken::generate(uid).0),
            refresh_expiry: Set(Utc::now().naive_utc() + RefreshToken::VALID_IN),
            access_expiry: Set(Utc::now().naive_utc() + AccessToken::VALID_IN),
            permissions: Set(permissions.bits() as _),
            source: Set(source),
        };

        let model = model.insert(&*self.0.database.conn).await?;

        Ok((
            RefreshToken(model.refresh_token),
            AccessToken(model.access_token),
        ))
    }

    pub async fn refresh_token(
        &self,
        refresh_token: &RefreshToken,
    ) -> Result<(RefreshToken, AccessToken), Error> {
        let record = session::Entity::find()
            .filter(session::Column::RefreshToken.eq(&refresh_token.0))
            .one(&*self.0.database.conn)
            .await?
            .ok_or_else(|| Error::unauthorized())?;
        if record.refresh_expiry.and_utc() < Utc::now() {
            return Err(Error::unauthorized());
        }
        if !Permissions::from_bits_retain(record.permissions as _).contains(Permissions::REFRESH) {
            return Err(Error::restricted_session());
        }
        let uid = record.uid;

        let mut record: session::ActiveModel = record.into();
        record.refresh_token = Set(RefreshToken::generate(Uid(uid)).0);
        record.refresh_expiry = Set(Utc::now().naive_utc() + RefreshToken::VALID_IN);
        record.access_token = Set(AccessToken::generate(Uid(uid)).0);
        record.access_expiry = Set(Utc::now().naive_utc() + AccessToken::VALID_IN);
        let record = record.update(&*self.0.database.conn).await.unwrap();

        Ok((
            RefreshToken(record.refresh_token),
            AccessToken(record.access_token),
        ))
    }
}

/// A session.
#[derive(Debug)]
pub struct Session {
    pub user: super::User,
    pub permissions: Permissions,
}
impl Session {
    /// Succeeds if the session may do administration work.
    pub fn will_administrate(&self) -> Result<(), Error> {
        if !self.permissions.contains(Permissions::ADMINISTRATION) {
            Err(Error::restricted_session())
        } else if !self.user.groups.iter().any(|x| x == super::group::WHEEL) {
            Err(Error::restricted_user())
        } else {
            Ok(())
        }
    }

    /// Succeeds if the session may upload songs.
    pub fn will_upload_song(&self) -> Result<(), Error> {
        if !self.permissions.contains(Permissions::UPLOAD_SONG) {
            Err(Error::restricted_session())
        } else {
            Ok(())
        }
    }

    /// Succeeds if the session may create albums.
    pub fn will_create_album(&self) -> Result<(), Error> {
        if !self.permissions.contains(Permissions::CREATE_ALBUM) {
            Err(Error::restricted_session())
        } else {
            Ok(())
        }
    }
}

/// Representation of a refresh token.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct RefreshToken(pub String);
impl RefreshToken {
    pub const VALID_IN: Duration = Duration::from_secs(6 * 31 * 24 * 60 * 60);

    pub fn generate(uid: Uid) -> Self {
        let mut data = [0u8; 128];
        let timestamp = Utc::now().timestamp_micros();
        data[0..8].copy_from_slice(&timestamp.to_le_bytes());
        data[8..16].copy_from_slice(&uid.0.to_le_bytes());
        rand::rng().fill_bytes(&mut data[16..]);
        Self(BASE64_URL_SAFE.encode(&data))
    }
}

/// Representation of an access token.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct AccessToken(pub String);
impl AccessToken {
    pub const VALID_IN: Duration = Duration::from_secs(14 * 24 * 60 * 60);

    pub fn generate(uid: Uid) -> Self {
        let mut data = [0u8; 80];
        let timestamp = Utc::now().timestamp_micros();
        data[0..8].copy_from_slice(&timestamp.to_le_bytes());
        data[8..16].copy_from_slice(&uid.0.to_le_bytes());
        rand::rng().fill_bytes(&mut data[16..]);
        Self(BASE64_URL_SAFE.encode(&data))
    }
}
