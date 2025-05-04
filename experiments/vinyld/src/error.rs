use axum::{
    body::Body,
    extract::multipart::MultipartError,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use sea_orm::DbErr;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

macro_rules! simple_error_constructor {
    ($name:ident, $code:ident, $msg:literal,) => {
        simple_error_constructor!($name, $code, $msg);
    };
    ($name:ident, $code:ident, $msg:literal) => {
        pub fn $name() -> Self {
            Self {
                code: ErrorCode::$code,
                message: $msg.into(),
                payload: None,
            }
        }
    };
}

/// An error that can be returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Error {
    pub code: ErrorCode,
    pub message: String,
    pub payload: Option<serde_json::Value>,
}
impl Error {
    pub fn bad_request<E: Display>(since: E) -> Self {
        Self {
            code: ErrorCode::BAD_REQUEST,
            message: format!("{since}"),
            payload: None,
        }
    }

    simple_error_constructor!(unauthorized, UNAUTHORIZED, "You must login to continue.");
    simple_error_constructor!(
        restricted_session,
        RESTRICTED_SESSION,
        "The operation requires a permission that has been restricted by current session.",
    );
    simple_error_constructor!(
        restricted_user,
        RESTRICTED_USER,
        "The operation requires a permission that cannot be performed by current user.",
    );
    simple_error_constructor!(
        username_conflict,
        USERNAME_CONFLICT,
        "The username to be saved conflicts with that of someone else.",
    );
    simple_error_constructor!(payload_too_large, PAYLOAD_TOO_LARGE, "Payload too large.");
    simple_error_constructor!(
        invalid_username,
        INVALID_USERNAME,
        "The username being requested is malformed.",
    );
    simple_error_constructor!(
        invalid_password,
        INVALID_PASSWORD,
        "The password is not allowed, it may be too long, be containing disallowed characters or be too simple.",
    );
    simple_error_constructor!(
        invalid_nickname,
        INVALID_NICKNAME,
        "The nickname is not allowed.",
    );
    simple_error_constructor!(login_incorrect, LOGIN_INCORRECT, "Login incorrect.");
    simple_error_constructor!(
        registration_form_not_filled,
        REGISTRATION_FORM_NOT_FILLED,
        "Registration on this server requires one or more form entry that is not filled in this request.",
    );
    simple_error_constructor!(
        not_found,
        NOT_FOUND,
        "The resource being requested is not found.",
    );
    simple_error_constructor!(
        non_existent_album,
        NON_EXISTENT_ALBUM,
        "The album required is not found.",
    );

    pub fn banned_user(payload: serde_json::Value) -> Self {
        Self {
            code: ErrorCode::BANNED_USER,
            message: "The user has been banned on this server.".into(),
            payload: Some(payload),
        }
    }

    pub fn denied_by_policy(class: &str) -> Self {
        Self {
            code: ErrorCode::DENIED_BY_POLICY,
            message: "The request is denied by the requested resource's policy.".into(),
            payload: Some(serde_json::json! {{"class": class}}),
        }
    }

    pub fn internal<E: Display>(since: E) -> Self {
        Self {
            code: ErrorCode::INTERNAL,
            message: format!("{since}"),
            payload: None,
        }
    }

    fn to_body(&self) -> Body {
        Body::new(serde_json::to_string(&self).unwrap())
    }

    fn to_headers(&self) -> HeaderMap<HeaderValue> {
        let mut headers = HeaderMap::new();

        if self.code.to_http() == StatusCode::UNAUTHORIZED {
            headers.append("WWW-Authenticate", HeaderValue::from_static("Vinyl-Token"));
        }

        headers
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "E{}: {}", self.code.0, self.message)
    }
}
impl std::error::Error for Error {}
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let mut response = Response::new(self.to_body());
        *response.status_mut() = self.code.to_http();
        *response.headers_mut() = self.to_headers();
        response
    }
}
impl From<DbErr> for Error {
    fn from(err: DbErr) -> Self {
        Self::internal(err)
    }
}
impl From<MultipartError> for Error {
    fn from(value: MultipartError) -> Self {
        match value.status() {
            StatusCode::PAYLOAD_TOO_LARGE => Self::payload_too_large(),
            _ => Self::internal(value),
        }
    }
}
impl From<vinioss::Error> for Error {
    fn from(value: vinioss::Error) -> Self {
        match value {
            vinioss::Error::ObjectNotFound => Self::not_found(),
            vinioss::Error::UnknownVendor(err) => Self::internal(err),
            vinioss::Error::InvalidConfiguration(err) => Self::internal(err),
            vinioss::Error::Unrecognized(err) => Self::internal(err),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct ErrorCode(pub u32);
impl ErrorCode {
    pub const BAD_REQUEST: Self = Self(400);
    pub const UNAUTHORIZED: Self = Self(401);
    pub const RESTRICTED_SESSION: Self = Self(40301);
    pub const RESTRICTED_USER: Self = Self(40302);
    pub const BANNED_USER: Self = Self(40303);
    pub const DENIED_BY_POLICY: Self = Self(40304);
    pub const NOT_FOUND: Self = Self(404);
    pub const USERNAME_CONFLICT: Self = Self(40901);
    pub const PAYLOAD_TOO_LARGE: Self = Self(413);
    pub const INVALID_USERNAME: Self = Self(42201);
    pub const INVALID_PASSWORD: Self = Self(42202);
    pub const INVALID_NICKNAME: Self = Self(42203);
    pub const LOGIN_INCORRECT: Self = Self(42204);
    pub const REGISTRATION_FORM_NOT_FILLED: Self = Self(42205);
    pub const NON_EXISTENT_ALBUM: Self = Self(42206);

    pub const INTERNAL: Self = Self(500);

    pub fn to_http(self) -> StatusCode {
        if self.0 > 10000 {
            StatusCode::from_u16(self.0 as u16 / 100).unwrap()
        } else {
            StatusCode::from_u16(self.0 as _).unwrap()
        }
    }
}
