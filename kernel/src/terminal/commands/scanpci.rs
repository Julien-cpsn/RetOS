use alloc::string::{ToString};
use crate::devices::pci::{parse_pci_subclass, PciClass, PciDevice, PCI_ACCESS, PCI_DEVICES};
use crate::terminal::error::CliError;
use goolog::{debug, set_target, trace};
use pci_types::{PciAddress, PciHeader};
use spin::RwLock;

pub fn scanpci() -> Result<(), CliError> {
    set_target!("SCANPCI");
    
    debug!("Locking PCI_DEVICES mutex...");
    PCI_DEVICES.write().clear();
    debug!("PCI_DEVICES mutex cleared and freed");

    trace!("Beginning PCI scan");
    for bus in 0..=255 {
        trace!("Bus {bus}");
        for device in 0..=31 {
            trace!("Device {device}");
            for function in 0..=7 {
                trace!("Function {function}");
                let pci_address = PciAddress::new(0, bus, device, function);
                let pci_header = PciHeader::new(pci_address);
                let (vendor_id, device_id) = pci_header.id(&PCI_ACCESS);

                // Invalid
                if vendor_id == 0xFFFF {
                    continue;
                }

                let (vendor_name, device_name) = find_pci_vendor_and_device(vendor_id, device_id);


                let (revision, class, subclass, _) = pci_header.revision_and_class(&PCI_ACCESS);

                let pci_class = PciClass::from_repr(class).expect("Unknown PCI class");
                let pci_subclass = parse_pci_subclass(&pci_class, subclass);
                
                let pci_device = PciDevice {
                    address: pci_address,
                    class: pci_class,
                    subclass: pci_subclass,
                    vendor_name: vendor_name.to_string(),
                    device_name: device_name.to_string(),
                    revision
                };
                
                debug!("Locking PCI_DEVICES mutex...");
                PCI_DEVICES.write().insert(
                    (bus, device, function),
                    RwLock::new(pci_device)
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
        _ => ("Unknown vendor", "Unknown device")
    }
}