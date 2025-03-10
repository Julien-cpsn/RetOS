use goolog::{debug, set_target};
use crate::println;
use crate::task::executor::{TASKS_MIRROR};
use crate::terminal::error::CliError;

pub fn ps() -> Result<(), CliError> {
    set_target!("PS");

    debug!("Locking TASKS_MIRROR mutex...");
    let tasks = TASKS_MIRROR.lock();
    debug!("TASKS_MIRROR mutex locked");

    for (id, name) in tasks.iter() {
        println!("{}: {}", id.0, name);
    }

    debug!("TASKS_MIRROR mutex freed");

    Ok(())
}