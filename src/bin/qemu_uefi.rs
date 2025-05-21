use std::process::{exit, Command};
use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};
use retos::constants::{MAC_ADDRESS, NETWORK_INTERFACE, NIC_MODEL, TELNET};

fn main() {
    let prebuilt = Prebuilt::fetch(Source::EDK2_STABLE202408_R1, "target/ovmf").expect("failed to update prebuilt");

    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-drive").arg(format!("format=raw,file={}", env!("UEFI_IMAGE")));
    qemu.arg("-drive").arg(format!("if=pflash,format=raw,file=./{}", prebuilt.get_file(Arch::X64, FileType::Code).display()));

    qemu.arg("-serial").arg(format!("telnet:{TELNET},server,nowait"));
    qemu.arg("-netdev").arg(format!("tap,id=u1,ifname={NETWORK_INTERFACE},script=no,downscript=no"));
    qemu.arg("-device").arg(format!("{NIC_MODEL},mac={MAC_ADDRESS},netdev=u1"));
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