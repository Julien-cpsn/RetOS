use goolog::trace;
use qemu_exit::QEMUExit;
use crate::clock::sleep;
use crate::println;
use crate::terminal::error::CliError;

const GOOLOG_TARGET: &str = "SHUTDOWN";

pub fn shutdown() -> Result<(), CliError> {
    trace!("SHUTDOWN");

    println!("shutting down in 5 seconds...");

    sleep(5);
    let exit = qemu_exit::X86::new(0xF4, 5);
    exit.exit(1);
}