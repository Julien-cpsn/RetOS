use crate::terminal::terminal::TerminalBuffer;
use embedded_cli::__private::io::{ErrorKind, ErrorType};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Error code: {0}")]
    Code(u64),
}

impl embedded_cli::__private::io::Error for CliError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

impl ErrorType for TerminalBuffer {
    type Error = CliError;
}

