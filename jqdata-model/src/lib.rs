//! JQData
//! 
//! Rust implementation of JQData API client

pub mod errors;
pub mod models;

pub use errors::*;
pub use models::*;

pub type Result<T> = std::result::Result<T, Error>;
