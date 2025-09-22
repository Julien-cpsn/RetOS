use std::env;
use std::process::{exit, Command};
use retos::constants::{NICS, TELNET};

fn main() {
    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-drive").arg(format!("format=raw,file={}", env!("BIOS_IMAGE")));

    qemu.arg("-serial").arg(format!("telnet:{TELNET},server,nowait"));

    for (index, nic) in NICS.iter().enumerate() {
        let net = match nic.tap {
            None => format!("user,id=u{index}"),
            Some(tap) => format!("tap,id=u{index},ifname={tap},script=no,downscript=no")
        };

        qemu.arg("-netdev").arg(net);
        qemu.arg("-device").arg(format!("{},mac={},netdev=u{index}", nic.model, nic.mac));
    }

    if NICS.len() > 1 {
        qemu.arg("-object").arg("filter-dump,id=f1,netdev=u1,file=dump.pcap");
    }

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