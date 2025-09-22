use crate::clock::{sleep};
use crate::terminal::error::CliError;
use goolog::{trace};

const GOOLOG_TARGET: &str = "SLEEP";

pub fn cli_sleep(seconds: u64) -> Result<(), CliError> {
    trace!("SLEEP");
    
    sleep(seconds);
    
    Ok(())
}