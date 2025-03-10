use crate::printer::buffer::WRITER;
use crate::terminal::error::CliError;
use goolog::set_target;

pub fn clear() -> Result<(), CliError> {
    set_target!("CLEAR");

    WRITER.write().clear();
    Ok(())
}

