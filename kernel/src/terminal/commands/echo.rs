use alloc::string::String;
use goolog::trace;
use no_std_clap_macros::Args;
use crate::println;
use crate::terminal::error::CliError;

const GOOLOG_TARGET: &str = "ECHO";


#[derive(Args)]
pub struct EchoCommand {
    /// Text to echo
    pub text: String
}

pub fn echo(text: &str) -> Result<(), CliError> {
    trace!("ECHO");

    println!("{text}");
    Ok(())
}