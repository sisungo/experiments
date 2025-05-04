//! Application settings.

use crate::{AppState, database::entity::app_settings, error::Error};
use sea_orm::{ActiveValue::Set, EntityTrait, sea_query::OnConflict};
use serde::{Serialize, de::DeserializeOwned};

/// Macro for creating an app setting entry.
macro_rules! entry {
    ($name:ident : $ty:ty) => {
        entry!($name : $ty = Default::default());
    };
    ($name:ident : $ty:ty = $defval:expr) => {
        #[derive(Debug)]
        pub struct $name;

        impl Entry for $name {
            type Ty = $ty;

            fn key() -> &'static str {
                stringify!($name)
            }

            fn default_value() -> Self::Ty {
                $defval
            }
        }
    };
}

/// An app setting entry.
pub trait Entry {
    /// The type value of the entry.
    type Ty: Serialize + DeserializeOwned;

    /// Returns the key of the entry.
    fn key() -> &'static str;

    /// Returns the default value of the entry.
    fn default_value() -> Self::Ty;
}

#[derive(Debug)]
pub struct AppSettings<'a>(&'a AppState);
impl AppSettings<'_> {
    /// Returns the value of the specified app setting entry.
    pub async fn get<T: Entry>(&self) -> T::Ty {
        let model = match app_settings::Entity::find_by_id(T::key())
            .one(&*self.0.database.conn)
            .await
        {
            Ok(x) => x,
            Err(err) => {
                tracing::warn!("failed to get app setting entry from database: {err}");
                return T::default_value();
            }
        };

        let Some(model) = model else {
            tracing::debug!(
                "key `{}` not recorded in the database, using default value",
                T::key()
            );
            return T::default_value();
        };

        serde_json::from_value(model.value)
            .inspect_err(|err| {
                tracing::warn!(
                    "failed to parse value found at \"AppSettings/{}\": {}",
                    T::key(),
                    err
                );
            })
            .unwrap_or_else(|_| T::default_value())
    }

    /// Sets the value of the specified app setting entry.
    pub async fn set<T: Entry>(&self, value: &T::Ty) -> Result<(), Error> {
        let value = serde_json::to_value(value).map_err(|err| Error::internal(err))?;

        app_settings::Entity::insert(app_settings::ActiveModel {
            key: Set(T::key().into()),
            value: Set(value),
        })
        .on_conflict(
            OnConflict::column(app_settings::Column::Key)
                .update_column(app_settings::Column::Key)
                .to_owned(),
        )
        .exec(&*self.0.database.conn)
        .await?;

        Ok(())
    }
}

entry!(SiteName : String = "Vinyl Server".into());
entry!(RegisterRequirements: crate::user::register::RegisterRequires);
entry!(MandatoryAlbumCensorship: bool);
entry!(MandatorySongCensorship: bool);
entry!(MandatoryCommentCensorship: bool);
entry!(LicenseHTML: String = include_str!("../resources/DefaultEula.html").into());

impl AppState {
    /// Returns the app settings manager.
    pub fn app_settings(&self) -> AppSettings {
        AppSettings(self)
    }
}
