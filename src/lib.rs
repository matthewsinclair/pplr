pub mod assets;
pub mod cli;
pub mod config;
pub mod contacts;
pub mod doctor;
pub mod fallback;
pub mod fs;
pub mod icons;
pub mod layout;
pub mod tools;

mod error;
pub use error::{Error, Result, clean};
