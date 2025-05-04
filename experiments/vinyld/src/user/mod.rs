pub mod auth;
pub mod extract;
pub mod group;
pub mod login;
pub mod register;
pub mod session;

use crate::{AppState, database::entity::user, error::Error};
use chrono::{DateTime, Utc};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use vinioss::ObjectKey;
use vinutie::def_verify;

#[derive(Debug)]
pub struct Users<'a>(&'a AppState);
impl Users<'_> {
    pub async fn find_by_uid(&self, uid: Uid) -> Result<User, Error> {
        user::Entity::find_by_id(uid.0)
            .one(&*self.0.database.conn)
            .await?
            .ok_or_else(|| Error::not_found())
            .map(User::from)
    }

    pub async fn find_by_username(&self, username: &str) -> Result<User, Error> {
        user::Entity::find()
            .filter(user::Column::Username.eq(username))
            .one(&*self.0.database.conn)
            .await?
            .ok_or_else(|| Error::not_found())
            .map(User::from)
    }

    pub async fn avatar_hq(&self, uid: Uid) -> Result<ObjectKey, Error> {
        user::Entity::find_by_id(uid.0)
            .one(&*self.0.database.conn)
            .await?
            .ok_or_else(|| Error::not_found())?
            .avatar
            .map(ObjectKey)
            .ok_or_else(|| Error::not_found())
    }
}

/// Representation of a UID.
///
/// A UID identifies a user-like entity uniquely and cannot be changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Uid(pub i64);

/// Public profile of a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Profile(User);
impl From<User> for Profile {
    fn from(mut user: User) -> Self {
        user.groups.clear();
        Self(user)
    }
}

/// Basic stub of a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub uid: Uid,
    pub username: String,
    pub groups: Vec<String>,
    pub banned: bool,
}
impl From<user::Model> for User {
    fn from(model: user::Model) -> Self {
        Self {
            uid: Uid(model.uid),
            username: model.username,
            groups: model.groups,
            banned: model.banned.is_some(),
        }
    }
}

/// Banning of a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ban {
    pub code: BanCode,
    pub begin_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub message: String,
}

/// Code of banning a user.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct BanCode(u32);

impl AppState {
    /// Returns a user manager.
    pub fn users(&self) -> Users<'_> {
        Users(self)
    }
}

def_verify!(pub UsernameLike<str>(err: Error = Error::invalid_username()) = |x: &str| {
    x.len() <= 24 && x.chars().all(|y| y.is_ascii_alphanumeric() || y == '-' || y == '_')
});
def_verify!(pub NicknameLike<str>(err: Error = Error::invalid_nickname()) = |x: &str| {
    x.chars().count() <= 24
});
