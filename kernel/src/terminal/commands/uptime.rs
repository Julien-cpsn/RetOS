use crate::clock::Clock;
use crate::println;
use crate::terminal::error::CliError;
use goolog::{trace};

const GOOLOG_TARGET: &str = "UPTIME";

pub fn uptime() -> Result<(), CliError> {
    trace!("UPTIME");
    
    println!("{}", Clock::format());
    Ok(())
}