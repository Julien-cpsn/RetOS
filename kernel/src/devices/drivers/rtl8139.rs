use crate::devices::network_controller::{register_network_controller, NetworkController, NETWORK_CONTROLLER_WAKER};
use crate::devices::pci::{PciDevice, PCI_ACCESS};
use crate::memory::tables::translate_addr;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::marker::PhantomData;
use crossbeam_queue::SegQueue;
use goolog::{debug};
use pci_types::{CommandRegister, EndpointHeader, PciHeader};
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;
use x86_64::VirtAddr;

const GOOLOG_TARGET: &str = "RTL8139";

pub const RTL8139_DEVICE_ID: u16 = 0x8139;

const RX_BUF_LEN: usize = 8192;
const RX_BUF_WRAP: usize = 1500; // Extra 1500 bytes with a WRAP mask for Rx
const RX_BUF_PAD: usize = 16;
const RX_BUF_LEN_WRAPPED: usize = RX_BUF_LEN + RX_BUF_PAD + RX_BUF_WRAP;

// Bit flags specific to the RCR
const APM: u32 = 0b10;
const AB: u32 = 0b1000;
const WRAP: u32 = 0b1000_0000;
const MXDMA_UNLIMITED: u32 = 0b111_0000_0000;
const RXFTH_NONE: u32 = 0b1110_0000_0000_0000;

// Bit flags specific to the CR
#[allow(unused)]
const RX_BUF_EMPTY: u8 = 0b1;
const TX_ENABLE: u8 = 0b100;
const RX_ENABLE: u8 = 0b1000;
const RST: u8 = 0b10000;

// Bit flags for IMR
const RX_OK: u16 = 0b1;
const RX_ERR: u16 = 0b10;
const TX_OK: u16 = 0b100;
const TX_ERR: u16 = 0b1000;
const RDU: u16 = 0b10000;
const TDU: u16 = 0b1000_0000;
const SYS_ERR: u16 = 0b1000_0000_0000_0000;

