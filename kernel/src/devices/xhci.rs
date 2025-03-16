use crate::devices::pci::PciClass::SerialBusController;
use crate::devices::pci::SerialBusController::UsbController;
use crate::devices::pci::{AnyPciSubclass, PciDevice, PCI_ACCESS};
use crate::memory::allocator::BOOT_INFO_FRAME_ALLOCATOR;
use crate::memory::tables::MAPPER;
use crate::println;
use alloc::sync::Arc;
use core::num::NonZeroUsize;
use core::ops::DerefMut;
use pci_types::{Bar, CommandRegister, EndpointHeader, PciHeader};
use spin::{Once, RwLock};
use x86_64::structures::paging::{Mapper, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};
use xhci::context::Device;
use xhci::extended_capabilities::List;
use xhci::registers::capability::CapabilityParameters1;
use xhci::ring::trb::event::TransferEvent;
use xhci::ring::trb::transfer::{EventData, SetupStage, TransferType};
use xhci::{accessor, ExtendedCapability, Registers};

pub static HOST_USB_CONTROLLER: Once<Arc<RwLock<HostController>>> = Once::new();

#[derive(Clone, Debug)]
pub struct MemoryMapper;

impl accessor::Mapper for MemoryMapper {
    unsafe fn map(&mut self, phys_base: usize, bytes: usize) -> NonZeroUsize {
        let mut mapper = MAPPER.get().unwrap().lock();
        let mut frame_allocator = BOOT_INFO_FRAME_ALLOCATOR.lock();

        let phys_addr = PhysAddr::new(phys_base as u64);
        let mut virt_addr = VirtAddr::new(mapper.phys_offset().as_u64() + phys_addr.as_u64());

        let start_frame: PhysFrame<Size4KiB> = PhysFrame::containing_address(phys_addr);
        let end_frame = PhysFrame::containing_address(phys_addr + bytes as u64 - 1);

        for frame in PhysFrame::range(start_frame, end_frame) {
            let page = Page::containing_address(virt_addr);
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

            let map_result = mapper.map_to(page, frame, flags, frame_allocator.deref_mut());
            map_result.expect("Memory mapping failed").flush();

            virt_addr += Size4KiB::SIZE;
        }

        NonZeroUsize::new(virt_addr.as_u64() as usize).expect("Virtual address cannot be zero")
    }

    fn unmap(&mut self, virt_base: usize, bytes: usize) {
        let virt_addr = VirtAddr::new(virt_base as u64);
        let mut mapper = MAPPER.get().unwrap().lock();

        let start_page: Page<Size4KiB> = Page::containing_address(virt_addr);
        let end_page = Page::containing_address(virt_addr + bytes as u64 - 1);

        for page in Page::range(start_page, end_page) {
            let res = mapper.unmap(page);
            res.expect("Unmapping failed").1.flush();
        }
    }
}

pub fn try_to_retrieve_xhci_registers(pci_device: &PciDevice) {
    // If the device is not a USB controller
    if pci_device.class != SerialBusController || pci_device.subclass != AnyPciSubclass::SerialBusController(UsbController) ||  pci_device.interface != 0x30 {
        return;
    }

    let pci_header = PciHeader::new(pci_device.address);
    let mut pci_endpoint_header = EndpointHeader::from_header(pci_header, &PCI_ACCESS).expect("Could not parse PCI endpoint header");


    pci_endpoint_header.update_command(&PCI_ACCESS, |_| CommandRegister::MEMORY_ENABLE);

    // BAR0
    let bar0 = pci_endpoint_header.bar(0, &PCI_ACCESS).expect("Could not retrieve BAR0");

    // I/O BARs are not supported by xHCI
    if let Bar::Io { .. } = bar0 {
        return
    };

    let (bar_address, _bar_size) = bar0.unwrap_mem();

    let mapper = MemoryMapper;
    HOST_USB_CONTROLLER.call_once(|| Arc::new(RwLock::new(HostController::new(bar_address, mapper))));
    let mut host_usb_controller = HOST_USB_CONTROLLER.get().unwrap().write();
    
    /* ------- */
    let xhci = &mut host_usb_controller.registers;

    let hcsparams1 = xhci.capability.hcsparams1.read_volatile();
    let num_ports = hcsparams1.number_of_ports();
    for i in 0..num_ports {
        let mut port = xhci.port_register_set.read_volatile_at(i as usize);
        if port.portsc.current_connect_status() {
            println!("Périphérique détecté sur le port {}", i + 1);

            // Reset the port
            port.portsc.set_port_reset();
            while !port.portsc.port_reset() {}
        }
    }

    xhci.operational.crcr.write_volatile(*xhci.operational.crcr.read_volatile().set_ring_cycle_state());
    println!("{}", xhci.operational.crcr.read_volatile().command_ring_running());

    let device = Device::new_32byte();

    // https://github.com/lihanrui2913/vmxkernel-os/blob/master/kernel/src/device/xhci.rs
    // https://github.com/sonhs99/RedOS/tree/master/kernel/src/device/xhc
}

fn get_descriptor() {
    let mut setup_data = SetupStage::new();
    setup_data.set_request_type(0x00 | TransferType::In as u8);
    setup_data.set_request(0x06);
    setup_data.set_value((0x01 << 8) | 0x00);
    setup_data.set_index(0);
    setup_data.set_length(18);

    let event_data = EventData::new();
    let transfer_event = TransferEvent::new();
}

pub struct HostController {
    registers: Registers<MemoryMapper>,
}

impl HostController {
    pub fn new(mmio_base: usize, mapper: MemoryMapper) -> HostController {
        let mut registers = unsafe { Registers::new(mmio_base, mapper.clone()) };

        HostController::request_host_controller_ownership(
            mmio_base,
            registers.capability.hccparams1.read_volatile(),
            mapper,
        );

        let mut usbcmd = registers.operational.usbcmd.read_volatile();
        usbcmd.clear_interrupter_enable();
        usbcmd.clear_host_system_error_enable();
        usbcmd.clear_enable_wrap_event();
        if !registers.operational.usbsts.read_volatile().hc_halted() {
            usbcmd.clear_run_stop();
        }
        registers.operational.usbcmd.write_volatile(usbcmd);
        while !registers.operational.usbsts.read_volatile().hc_halted() {}

        // Reset Controller
        let mut usbcmd = registers.operational.usbcmd.read_volatile();
        usbcmd.set_host_controller_reset();
        registers.operational.usbcmd.write_volatile(usbcmd);

        HostController {
            registers
        }
    }

    fn request_host_controller_ownership(mmio_base: usize, hccparams1: CapabilityParameters1, mapper: MemoryMapper) {
        let mut extended_capabilities = unsafe { List::new(mmio_base, hccparams1, mapper.clone()) }.expect("The xHC does not support the xHCI Extended Capability.");

        for extended_capability in &mut extended_capabilities {
            if extended_capability.is_err() {
                continue;
            }

            if let ExtendedCapability::UsbLegacySupport(mut usb) = extended_capability.unwrap() {
                let mut usblegsup = usb.usblegsup.read_volatile();
                if usblegsup.hc_os_owned_semaphore() {
                    return;
                }
                usblegsup.set_hc_os_owned_semaphore();
                usb.usblegsup.write_volatile(usblegsup);

                loop {
                    let leg_sup = usb.usblegsup.read_volatile();
                    if leg_sup.hc_bios_owned_semaphore() && !leg_sup.hc_os_owned_semaphore() {
                        break;
                    }
                }
            }
        }
    }
}