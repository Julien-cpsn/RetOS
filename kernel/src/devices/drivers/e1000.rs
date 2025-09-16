use crate::devices::mmio::MemoryMapper;
use crate::devices::pci::{PciDevice, PCI_ACCESS};
use crate::memory::tables::translate_addr;
use alloc::boxed::Box;
use alloc::{vec};
use alloc::sync::Arc;
use alloc::vec::Vec;
use accessor::Mapper;
use crossbeam_queue::SegQueue;
use goolog::{debug};
use pci_types::{CommandRegister, EndpointHeader, PciHeader};
use spin::Mutex;
use x86_64::instructions::interrupts;
use x86_64::VirtAddr;
use crate::devices::network::manager::NETWORK_MANAGER;

const GOOLOG_TARGET: &str = "E1000";

pub const E1000_DEVICE_ID: u16 = 0x100E;

// E1000 Register Offsets
const REG_CTRL: u16 = 0x0000;       // Device Control
const REG_STATUS: u16 = 0x0008;     // Device Status
const REG_EEPROM: u16 = 0x0014;     // EEPROM Read
#[allow(unused)]
const REG_CTRL_EXT: u16 = 0x0018;   // Extended Device Control

const REG_INTERRUPT_CAUSE: u16 = 0x00C0;  // Interrupt Cause Read
const REG_INTERRUPT_MASK: u16 = 0x00D0;   // Interrupt Mask Set/Read
#[allow(unused)]
const REG_INTERRUPT_MASK_CLEAR: u16 = 0x00D8;  // Interrupt Mask Clear

// MAC Address Registers
const REG_MAC_LOW: u16 = 0x5400;    // MAC address low 32 bits
const REG_MAC_HIGH: u16 = 0x5404;   // MAC address high 16 bits

// Transmit Registers
const REG_TCTL: u16 = 0x0400;       // Transmit Control
const REG_TIPG: u16 = 0x0410;       // Transmit Inter Packet Gap
const REG_TDBAL: u16 = 0x3800;      // TX Descriptor Base Address Low
const REG_TDBAH: u16 = 0x3804;      // TX Descriptor Base Address High
const REG_TDLEN: u16 = 0x3808;      // TX Descriptor Length
const REG_TDH: u16 = 0x3810;        // TX Descriptor Head
const REG_TDT: u16 = 0x3818;        // TX Descriptor Tail

// Receive Registers
const REG_RCTL: u16 = 0x0100;       // Receive Control
const REG_RDBAL: u16 = 0x2800;      // RX Descriptor Base Address Low
const REG_RDBAH: u16 = 0x2804;      // RX Descriptor Base Address High
const REG_RDLEN: u16 = 0x2808;      // RX Descriptor Length
const REG_RDH: u16 = 0x2810;        // RX Descriptor Head
const REG_RDT: u16 = 0x2818;        // RX Descriptor Tail
const REG_RAL0: u16 = 0x5400;       // Receive Address Low (0)
const REG_RAH0: u16 = 0x5404;       // Receive Address High (0)

// Control Register Bits
const CTRL_RESET: u32 = 1 << 26;    // Reset
const CTRL_SLU: u32 = 1 << 6;       // Set Link Up

// Transmit Control Register Bits
const TCTL_EN: u32 = 1 << 1;        // Transmit Enable
const TCTL_PSP: u32 = 1 << 3;       // Pad Short Packets
const TCTL_CT: u32 = 0x10 << 4;     // Collision Threshold
const TCTL_COLD: u32 = 0x40 << 12;  // Collision Distance

// Receive Control Register Bits
const RCTL_EN: u32 = 1 << 1;        // Receive Enable
const RCTL_SBP: u32 = 1 << 2;       // Store Bad Packets
#[allow(unused)]
const RCTL_UPE: u32 = 1 << 3;       // Unicast Promiscuous Enable
#[allow(unused)]
const RCTL_MPE: u32 = 1 << 4;       // Multicast Promiscuous Enable
#[allow(unused)]
const RCTL_LBM_NONE: u32 = 0 << 6;  // No Loopback
const RCTL_BAM: u32 = 1 << 15;      // Broadcast Accept Mode
const RCTL_SECRC: u32 = 1 << 26;    // Strip Ethernet CRC

