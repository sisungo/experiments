use super::{
    Uid, Users,
    auth::password,
    session::{AccessToken, Permissions, RefreshToken},
};
use crate::{database::entity::user_auth_password, error::Error};
use argon2::password_hash::{PasswordHash, PasswordVerifier};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub refresh_token: RefreshToken,
    pub access_token: AccessToken,

    pub uid: Uid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserProber {
    Username(String),
}

impl Users<'_> {
    pub async fn login_via_password(
        &self,
        prober: &UserProber,
        password: &str,
        source: String,
    ) -> Result<LoginResponse, Error> {
        let user = match prober {
            UserProber::Username(username) => self
                .find_by_username(username)
                .await
                .map_err(|_| Error::login_incorrect())?,
        };
        let passwd_rec = user_auth_password::Entity::find_by_id(user.uid.0)
            .one(&*self.0.database.conn)
            .await?
            .ok_or_else(|| Error::login_incorrect())?;

        let parsed_hash =
            PasswordHash::new(&passwd_rec.password).map_err(|err| Error::internal(err))?;
        if password::hasher()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
        {
            let (refresh_token, access_token) = self
                .put_session(user.uid, Permissions::all(), source)
                .await?;
            Ok(LoginResponse {
                refresh_token,
                access_token,
                uid: user.uid,
            })
        } else {
            Err(Error::login_incorrect())
        }
    }
}
