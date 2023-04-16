#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::EncrypterApp;

pub mod encrypt;

pub mod folder;
