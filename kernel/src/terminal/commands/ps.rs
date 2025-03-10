use goolog::{debug, set_target};
use crate::println;
use crate::task::executor::TASKS;
use crate::terminal::error::CliError;

pub fn ps() -> Result<(), CliError> {
    set_target!("PS");

    debug!("Locking TASKS_MIRROR mutex...");
    let tasks = TASKS.read();
    debug!("TASKS_MIRROR mutex locked");

    for (id, task) in tasks.iter() {
        println!("{}: {}", id.0, task.name);
    }

    debug!("TASKS_MIRROR mutex freed");

    Ok(())
}