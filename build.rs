use bootloader::DiskImageBuilder;
use std::env;
use std::path::PathBuf;

fn main() {
    let disk_builder = DiskImageBuilder::new(PathBuf::from(
        env::var("CARGO_BIN_FILE_RETOS_KERNEL").unwrap(),
    ));

    // specify output paths
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let uefi_path = out_dir.join("retos-uefi.img");
    let bios_path = out_dir.join("retos-bios.img");

    // create the disk images
    disk_builder.create_uefi_image(&uefi_path).unwrap();
    disk_builder.create_bios_image(&bios_path).unwrap();

    // pass the disk image paths via environment variables
    println!("cargo:rustc-env=UEFI_IMAGE={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_IMAGE={}", bios_path.display());
}

