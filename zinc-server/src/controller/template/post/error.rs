//!
//! The template resource POST response error.
//!

use std::fmt;

use actix_web::http::StatusCode;
use actix_web::ResponseError;

///
/// The template resource POST response error.
///
#[derive(Debug)]
pub enum Error {
    Compiler(String),
    Database(sqlx::Error),
    NotAContract,
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Compiler(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Database(_inner) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotAContract => StatusCode::UNPROCESSABLE_ENTITY,
        }
    }
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compiler(inner) => write!(f, "{}", inner),
            Self::Database(inner) => write!(f, "Database: {:?}", inner),
            Self::NotAContract => write!(f, "Not a contract"),
        }
    }
}