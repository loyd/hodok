use std::error;
use std::result;

#[macro_use]
pub mod node;
pub mod logger;


pub type Result<T> = result::Result<T, Box<error::Error + Send + Sync>>;
