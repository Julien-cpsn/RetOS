use goolog::trace;
use crate::println;
use crate::terminal::error::CliError;

const GOOLOG_TARGET: &str = "ECHO";

pub fn echo(text: &str) -> Result<(), CliError> {
    trace!("ECHO");

    println!("{text}");
    Ok(())
}