// Buffer Sizes (RCTL_BSIZE value shifts)
#[allow(unused)]
const RCTL_BSIZE_256: u32 = 3 << 16;
#[allow(unused)]
const RCTL_BSIZE_512: u32 = 2 << 16;
#[allow(unused)]
const RCTL_BSIZE_1024: u32 = 1 << 16;
const RCTL_BSIZE_2048: u32 = 0 << 16;
#[allow(unused)]
const RCTL_BSIZE_4096: u32 = (3 << 16) | (1 << 25);
#[allow(unused)]
const RCTL_BSIZE_8192: u32 = (2 << 16) | (1 << 25);
#[allow(unused)]
const RCTL_BSIZE_16384: u32 = (1 << 16) | (1 << 25);

// Interrupt Bits
const INTERRUPT_TXDW: u32 = 1 << 0;  // Transmit Descriptor Written Back
const INTERRUPT_TXQE: u32 = 1 << 1;  // Transmit Queue Empty
const INTERRUPT_LSC: u32 = 1 << 2;   // Link Status Change
#[allow(unused)]
const INTERRUPT_RXSEQ: u32 = 1 << 3; // Receive Sequence Error
const INTERRUPT_RXDMT0: u32 = 1 << 4; // Receive Descriptor Minimum Threshold
const INTERRUPT_RXO: u32 = 1 << 6;   // Receiver Overrun
const INTERRUPT_RXT0: u32 = 1 << 7;  // Receiver Timer Interrupt

// We'll use these interrupts for our driver
const INTERRUPT_MASK: u32 = INTERRUPT_LSC | INTERRUPT_RXT0 | INTERRUPT_RXDMT0 | INTERRUPT_RXO | INTERRUPT_TXDW | INTERRUPT_TXQE;

// Transmit Descriptor Bits
const TX_DESC_CMD_EOP: u8 = 1 << 0;  // End of Packet
const TX_DESC_CMD_IFCS: u8 = 1 << 1; // Insert FCS (CRC)
const TX_DESC_CMD_RS: u8 = 1 << 3;   // Report Status
const TX_DESC_STATUS_DD: u8 = 1 << 0; // Descriptor Done

// Receive Descriptor Bits
const RX_DESC_STATUS_DD: u8 = 1 << 0; // Descriptor Done
const RX_DESC_STATUS_EOP: u8 = 1 << 1; // End of Packet

// Number of descriptors
const RX_DESCRIPTORS: usize = 32;
const TX_DESCRIPTORS: usize = 32;

// Packet buffer size - must align with RCTL_BSIZE setting
pub const RX_BUFFER_SIZE: usize = 2048;
pub const TX_BUFFER_SIZE: usize = 2048;

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone)]
pub struct TxDescriptor {
    buffer_addr: u64,     // Buffer address
    length: u16,          // Data buffer length
    cso: u8,              // Checksum offset
    cmd: u8,              // Command
    status: u8,           // Status
    css: u8,              // Checksum start
    special: u16,         // Special field
}

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone)]
pub struct RxDescriptor {
    buffer_addr: u64,     // Buffer address
    length: u16,          // Data length
    checksum: u16,        // Checksum
    status: u8,           // Status
    errors: u8,           // Errors
    special: u16,         // Special field
}

#[derive(Debug)]
pub struct E1000 {
    pub mac: [u8; 6],
    state: E1000State,
    frames: SegQueue<Vec<u8>>
}

#[derive(Debug)]
struct E1000State {
    pub mmio_base: u64,
    pub rx: Mutex<E1000Rx>,
    pub tx: Mutex<E1000Tx>,
}

