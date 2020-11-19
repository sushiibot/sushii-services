use darkredis::Error as RedisError;
use dotenv::Error as DotenvError;
use serde_json::Error as SerdeJsonError;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Error as IoError;
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    // Crate errors
    Dotenv(DotenvError),
    Io(IoError),
    Json(SerdeJsonError),
    Redis(RedisError),
}

impl From<RedisError> for Error {
    fn from(err: RedisError) -> Error {
        Error::Redis(err)
    }
}

impl From<SerdeJsonError> for Error {
    fn from(err: SerdeJsonError) -> Error {
        Error::Json(err)
    }
}

impl From<DotenvError> for Error {
    fn from(err: DotenvError) -> Error {
        Error::Dotenv(err)
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::Io(err)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(self)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Error::Dotenv(ref inner) => inner.fmt(f),
            Error::Io(ref inner) => inner.fmt(f),
            Error::Json(ref inner) => inner.fmt(f),
            Error::Redis(ref inner) => inner.fmt(f),
        }
    }
}