#[derive(Debug)]
pub struct RTL8139 {
    pub mac: [u8; 6],
    pub state: Rtl8139State,
    pub frames: SegQueue<Vec<u8>>,
    pub rx: PhantomData<()>,
    pub tx: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct Rtl8139State {
    pub config_1: Port<u32>,
    pub cmd_reg: Port<u8>,
    pub rbstart: Port<u32>,
    pub imr: Port<u16>,
    pub rcr: Port<u32>,
    #[allow(unused)]
    pub tppoll: Port<u8>,
    pub ack: Port<u16>,
    #[allow(unused)]
    pub cpcr: Port<u16>,
    pub capr: Port<u16>,

    // Registers holding our MAC bytes
    pub idr: [Port<u8>; 6],

    pub tx_dat: [Port<u32>; 4],
    pub tx_cmd: [Port<u32>; 4],
    pub tx_cursor: usize,

    pub buffer: Box<[u8; RX_BUF_LEN_WRAPPED]>,
    pub rx_cursor: usize,
}

pub fn init_rtl8139(pci_device: &PciDevice) {
    let pci_header = PciHeader::new(pci_device.address);
    let mut pci_endpoint_header = EndpointHeader::from_header(pci_header, &PCI_ACCESS).expect("Could not parse PCI endpoint header");

    let (_, line) = pci_endpoint_header.interrupt(&PCI_ACCESS);

    pci_endpoint_header.update_command(&PCI_ACCESS, |_| CommandRegister::BUS_MASTER_ENABLE | CommandRegister::IO_ENABLE);

    let bar0 = pci_endpoint_header.bar(0, &PCI_ACCESS).unwrap();
    let io_base = bar0.unwrap_io();

    let mut rtl8139 = unsafe { RTL8139::new(io_base as u16) };
    rtl8139.load();

    register_network_controller(NetworkController::RTL8139(Box::from(rtl8139)), line as u32);
}

impl RTL8139 {
    /// Function preloads the driver.
    /// # Arguments
    /// * `base` - base port for the device
    /// * `virt_to_phys` - function or closure that converts a virtaddr into a physaddr (used for
    /// DMA).
    /// # Safety
    /// The caller of this function must pass a valid port base for a valid PCI device. Furthermore
    /// the caller must ensure that mastering and interrupts for the PCI device are enabled.
    pub unsafe fn new(base: u16) -> Self {
        debug!("Preloading RTL8139");
        let inner = Rtl8139State {
            config_1: Port::new(base + 0x52),
            cmd_reg: Port::new(base + 0x37),
            rbstart: Port::new(base + 0x30),
            imr: Port::new(base + 0x3c),
            rcr: Port::new(base + 0x44),
            tppoll: Port::new(base + 0xd9),
            ack: Port::new(base + 0x3e),
            cpcr: Port::new(base + 0xe0),
            capr: Port::new(base + 0x38),

            idr: [
                Port::new(base + 0x00),
                Port::new(base + 0x01),
                Port::new(base + 0x02),
                Port::new(base + 0x03),
                Port::new(base + 0x04),
                Port::new(base + 0x05),
            ],

            tx_dat: [
                Port::new(base + 0x20),
                Port::new(base + 0x24),
                Port::new(base + 0x28),
                Port::new(base + 0x2c),
            ],
            tx_cmd: [
                Port::new(base + 0x10),
                Port::new(base + 0x14),
                Port::new(base + 0x18),
                Port::new(base + 0x1c),
            ],
            tx_cursor: 0,

            buffer: Box::from([0u8; RX_BUF_LEN_WRAPPED]),
            rx_cursor: 0,
        };
        debug!("RTL8139 preloaded");

        Self {
            mac: [0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
            state: inner,
            frames: SegQueue::new(),
            rx: PhantomData,
            tx: Vec::new(),
        }
    }

    /// Function sets up the device.
    pub fn load(&mut self) {
        debug!("Loading RTL8139");
        // Turn the device on by writing to config_1 then reset the device to clear all data in the
        // buffers by writing 0x10 to cmd_reg
        unsafe {
            self.state.config_1.write(0x0);
            self.state.cmd_reg.write(RST);
        }

        // Wait while the device resets
        loop {
            if (unsafe { self.state.cmd_reg.read() } & 0x10) == 0 {
                break;
            }
        }

        let raw_mac = self
            .state
            .idr
            .iter_mut()
            .map(|x| unsafe { x.read() })
            .collect::<Vec<u8>>();

        self.mac = raw_mac
            .as_slice()
            .try_into()
            .expect("rtl8139: failed to read mac");

        let buffer_virt = unsafe { VirtAddr::new_unsafe(self.state.buffer.as_ptr() as u64) };
        let buffer_ptr = translate_addr(buffer_virt).unwrap().as_u64() as u32;

        // Unsafe block specific for pre-launch NIC config
        unsafe {
            // Accept Physically Match packets
            // Accept Broadcast packets
            // Enable Max DMA burst
            // No RX Threshold
            self.state.rcr.write(APM | AB | MXDMA_UNLIMITED | RXFTH_NONE | WRAP);

            // Enable Tx on the CR register
            self.state.cmd_reg.write(RX_ENABLE | TX_ENABLE);

            // Write the PHYSICAL address of our Rx buffer to the NIC
            self.state.rbstart.write(buffer_ptr);
        }

        // Unsafe block specific to launch of NIC
        unsafe {
            // Enable Tx/Rx
            // NOTE: TX is technically already enabled but fuck it
            self.state.cmd_reg.write(RX_ENABLE | TX_ENABLE);

            // Mask only RxOk, TxOk, and some Err registers for internal book-keeping
            self.state.imr.write(0xffff | RX_OK | TX_OK | RX_ERR | TX_ERR | SYS_ERR | RDU | TDU);
        }

        debug!("RTL8139 loaded");
    }

    /// Function that we need to call when we receive an interrupt for this device.
    /// The callee must ensure that the PIC/APIC has received an EOI.
    pub fn on_interrupt(&mut self) {
        // At some point here we will want to also wake the network stack because there are packets
        // available.
        let isr = unsafe { self.state.ack.read() };

        if (isr & RX_OK) != 0 {
            while (unsafe { self.state.cmd_reg.read() } & RX_BUF_EMPTY) == 0 {
                self.frames.push(self.state.rok());
                NETWORK_CONTROLLER_WAKER.wake();
            }
        }

        unsafe {
            self.state.ack.write(isr);
        }
    }

    pub fn try_recv_sync(&mut self) -> Option<Vec<u8>> {
        self.frames.pop()
    }

    /// FIXME: Disable interrupts for the PCI device only instead of globally.
    pub fn send_sync(&mut self, buffer: &[u8]) {
        interrupts::without_interrupts(|| unsafe {
            self.state.write(buffer)
        });
    }
}

impl Rtl8139State {
    /// Function called on a ROK interrupt from the RTL8139 NIC, it parses the data written into
    /// the buffer as a ethernet frame and pushes it into our Vec.
    fn rok(&mut self) -> Vec<u8> {
        // A packet frame looks something like this
        // +--------------------------------------------+
        // | |     HEADER     |            |   DATA   | |
        // | +----------------+            +----------+ |
        // | |??|len = 2 bytes| = 4 bytes  |data = len| |
        // | +----------------+            +----------+ |
        // +--------------------------------------------+
        //
        // As per the diagram the packet structure is a 4 byte header where the last 2 bytes is the
        // length of the incoming data.
        // The length given also includes the length of the header itself.
        let buffer = &self.buffer[self.rx_cursor..];
        let length = u16::from_le_bytes(buffer[2..4].try_into().expect("Got wrong len")) as usize;

        // NOTE: The length in the header will never be less than 64, if a packet is received that
        //       has a length less than 64, the NIC will simply pad the packet with 0x00.
        assert!(
            length >= 64,
            "rtl8139: ROK Len is less than 64. THIS IS A BUG."
        );

        // NOTE: We are currently not zeroing out memory after a packet has been parsed and pushed.
        //       Are we sure that if packets with length less than 64 bytes will not contain
        //       remnants of the old packets?
        // If the frame is correctly parsed we push it into the queue, otherwise just skip it

        // NOTE: *** HACK ***
        // basically for some reason when receiving an `ACK` packet from a client as part of a
        // 3-way handshake the tcp packet is padded with extra zeroes at the end. My guessing is
        // that because the packet is less than 64 bytes the packet gets padded.
        let frame = buffer[4..length].to_vec(); // skip 4 bytes length and dont copy 4 bytes crc at the end.

        // Here we set the new index/cursor from where to read new packets, self.rx_cursor should
        // always point to the start of the header.
        // To calculate the new cursor we add the length of the previous frame which SHOULD include
        // the 4 bytes for the header, we also add 3 for 32 bit alignment and then mask the result.
        self.rx_cursor = (self.rx_cursor + length as usize + 4 + 3) & !3;

        if self.rx_cursor > RX_BUF_LEN {
            self.rx_cursor -= RX_BUF_LEN
        }

        unsafe {
            // The NIC is then informed of the new cursor. We remove 0x10 to avoid a overflow as
            // the NIC takes the padding into account I think.
            self.capr.write((self.rx_cursor - 0x10) as u16);
        }

        frame
    }

    /// # Safety
    /// The caller must make sure that interrupts are disabled before calling and are re-enabled
    /// after calling or the program will deadlock.
    pub unsafe fn write(&mut self, data: &[u8]) {
        // NOTE: Are we sure we absolutely need to disable interrupts? maybe we can bypass this
        //       with DMA.
        // We clone the inner PCI device to avoid a deadlock when we re-enable PCI interrupts for
        // this device

        // Disable interrupts for this PCI device to avoid deadlock to do with inner
        let cursor = self.tx_cursor;
        let data_ptr = translate_addr(VirtAddr::new_unsafe(data.as_ptr() as u64)).unwrap().as_u64();

        self.tx_dat[cursor].write(data_ptr as u32);
        self.tx_cmd[cursor].write((data.len() as u32) & 0xfff);

        loop {
            if (self.tx_cmd[cursor].read() & 0x8000) != 0 {
                break;
            }
        }

        self.tx_cursor = (cursor + 1) % 4;
    }
}