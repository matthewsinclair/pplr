pub mod cli;
pub mod config;
pub mod fallback;
pub mod tree;

mod error;
pub use error::{Error, Result, clean};
