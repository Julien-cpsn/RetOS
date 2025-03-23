use alloc::string::{ToString};
use alloc::sync::Arc;
use crate::devices::pci::{parse_pci_subclass, AnyPciSubclass, NetworkController, PciClass, PciDevice, SerialBusController, PCI_ACCESS, PCI_DEVICES};
use crate::terminal::error::CliError;
use goolog::{debug, trace};
use pci_types::{PciAddress, PciHeader};
use spin::RwLock;
use crate::devices::drivers::e1000::{init_e1000, E1000_DEVICE_ID};
use crate::devices::drivers::rtl8139::{init_rtl8139, RTL8139_DEVICE_ID};
use crate::devices::xhci::try_to_retrieve_xhci_registers;

const GOOLOG_TARGET: &str = "SCANPCI";

pub fn scanpci() -> Result<(), CliError> {
    trace!("SCANPCI");
    
    debug!("Locking PCI_DEVICES mutex...");
    PCI_DEVICES.write().clear();
    debug!("PCI_DEVICES mutex cleared and freed");

    trace!("Beginning PCI scan");
    for bus in 0..=255 {
        for device in 0..=31 {
            for function in 0..=7 {
                let pci_address = PciAddress::new(0, bus, device, function);
                let pci_header = PciHeader::new(pci_address);
                let (vendor_id, device_id) = pci_header.id(&PCI_ACCESS);

                // Invalid
                if vendor_id == 0xFFFF {
                    continue;
                }

                debug!("Found Vendor: {:X}, Device: {:X}", vendor_id, device_id);
                
                let (vendor_name, device_name) = find_pci_vendor_and_device(vendor_id, device_id);

                let (revision, class, subclass, interface) = pci_header.revision_and_class(&PCI_ACCESS);

                let pci_class = PciClass::from_repr(class).expect("Unknown PCI class");
                let pci_subclass = parse_pci_subclass(&pci_class, subclass);
                
                let pci_device = PciDevice {
                    address: pci_address,
                    class: pci_class,
                    subclass: pci_subclass,
                    revision,
                    interface,
                    vendor_name: vendor_name.to_string(),
                    device_name: device_name.to_string(),
                };
                
                look_for_usable_device(&pci_device, device_id);
                
                debug!("Locking PCI_DEVICES mutex...");
                PCI_DEVICES.write().insert(
                    (bus, device, function),
                    Arc::new(RwLock::new(pci_device))
                );
                debug!("PCI_DEVICES mutex freed");
            }
        }
    }
    
    Ok(())
}

fn find_pci_vendor_and_device(vendor_id: u16, device_id: u16) -> (&'static str, &'static str) {
    match vendor_id {
        0x8086 => {
            let vendor = "Intel";
            let device = match device_id {
                _ if device_id == E1000_DEVICE_ID => "82540EM Gigabit Ethernet Controller",
                _ => "Unknown device"
            };

            (vendor, device)
        },
        0x1022 => {
            let vendor = "AMD";
            let device = match device_id {
                _ => "Unknown device"
            };

            (vendor, device)
        },
        0x106B => {
            let vendor = "Apple";
            let device = match device_id {
                _ => "Unknown device"
            };

            (vendor, device)
        },
        0x1AF4 => {
            let vendor = "VirtIO";
            let device = match device_id {
                _ => "Unknown device"
            };

            (vendor, device)
        },
        0x10DE => {
            let vendor = "NVIDIA";
            let device = match device_id {
                _ => "Unknown device"
            };

            (vendor, device)
        },
        0x1234 => {
            let vendor = "QEMU";
            let device = match device_id {
                0x1111 => "Standard VGA",
                _ => "Unknown device"
            };

            (vendor, device)
        },
        0x10EC => {
            let vendor = "Realtek Semiconductor Co., Ltd.";
            let device = match device_id {
                _ if device_id == RTL8139_DEVICE_ID => "RTL-8100/8101L/8139 PCI Fast Ethernet Adapter",
                _ => "Unknown device"
            };

            (vendor, device)
        }
        _ => ("Unknown vendor", "Unknown device")
    }
}

fn look_for_usable_device(pci_device: &PciDevice, device_id: u16) {
    match pci_device.class {
        PciClass::NetworkController => match &pci_device.subclass {
            AnyPciSubclass::NetworkController(network_controller) => match network_controller {
                // Ethernet controller
                NetworkController::EthernetController => match device_id {
                    _ if device_id == E1000_DEVICE_ID => init_e1000(&pci_device),
                    _ if device_id == RTL8139_DEVICE_ID => init_rtl8139(&pci_device),
                    _ => {}
                },
                _ => {}
            }
            _ => {}
        }
        PciClass::SerialBusController => match &pci_device.subclass  {
            AnyPciSubclass::SerialBusController(serial_bus_controller) => match serial_bus_controller {
                SerialBusController::UsbController => {
                    // xHCI/USB controller
                    if pci_device.interface == 0x30 {
                        try_to_retrieve_xhci_registers(&pci_device);
                    }
                }
                _ => {}
            }
            _ => {}
        },
        _ => {}
    }
}