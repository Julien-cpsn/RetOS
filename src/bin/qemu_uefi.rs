use std::process::{exit, Command};
use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};
use retos::constants::{NICS, TELNET};

fn main() {
    let prebuilt = Prebuilt::fetch(Source::EDK2_STABLE202408_R1, "target/ovmf").expect("failed to update prebuilt");

    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-drive").arg(format!("format=raw,file={}", env!("UEFI_IMAGE")));
    qemu.arg("-drive").arg(format!("if=pflash,format=raw,file=./{}", prebuilt.get_file(Arch::X64, FileType::Code).display()));

    qemu.arg("-serial").arg(format!("telnet:{TELNET},server,nowait"));
    
    for (index, nic) in NICS.iter().enumerate() {
        let net = match nic.tap {
            None => format!("user,id=u{index}"),
            Some(tap) => format!("tap,id=u{index},ifname={tap},script=no,downscript=no")
        };

        qemu.arg("-netdev").arg(net);
        qemu.arg("-device").arg(format!("{},mac={},netdev=u{index}", nic.model, nic.mac));

        qemu.arg("-object").arg(format!("filter-dump,id=f{index},netdev=u{index},file=dump_u{index}.pcap"));
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