#[derive(Debug)]
pub struct E1000Rx {
    pub rx_descriptors: Box<[RxDescriptor; RX_DESCRIPTORS]>,
    pub rx_buffers: Box<Vec<[u8; RX_BUFFER_SIZE]>>,
    pub rx_cursor: usize,
}

#[derive(Debug)]
pub struct E1000Tx {
    pub tx_descriptors: Box<[TxDescriptor; TX_DESCRIPTORS]>,
    pub tx_buffers: Box<Vec<[u8; TX_BUFFER_SIZE]>>,
    pub tx_cursor: usize,
}

pub fn init_e1000(pci_device: &PciDevice) {
    let pci_header = PciHeader::new(pci_device.address);
    let mut pci_endpoint_header = EndpointHeader::from_header(pci_header, &PCI_ACCESS).expect("Could not parse PCI endpoint header");

    let (_, line) = pci_endpoint_header.interrupt(&PCI_ACCESS);

    // Enable bus mastering and memory space
    pci_endpoint_header.update_command(&PCI_ACCESS, |_| CommandRegister::BUS_MASTER_ENABLE | CommandRegister::MEMORY_ENABLE);

    // Get the BAR0 which contains the memory-mapped I/O base address
    let bar0 = pci_endpoint_header.bar(0, &PCI_ACCESS).unwrap();
    let (mmio_base, mmio_size) = bar0.unwrap_mem();

    let mut mapper = MemoryMapper;
    let virt_mmio_base = unsafe { mapper.map(mmio_base, mmio_size).get() };

    let mut e1000 = E1000::new(virt_mmio_base as u64);
    e1000.init();

    NETWORK_MANAGER
        .lock()
        .register_device(line, Arc::new(Mutex::new(e1000)));
}

impl E1000 {
    pub fn new(mmio_base: u64) -> Self {
        debug!("Creating E1000 driver");

        // Create TX and RX descriptor rings and buffers
        let tx_descriptors = [TxDescriptor {
            buffer_addr: 0,
            length: 0,
            cso: 0,
            cmd: 0,
            status: 0,
            css: 0,
            special: 0,
        }; TX_DESCRIPTORS];

        let rx_descriptors = [RxDescriptor {
            buffer_addr: 0,
            length: 0,
            checksum: 0,
            status: 0,
            errors: 0,
            special: 0,
        }; RX_DESCRIPTORS];

        let tx_buffers = vec![[0u8; TX_BUFFER_SIZE]; TX_DESCRIPTORS];
        let rx_buffers = vec![[0u8; RX_BUFFER_SIZE]; RX_DESCRIPTORS];

        let state = E1000State {
            mmio_base,
            rx: Mutex::new(E1000Rx {
                rx_descriptors: Box::from(rx_descriptors),
                rx_buffers: Box::from(rx_buffers),
                rx_cursor: 0,
            }),
            tx: Mutex::new(E1000Tx {
                tx_descriptors: Box::from(tx_descriptors),
                tx_buffers: Box::from(tx_buffers),
                tx_cursor: 0,
            })
        };

        Self {
            mac: [0; 6],
            state,
            frames: SegQueue::new(),
        }
    }

    pub fn init(&mut self) {
        debug!("Initializing E1000");

        // Reset device
        self.write_register(REG_CTRL, self.read_register(REG_CTRL) | CTRL_RESET);

        // Wait for reset to complete
        while (self.read_register(REG_CTRL) & CTRL_RESET) != 0 {
            core::hint::spin_loop();
        }

        // Link Up
        self.write_register(REG_CTRL, self.read_register(REG_CTRL) | CTRL_SLU);

        // Read MAC address from device
        self.read_mac_address();

        // Initialize rx/tx descriptors
        self.setup_rx_descriptors();
        self.setup_tx_descriptors();

        // Configure transmit and receive
        self.configure_tx();
        self.configure_rx();

        // Enable interrupts
        self.enable_interrupts();

        debug!("E1000 initialized");
    }

