use alloc::string::String;
use embedded_cli::__private::io::{ErrorKind, ErrorType};
use thiserror::Error;
use crate::terminal::terminal::TerminalBuffer;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("error code {0}")]
    Code(u64),
    
    #[error("{0}")]
    Message(String),
}

impl embedded_cli::__private::io::Error for CliError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

impl ErrorType for TerminalBuffer {
    type Error = CliError;
}