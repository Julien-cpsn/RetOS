use crate::clock::MilliSecondClock;
use crate::println;
use crate::terminal::error::CliError;
use goolog::set_target;

pub fn uptime() -> Result<(), CliError> {
    set_target!("UPTIME");

    println!("{}", MilliSecondClock::format());
    Ok(())
}

