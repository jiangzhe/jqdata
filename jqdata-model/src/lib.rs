//! JQData
//! 
//! Rust implementation of JQData API client

pub mod errors;
pub mod models;

pub use crate::errors::*;
pub use crate::models::*;

pub type Result<T> = std::result::Result<T, Error>;
