> [!WARNING]
> This is a PhD project, and it is still very WIP.

# RetOS

A Router Network Operating System. RetOS comes from *retis* which mean *network* in Latin.

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

### Burn to hard drive/USB stick

#### 1. Get the image path

Build the OS and find the image file you want to use (BIOS or UEFI), you can do it like so:

```shell
cargo build --release
find ./target/release -maxdepth 1 -name "*.img"
```

#### 2. Find the hard drive/USB stick

You can find it by using the `lsblk` command. A possible selection for your device can be like `/dev/sda1`.

#### 3. Burn the image

Now you have to burn the image on the hard drive/USB stick with the following command:

```shell
dd if=<IMAGE_PATH> of=<DEVICE> bs=1M && sync
```
#### 4. Run it live!

You can directly plug your device onto the PC and boot it :)

> [!NOTE]
> If you are using the BIOS image, you may want to enable CMS

## Done & TODOs

- **Core**
  - [ ] Multi-threading (SMP)
  - [x] ANSI colors (WIP)
  - [x] Log system (with [goolog](https://github.com/Gooxey/goolog))
  - [x] Internal clock
  - [x] Command Line Interface (with [embedded-cli-rs](https://github.com/funbiscuit/embedded-cli-rs))
  - [x] Async/Await
  - [x] Framebuffer (print, clear, colors)
  - [x] Main x86_64 instructions, exceptions and interruptions (with [x86_64](https://github.com/rust-osdev/x86_64))
  - [x] Bootloader (with [bootloader](https://github.com/rust-osdev/bootloader))
  - [x] Standalone kernel
- **Devices**
  - [ ] VirtIO? (maybe [rust-osdev/virtio](https://docs.rs/virtio-spec/latest/virtio_spec/))
  - [ ] USB (maybe [rust-osdev/usb](https://github.com/rust-osdev/usb) or [usb-device](https://docs.rs/usb-device/latest/usb_device/index.html))
  - [x] Serial port (with [uart_16550](https://github.com/rust-osdev/uart_16550))
  - [x] NIC E1000
  - [x] NIC RTL8139 (with [my fork](https://github.com/Julien-cpsn/rtl8139-rs) of [rtl8139-rs](https://github.com/vgarleanu/rtl8139-rs))
  - [x] xHCI (WIP, with [rust-osdev/xhci](https://docs.rs/xhci/latest/xhci/))
  - [x] APIC (with [this merge request](https://github.com/rust-osdev/bootloader/pull/460/files) but revisited)
  - [x] PCI (WIP with [pci_types](https://docs.rs/pci_types/0.10.0/pci_types/))
  - [x] PS2 Keyboard (with [pc_keyboard](https://github.com/rust-embedded-community/pc-keyboard))
  - [x] Legacy PIC (with [pic8259](https://github.com/rust-osdev/pic8259))
- **Commands**
  - [x] ip
    - [x] interface
      - [x] show
    - [x] address
      - [x] show
      - [x] add
      - [x] delete
      - [ ] modify
    - [x] route
      - [x] show
      - [x] add
      - [x] delete
      - [ ] modify
  - [x] ping (WIP)
  - [x] sleep
  - [x] top (WIP)
  - [x] scanpci
  - [x] lspci
  - [x] ps
  - [x] shutdown (with [qemu-exit](https://github.com/rust-embedded/qemu-exit))
  - [x] keyboard (change keyboard layout)
  - [x] uptime
  - [x] clear
  - [x] echo
- **Network**
  - [ ] SSH
  - [ ] Routing stack
  - [ ] Packet forwarding
  - [x] Host-side TCP/IP stack (with [smol-tcp](https://github.com/smoltcp-rs/smoltcp))
- **Memory**
  - [ ] More precise heap allocation
  - [x] Heap allocation (with [Talc](https://github.com/SFBdragon/talc))
  - [x] Memory pagination
- **Others**
  - [ ] multiboot2 (maybe need [multiboot2](https://github.com/rust-osdev/multiboot2), [doc](https://docs.rs/multiboot2/latest/multiboot2/))
  - [ ] Linux VM virtualization

## Contributors

- [@Julien-cpsn](https://github.com/Julien-cpsn) - Main contributor
- [@i5-650](https://github.com/i5-650) - Discussion & help

## License

This project is licensed under the MIT license and can be found [here](https://github.com/Julien-cpsn/RetOS/blob/main/LICENSE).