use std::env;
use std::process::{exit, Command};

fn main() {
    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-drive");
    qemu.arg(format!("format=raw,file={}", env!("BIOS_IMAGE")));
    /*
    qemu.arg("-device");
    qemu.arg("qemu-xhci");
    qemu.arg("-device");
    qemu.arg("usb-kbd");
     */

    let exit_status = qemu.status().unwrap();
    match exit_status.code() {
        None => exit(-1),
        Some(code) => exit(code),
    }
}