    fn read_register(&self, offset: u16) -> u32 {
        unsafe {
            core::ptr::read_volatile((self.state.mmio_base + offset as u64) as *const u32)
        }
    }

    fn write_register(&self, offset: u16, value: u32) {
        unsafe {
            core::ptr::write_volatile((self.state.mmio_base + offset as u64) as *mut u32, value);
        }
    }

    fn read_mac_address(&mut self) {
        // Read MAC address from device-specific location
        // This could be from the RAL/RAH registers or from EEPROM depending on the device

        // First, check if we can read from EEPROM
        let mut has_eeprom = false;
        self.write_register(REG_EEPROM, 0x1);

        for _ in 0..1000 {
            let val = self.read_register(REG_EEPROM);
            if (val & (1 << 4)) != 0 {
                has_eeprom = true;
                break;
            }
        }

        if has_eeprom {
            // Read from EEPROM (implementation would depend on specific E1000 variant)

            // This is a simplification - actual EEPROM reading would be more complex
            let low = self.read_register(REG_MAC_LOW);
            let high = self.read_register(REG_MAC_HIGH);

            self.mac[0] = (low & 0xFF) as u8;
            self.mac[1] = ((low >> 8) & 0xFF) as u8;
            self.mac[2] = ((low >> 16) & 0xFF) as u8;
            self.mac[3] = ((low >> 24) & 0xFF) as u8;
            self.mac[4] = (high & 0xFF) as u8;
            self.mac[5] = ((high >> 8) & 0xFF) as u8;
        } else {
            // Read from RAL0/RAH0 registers which may be pre-programmed

            let low = self.read_register(REG_RAL0);
            let high = self.read_register(REG_RAH0);

            self.mac[0] = (low & 0xFF) as u8;
            self.mac[1] = ((low >> 8) & 0xFF) as u8;
            self.mac[2] = ((low >> 16) & 0xFF) as u8;
            self.mac[3] = ((low >> 24) & 0xFF) as u8;
            self.mac[4] = (high & 0xFF) as u8;
            self.mac[5] = ((high >> 8) & 0xFF) as u8;
        }

        // Set the MAC address in the device registers
        self.write_register(REG_RAL0, (self.mac[3] as u32) << 24 | (self.mac[2] as u32) << 16 | (self.mac[1] as u32) << 8 | (self.mac[0] as u32));
        self.write_register(REG_RAH0, (1 << 31) | (self.mac[5] as u32) << 8 | (self.mac[4] as u32));
    }

    fn setup_rx_descriptors(&self) {
        let mut rx = self.state.rx.lock();
        
        // Initialize rx descriptors and link them to buffers
        for i in 0..RX_DESCRIPTORS {
            let buffer_addr = VirtAddr::new(rx.rx_buffers[i].as_ptr() as u64);
            let phys_addr = translate_addr(buffer_addr).unwrap().as_u64();

            rx.rx_descriptors[i] = RxDescriptor {
                buffer_addr: phys_addr,
                length: 0,
                checksum: 0,
                status: 0,
                errors: 0,
                special: 0,
            };
        }

        // Setup the registers for the rx ring
        let desc_addr = VirtAddr::new(rx.rx_descriptors.as_ptr() as u64);
        let phys_addr = translate_addr(desc_addr).unwrap().as_u64();

        // Set the descriptor base address
        self.write_register(REG_RDBAL, phys_addr as u32);
        self.write_register(REG_RDBAH, (phys_addr >> 32) as u32);

        // Set the descriptor ring length (in bytes)
        self.write_register(REG_RDLEN, (RX_DESCRIPTORS * size_of::<RxDescriptor>()) as u32);

        // Initialize head and tail pointers
        self.write_register(REG_RDH, 0);
        // Set tail to size-1 so that we have descriptors available immediately
        self.write_register(REG_RDT, (RX_DESCRIPTORS - 1) as u32);

        // Start with the first descriptor
        rx.rx_cursor = 0;
    }

