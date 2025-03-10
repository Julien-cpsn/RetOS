use crate::println;
use crate::terminal::error::CliError;
use goolog::set_target;

pub fn shutdown() -> Result<(), CliError> {
    set_target!("SHUTDOWN");

    println!("shutting down");
    Ok(())
}

