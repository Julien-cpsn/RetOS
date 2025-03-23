use goolog::trace;
use crate::printer::buffer::WRITER;
use crate::terminal::error::CliError;

const GOOLOG_TARGET: &str = "CLEAR";

pub fn clear() -> Result<(), CliError> {
    trace!("Clear");
    WRITER.write().clear();
    Ok(())
}