    fn setup_tx_descriptors(&mut self) {
        let mut tx = self.state.tx.lock();
        
        // Initialize tx descriptors and link them to buffers
        for i in 0..TX_DESCRIPTORS {
            let buffer_addr = VirtAddr::new(tx.tx_buffers[i].as_ptr() as u64);
            let phys_addr = translate_addr(buffer_addr).unwrap().as_u64();

            tx.tx_descriptors[i] = TxDescriptor {
                buffer_addr: phys_addr,
                length: 0,
                cso: 0,
                cmd: 0,
                status: 0,
                css: 0,
                special: 0,
            };
        }

        // Setup the registers for the tx ring
        let desc_addr = VirtAddr::new(tx.tx_descriptors.as_ptr() as u64);
        let phys_addr = translate_addr(desc_addr).unwrap().as_u64();

        // Set the descriptor base address
        self.write_register(REG_TDBAL, phys_addr as u32);
        self.write_register(REG_TDBAH, (phys_addr >> 32) as u32);

        // Set the descriptor ring length (in bytes)
        self.write_register(REG_TDLEN, (TX_DESCRIPTORS * core::mem::size_of::<TxDescriptor>()) as u32);

        // Initialize head and tail pointers
        self.write_register(REG_TDH, 0);
        self.write_register(REG_TDT, 0);

        // Start with the first descriptor
        tx.tx_cursor = 0;
    }

    fn configure_tx(&self) {
        // Setup transmit control register
        let tctl = TCTL_EN | TCTL_PSP | TCTL_CT | TCTL_COLD;
        self.write_register(REG_TCTL, tctl);

        // Set inter-packet gap timing
        self.write_register(REG_TIPG, 0x0060200A); // Standard values for fiber/copper
    }

    fn configure_rx(&self) {
        // Setup receive control register
        let rctl = RCTL_EN | RCTL_SBP | RCTL_BAM | RCTL_SECRC | RCTL_BSIZE_2048;
        self.write_register(REG_RCTL, rctl);
    }

    fn enable_interrupts(&self) {
        // Clear any pending interrupts first
        let icr = self.read_register(REG_INTERRUPT_CAUSE);
        if icr != 0 {
            self.write_register(REG_INTERRUPT_CAUSE, icr);
        }

        // Enable interrupts (set mask bits)
        let old_mask = self.read_register(REG_INTERRUPT_MASK);
        self.write_register(REG_INTERRUPT_MASK, old_mask | INTERRUPT_MASK);
    }

    /// Function that handles interrupts from the E1000
    pub fn on_interrupt(&self) -> bool {
        // Read the interrupt cause register
        let interrupt_cause = self.read_register(REG_INTERRUPT_CAUSE);

        if interrupt_cause == 0 {
            return false;
        }

        // Clear interrupts by writing back the value
        self.write_register(REG_INTERRUPT_CAUSE, interrupt_cause);

        // Handle link status change
        if (interrupt_cause & INTERRUPT_LSC) != 0 {
            let status = self.read_register(REG_STATUS);
            if (status & (1 << 1)) != 0 {
                let ctrl = self.read_register(REG_CTRL);
                if (ctrl & CTRL_SLU) == 0 {
                    self.write_register(REG_CTRL, ctrl | CTRL_SLU);
                }
            }
        }

        // Handle receive interrupts
        if (interrupt_cause & (INTERRUPT_RXT0 | INTERRUPT_RXDMT0 | INTERRUPT_RXO)) != 0 {
            //println!("inter rx");

            if (interrupt_cause & INTERRUPT_RXO) != 0 {
                // Implement recovery logic for overruns
                // Reset RX if needed
                self.reset_rx_ring();
            }

            self.process_rx_packets();
        }

        // Handle transmit interrupts
        if (interrupt_cause & (INTERRUPT_TXDW | INTERRUPT_TXQE)) != 0 {
            //println!("inter tx");

            let mut tx = self.state.tx.lock();
            let mut tx_processed = 0;
            
            // Check for completed transmissions
            for i in 0..TX_DESCRIPTORS {
                if (tx.tx_descriptors[i].status & TX_DESC_STATUS_DD) == 0 {
                    continue;
                }

                // Free any associated buffers if needed
                tx.tx_buffers[i] = [0; TX_BUFFER_SIZE];

                // Reset descriptor status
                tx.tx_descriptors[i].status = 0;
                tx.tx_descriptors[i].cmd = 0;
                tx.tx_descriptors[i].length = 0;
                tx_processed += 1;
            }

            // Wake any tasks waiting on transmit completion
            if tx_processed > 0 {
                
            }
        }

        return true;
    }

