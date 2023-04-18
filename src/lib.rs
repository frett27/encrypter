#![warn(clippy::all, rust_2018_idioms)]

mod app;

pub use app::EncrypterApp;

pub mod encrypt;

pub mod folder;

pub mod keys_management;

use std::error;
use std::fmt;
use std::str;

/// Enum listing possible errors from rusqlite.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Utf8Error(str::Utf8Error),
}

impl From<str::Utf8Error> for Error {
    #[cold]
    fn from(err: str::Utf8Error) -> Error {
        Error::Utf8Error(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
}

pub type Result<T> = std::result::Result<T, Box<dyn error::Error>>;
