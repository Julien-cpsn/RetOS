use crate::clock::MilliSecondClock;
use crate::println;
use crate::terminal::error::CliError;
use goolog::{trace};

const GOOLOG_TARGET: &str = "UTPIME";

pub fn uptime() -> Result<(), CliError> {
    trace!("UPTIME");
    
    println!("{}", MilliSecondClock::format());
    Ok(())
}