use super::{
    NicknameLike, Uid, UsernameLike, Users,
    auth::password::{self, PasswordLike},
    login::LoginResponse,
    session::Permissions,
};
use crate::{
    app_settings::RegisterRequirements,
    database::entity::{user, user_auth_password},
    error::Error,
};
use argon2::{PasswordHasher, password_hash::SaltString};
use bitflags::bitflags;
use chrono::Utc;
use sea_orm::{
    ActiveValue::{NotSet, Set},
    EntityTrait,
    sea_query::OnConflict,
};
use serde::{Deserialize, Serialize};
use vinutie::verify::verify_option;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: Option<String>,
    pub nickname: Option<String>,
    pub password: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub source: String,
}

bitflags! {
    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct RegisterRequires: u32 {
        /// Require the user to specify a username on registration.
        const USERNAME = 2 << 0;

        /// Require the user to set a password on registration.
        const PASSWORD = 2 << 1;

        /// Require the user to set an email address on registration.
        ///
        /// Note that this does not enable mandatory email-based verification.
        const EMAIL = 2 << 2;

        /// Require the user to set a phone number on registration.
        ///
        /// Note that this does not enable mandatory SMS-based verification.
        const PHONE = 2 << 3;
    }
}
impl Default for RegisterRequires {
    fn default() -> Self {
        Self::empty()
    }
}

impl Users<'_> {
    pub async fn register(
        &self,
        RegisterRequest {
            username,
            password,
            nickname,
            email,
            phone,
            source,
        }: RegisterRequest,
    ) -> Result<LoginResponse, Error> {
        let requires = self.0.app_settings().get::<RegisterRequirements>().await;

        verify_option::<UsernameLike, _>(username.as_deref())?;
        verify_option::<NicknameLike, _>(nickname.as_deref())?;
        verify_option::<PasswordLike, _>(password.as_deref())?;

        let username = match username {
            Some(x) => x,
            None => {
                if requires.contains(RegisterRequires::USERNAME) {
                    return Err(Error::registration_form_not_filled());
                } else {
                    todo!();
                }
            }
        };

        let hashed_password = match &password {
            Some(x) => Some(
                password::hasher()
                    .hash_password(
                        x.as_bytes(),
                        &SaltString::generate(&mut argon2::password_hash::rand_core::OsRng),
                    )
                    .map_err(|_| Error::invalid_password())?
                    .to_string(),
            ),
            None => {
                if requires.contains(RegisterRequires::PASSWORD) {
                    return Err(Error::registration_form_not_filled());
                }
                None
            }
        };

        let user = user::Entity::insert(user::ActiveModel {
            uid: NotSet,
            username: Set(username.clone()),
            nickname: Set(nickname.unwrap_or(username)),
            avatar: Set(None),
            gender: Set(None),
            date_of_birth: Set(None),
            country: Set(None),
            city: Set(None),
            signature: Set(None),
            banner_image: Set(None),
            banned: Set(None),
            groups: Set(Vec::new()),
            created_at: Set(Utc::now().naive_utc()),
            last_logined_at: Set(Utc::now().naive_utc()),
        })
        .on_conflict(
            OnConflict::column(user::Column::Username)
                .do_nothing()
                .to_owned(),
        )
        .exec(&*self.0.database.conn)
        .await
        .map_err(|err| match err {
            sea_orm::DbErr::RecordNotInserted => Error::username_conflict(),
            err => Error::internal(err),
        })?;

        let uid = Uid(user.last_insert_id);

        if let Some(x) = hashed_password {
            user_auth_password::Entity::insert(user_auth_password::ActiveModel {
                uid: Set(uid.0),
                password: Set(x),
            })
            .exec(&*self.0.database.conn)
            .await?;
        }

        let (refresh_token, access_token) =
            self.put_session(uid, Permissions::all(), source).await?;

        Ok(LoginResponse {
            refresh_token,
            access_token,
            uid,
        })
    }
}
