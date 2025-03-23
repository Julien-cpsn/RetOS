use goolog::{debug, trace};
use crate::println;
use crate::task::executor::TASKS;
use crate::terminal::error::CliError;

const GOOLOG_TARGET: &str = "PS";

pub fn ps() -> Result<(), CliError> {
    trace!("PS");
    
    debug!("Locking TASKS_MIRROR mutex...");
    let tasks = TASKS.read();
    debug!("TASKS_MIRROR mutex locked");

    for (id, task) in tasks.iter() {
        println!("{}: {}", id.0, task.name);
    }

    debug!("TASKS_MIRROR mutex freed");

    Ok(())
}