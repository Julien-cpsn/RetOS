use crate::println;
use crate::terminal::error::CliError;
use goolog::set_target;

pub fn echo(text: &str) -> Result<(), CliError> {
    set_target!("ECHO");

    println!("{text}");
    Ok(())
}

