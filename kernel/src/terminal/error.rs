use alloc::string::String;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("error code {0}")]
    Code(u64),
    
    #[error("{0}")]
    Message(String),
}