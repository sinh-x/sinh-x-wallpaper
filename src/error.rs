use crate::database::DatabaseError;
use serde_json::Error as SerdeJsonError;
use std::convert::From;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum MyError {
    Io(std::io::Error),
    Reqwest(reqwest::Error),
    JsonError(String),
    DatabaseError(String),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::DatabaseError(err) => write!(f, "Database error: {}", err),
            MyError::Io(err) => write!(f, "IO error: {}", err),
            MyError::Reqwest(err) => write!(f, "Reqwest error: {}", err),
            MyError::JsonError(err) => write!(f, "JSON error: {}", err),
        }
    }
}

impl Error for MyError {}

impl From<std::io::Error> for MyError {
    fn from(err: std::io::Error) -> MyError {
        MyError::Io(err)
    }
}

impl From<reqwest::Error> for MyError {
    fn from(err: reqwest::Error) -> MyError {
        MyError::Reqwest(err)
    }
}

impl From<SerdeJsonError> for MyError {
    fn from(error: SerdeJsonError) -> Self {
        MyError::JsonError(error.to_string())
    }
}

impl From<DatabaseError> for MyError {
    fn from(err: DatabaseError) -> MyError {
        MyError::DatabaseError(err.to_string())
    }
}
