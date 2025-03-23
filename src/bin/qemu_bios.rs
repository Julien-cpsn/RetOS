use std::env;
use std::process::{exit, Command};
use retos::constants::{NETWORK_INTERFACE, NIC_MODEL};

fn main() {
    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-drive").arg(format!("format=raw,file={}", env!("BIOS_IMAGE")));
    qemu.arg("-serial").arg("stdio");
    qemu.arg("-netdev").arg(format!("tap,id=u1,ifname={NETWORK_INTERFACE},script=no,downscript=no"));
    qemu.arg("-device").arg(format!("{NIC_MODEL},netdev=u1"));
    qemu.arg("-object").arg("filter-dump,id=f1,netdev=u1,file=dump.pcap");
    qemu.arg("--device").arg("isa-debug-exit,iobase=0xf4,iosize=0x04");
    /*
    qemu.arg("-device").arg("qemu-xhci");
    qemu.arg("-device").arg("usb-kbd");
     */
    
    let exit_status = qemu.status().unwrap();
    match exit_status.code() {
        None => exit(-1),
        Some(code) => exit(code),
    }
}