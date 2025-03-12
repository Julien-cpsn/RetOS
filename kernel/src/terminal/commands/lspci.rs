use crate::devices::pci::PCI_DEVICES;
use crate::printer::buffer::WRITER;
use crate::terminal::error::CliError;
use alloc::string::{String, ToString};
use alloc::{format, vec};
use goolog::set_target;

pub fn lspci() -> Result<(), CliError> {
    set_target!("LSPCI");

    let mut table = vec![
        [String::from("Bus:Device.Function"), String::from("Vendor"), String::from("Device name"), String::from("Revision"), String::from("Class"), String::from("Subclass")]
    ];

    for ((bus, device, function), pci_address) in PCI_DEVICES.read().iter() {
        let pci_device = pci_address.read();
        
        table.push([
            format!("{:02x}:{:02x}.{}", bus, device, function),
            pci_device.vendor_name.clone(),
            pci_device.device_name.clone(),
            pci_device.revision.to_string(),
            pci_device.class.to_string(),
            pci_device.subclass.to_string(),
        ])
    }

    let mut writer = WRITER.write();
    text_tables::render(&mut *writer, table).unwrap();

    Ok(())
}