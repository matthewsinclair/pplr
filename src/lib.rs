pub mod cli;
pub mod config;
pub mod contacts;
pub mod fallback;
pub mod fs;
pub mod tree;

mod error;
pub use error::{Error, Result, clean};
