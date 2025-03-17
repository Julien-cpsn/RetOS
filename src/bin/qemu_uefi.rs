use std::process::{exit, Command};
use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};

fn main() {
    let prebuilt = Prebuilt::fetch(Source::EDK2_STABLE202408_R1, "target/ovmf").expect("failed to update prebuilt");

    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-drive").arg(format!("format=raw,file={}", env!("UEFI_IMAGE")));
    qemu.arg("-drive").arg(format!("if=pflash,format=raw,file=./{}", prebuilt.get_file(Arch::X64, FileType::Code).display()));
    qemu.arg("-serial").arg("stdio");
    
    let exit_status = qemu.status().unwrap();
    match exit_status.code() {
        None => exit(-1),
        Some(code) => exit(code),
    }
}