# RetOS

A Router Network Operating System. RetOS comes from *retis* which mean *network* in Latin.

> [!WARNING]
> This is a PhD project, and it is still very WIP.

> [!NOTE]
> This Operating System is based on the great [Writing an OS in Rust](https://os.phil-opp.com/) from [@phil-opp](https://github.com/phil-opp). 

## How to use

### Development

> [!IMPORTANT]
> You will need QEMU x86_64 in order to run project in development mode
> - [Download for Linux](https://www.qemu.org/download/#linux) 
> - [Download for MacOS](https://www.qemu.org/download/#macos)
> - [Download for Windows](https://www.qemu.org/download/#windows)
> - [Install with Nix](https://search.nixos.org/packages?show=qemu)
> 
> You can check the command is available by running
> ```shell
> qemu-system-x86_64 --version
> ```

Run the project with QEMU

```shell
cargo run
```

> [!NOTE]
> If further toolchain installation is needed, `rust-toolchain.toml` depicts what's needed.

### Production

By running the following command, you will build the OS images and the executables that will use QEMU.

```shell
cargo build --release
```

If you need the images (`.img` files), you can find them like so:

```shell
find ./target/release -name "retos-*.img"
```

## Done & TODOs

- Core
  - [ ] ANSII colors
  - [ ] Multi-threading
  - [x] Log system (with [goolog](https://github.com/Gooxey/goolog))
  - [x] Internal clock
  - [x] Command Line Interface (with [embedded-cli-rs](https://github.com/funbiscuit/embedded-cli-rs))
  - [x] Async/Await
  - [x] Framebuffer (print, clear, colors)
  - [x] Main x86_64 instructions, exceptions and interruptions (with [x86_64](https://github.com/rust-osdev/x86_64))
  - [x] Bootloader (with [bootloader](https://github.com/rust-osdev/bootloader))
  - [x] Standalone kernel
- Devices
  - [ ] VirtIO? (maybe [rust-osdev/virtio](https://docs.rs/virtio-spec/latest/virtio_spec/))
  - [ ] NIC (E1000)
  - [ ] USB (maybe [rust-osdev/usb](https://github.com/rust-osdev/usb) or [usb-device](https://docs.rs/usb-device/latest/usb_device/index.html))
  - [ ] xHCI (maybe [rust-osdev/xhci](https://docs.rs/xhci/latest/xhci/))
  - [ ] APIC (maybe [this merge request](https://github.com/rust-osdev/bootloader/pull/460/files))
  - [x] PCI (WIP with [pci_types](https://docs.rs/pci_types/0.10.0/pci_types/))
  - [x] PS2 Keyboard (with [pc_keyboard](https://github.com/rust-embedded-community/pc-keyboard))
  - [x] PIC (with [pic8259](https://github.com/rust-osdev/pic8259))
- Commands
  - [x] scanpci
  - [x] lspci
  - [x] ps
  - [x] shutdown (WIP)
  - [x] keyboard (change keyboard layout)
  - [x] uptime
  - [x] clear
  - [x] echo
- Network
  - [ ] TCP/IP stack (maybe [smol-tcp](https://github.com/smoltcp-rs/smoltcp))
  - [ ] Routing stack
- Memory
  - [ ] More precise heap allocation
  - [x] Heap allocation (with [Talc](https://github.com/SFBdragon/talc))
  - [x] Memory pagination
- Others
  - [ ] Burn image to USB stick (maybe need [multiboot2](https://github.com/rust-osdev/multiboot2), [doc](https://docs.rs/multiboot2/latest/multiboot2/))
  - [ ] Linux VM?

## Contributors

- [@Julien-cpsn](https://github.com/Julien-cpsn) - Main contributor
- [@i5-650](https://github.com/i5-650) - Discussion & help

## License

This project is licensed under the MIT license and can be found [here](https://github.com/Julien-cpsn/RetOS/blob/main/LICENSE).