    fn reset_rx_ring(&self) {
        // Disable receive
        self.write_register(REG_RCTL, 0);

        // Reinitialize RX descriptors
        self.setup_rx_descriptors();

        // Re-enable receive
        let rctl = RCTL_EN | RCTL_SBP | RCTL_BAM | RCTL_SECRC | RCTL_BSIZE_2048;
        self.write_register(REG_RCTL, rctl);
    }

    fn process_rx_packets(&self) {
        let mut rx = self.state.rx.lock();
        
        // Process received packets
        let mut i = rx.rx_cursor;
        let mut processed_packets = 0;
        let max_packets = RX_DESCRIPTORS;

        loop {
            // Check if descriptor is done
            let desc = &mut rx.rx_descriptors[i];

            if (desc.status & RX_DESC_STATUS_DD) == 0 || processed_packets >= max_packets {
                // Not ready or processed too many packets
                break;
            }

            if (desc.status & RX_DESC_STATUS_EOP) == 0 {
                // Not end of packet - we don't handle multi-descriptor packets yet

                // Reset the descriptor
                desc.status = 0;

                // Move to next descriptor
                i = (i + 1) % RX_DESCRIPTORS;
                continue;
            }

            // Reset the descriptor
            desc.status = 0;
            // Indicate that the descriptor is available
            self.write_register(REG_RDT, i as u32);

            // Should be done before the descriptor reset but then rx cannot be borrowed as mutable twice
            // We have a complete packet
            let length = desc.length as usize;
            if length > 0 {
                // Copy the packet to a new buffer
                let packet = rx.rx_buffers[i][0..length].to_vec();
                self.frames.push(packet);
            }

            // Update tail pointer
            let new_tail = (i + 1) % RX_DESCRIPTORS;

            // Move to next descriptor
            i = new_tail;
            processed_packets += 1;
        }

        // Update our position in the ring
        rx.rx_cursor = i;
    }

    /// Send a packet
    pub fn send_sync(&self, buffer: &[u8]) {
        interrupts::without_interrupts(|| {
            let mut tx = self.state.tx.lock();
            
            let i = tx.tx_cursor;

            // Wait for the descriptor to be free
            #[allow(clippy::while_immutable_condition)]
            while (tx.tx_descriptors[i].status & TX_DESC_STATUS_DD) == 0 && tx.tx_descriptors[i].cmd != 0 {
                core::hint::spin_loop();
            }

            // Copy the data to the transmit buffer
            let len = core::cmp::min(buffer.len(), TX_BUFFER_SIZE);
            tx.tx_buffers[i][0..len].copy_from_slice(&buffer[0..len]);

            // Setup the descriptor
            tx.tx_descriptors[i].length = len as u16;
            tx.tx_descriptors[i].cmd = TX_DESC_CMD_EOP | TX_DESC_CMD_IFCS | TX_DESC_CMD_RS;
            tx.tx_descriptors[i].status = 0;

            // Update the tail pointer to start transmission
            let new_cursor = (i + 1) % TX_DESCRIPTORS;
            self.write_register(REG_TDT, new_cursor as u32);

            // Update our position in the ring
            tx.tx_cursor = new_cursor;
        });
    }

    pub fn recv_sync(&mut self) -> Option<Vec<u8>> {
        self.frames.pop()
    }
}