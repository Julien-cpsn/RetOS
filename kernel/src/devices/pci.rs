use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use core::fmt::{Display, Formatter};
use pci_types::{ConfigRegionAccess, PciAddress};
use spin::RwLock;
use strum::{Display, FromRepr};
use x86_64::instructions::port::Port;

const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

pub static PCI_ACCESS: PciAccess = PciAccess;
pub static PCI_DEVICES: RwLock<BTreeMap<(u8, u8, u8), Arc<RwLock<PciDevice>>>> = RwLock::new(BTreeMap::new());

pub struct PciAccess;

pub struct PciDevice {
    pub address: PciAddress,
    pub class: PciClass,
    pub subclass: AnyPciSubclass,
    pub revision: u8,
    pub interface: u8,
    pub vendor_name: String,
    pub device_name: String,
}

impl ConfigRegionAccess for PciAccess {
    unsafe fn read(&self, address: PciAddress, offset: u16) -> u32 {
        let address = 0x80000000
            | ((address.bus() as u32) << 16)
            | ((address.device() as u32) << 11)
            | ((address.function() as u32) << 8)
            | ((offset as u32) & 0xFC);

        unsafe {
            let mut outl = Port::new(PCI_CONFIG_ADDRESS);
            outl.write(address);
            let mut inl = Port::new(PCI_CONFIG_DATA);
            inl.read()
        }
    }

    unsafe fn write(&self, address: PciAddress, offset: u16, value: u32) {
        let address = 0x80000000
            | ((address.bus() as u32) << 16)
            | ((address.device() as u32) << 11)
            | ((address.function() as u32) << 8)
            | ((offset as u32) & 0xFC);

        unsafe {
            let mut outl = Port::new(PCI_CONFIG_ADDRESS);
            outl.write(address);
            let mut outl = Port::new(PCI_CONFIG_DATA);
            outl.write(value);
        }
    }
}

pub fn parse_pci_subclass(class: &PciClass, subclass: u8) -> AnyPciSubclass {
    let result = match class {
        PciClass::DevicesBuiltBeforeClassCodes => DevicesBuiltBeforeClassCodes::from_repr(subclass).map(AnyPciSubclass::DevicesBuiltBeforeClassCodes),
        PciClass::MassStorageController => Some(AnyPciSubclass::MassStorageController(MassStorageController::from_repr(subclass).unwrap_or_default())),
        PciClass::NetworkController => Some(AnyPciSubclass::NetworkController(NetworkController::from_repr(subclass).unwrap_or_default())),
        PciClass::DisplayController => Some(AnyPciSubclass::DisplayController(DisplayController::from_repr(subclass).unwrap_or_default())),
        PciClass::MultimediaDevice => Some(AnyPciSubclass::MultimediaController(MultimediaController::from_repr(subclass).unwrap_or_default())),
        PciClass::MemoryController => Some(AnyPciSubclass::MemoryController(MemoryController::from_repr(subclass).unwrap_or_default())),
        PciClass::Bridge => Some(AnyPciSubclass::Bridge(Bridge::from_repr(subclass).unwrap_or_default())),
        PciClass::CommunicationsController => Some(AnyPciSubclass::CommunicationsController(CommunicationController::from_repr(subclass).unwrap_or_default())),
        PciClass::GenericSystemPeripheral => Some(AnyPciSubclass::GenericSystemPeripheral(GenericSystemPeripheral::from_repr(subclass).unwrap_or_default())),
        PciClass::InputDevice => Some(AnyPciSubclass::InputDeviceController(InputDeviceController::from_repr(subclass).unwrap_or_default())),
        PciClass::DockingStation => Some(AnyPciSubclass::DockingStation(DockingStation::from_repr(subclass).unwrap_or_default())),
        PciClass::Processor => Processor::from_repr(subclass).map(AnyPciSubclass::Processor),
        PciClass::SerialBusController => Some(AnyPciSubclass::SerialBusController(SerialBusController::from_repr(subclass).unwrap_or_default())),
        PciClass::WirelessController => WirelessController::from_repr(subclass).map(AnyPciSubclass::WirelessController),
        PciClass::IntelligentController => match subclass {
            0x00 => Some(AnyPciSubclass::IntelligentController),
            _ => None
        }
        PciClass::SatelliteCommunicationsController => SatelliteCommunicationsController::from_repr(subclass).map(AnyPciSubclass::SatelliteCommunicationsController),
        PciClass::EncryptionController => Some(AnyPciSubclass::EncryptionController(EncryptionController::from_repr(subclass).unwrap_or_default())),
        PciClass::SignalProcessingController => Some(AnyPciSubclass::SignalProcessingController(SignalProcessingController::from_repr(subclass).unwrap_or_default())),
        PciClass::ProcessingAccelerators => Some(AnyPciSubclass::ProcessingAccelerators(ProcessingAccelerators::from_repr(subclass).unwrap_or_default())),
        PciClass::NonEssentialInstrumentation => Some(AnyPciSubclass::NonEssentialInstrumentation),
        PciClass::CoProcessor => Some(AnyPciSubclass::CoProcessor),
        PciClass::UnassignedClass => Some(AnyPciSubclass::UnassignedClass)
    };

    result.unwrap_or(AnyPciSubclass::UnassignedClass)
}

#[derive(Debug, Display, FromRepr, PartialEq)]
#[repr(u8)]
pub enum PciClass {
    DevicesBuiltBeforeClassCodes = 0x00,
    MassStorageController = 0x01,
    NetworkController = 0x02,
    DisplayController = 0x03,
    MultimediaDevice = 0x04,
    MemoryController = 0x05,
    Bridge = 0x06,
    CommunicationsController = 0x07,
    GenericSystemPeripheral = 0x08,
    InputDevice = 0x09,
    DockingStation = 0x0A,
    Processor = 0x0B,
    SerialBusController = 0x0C,
    WirelessController = 0x0D,
    IntelligentController = 0x0E,
    SatelliteCommunicationsController = 0x0F,
    EncryptionController = 0x10,
    SignalProcessingController = 0x11,
    ProcessingAccelerators = 0x12,
    NonEssentialInstrumentation = 0x13,
    CoProcessor = 0x40,
    UnassignedClass = 0xFF,
}

#[derive(Debug, PartialEq)]
pub enum AnyPciSubclass {
    DevicesBuiltBeforeClassCodes(DevicesBuiltBeforeClassCodes),
    MassStorageController(MassStorageController),
    NetworkController(NetworkController),
    DisplayController(DisplayController),
    MultimediaController(MultimediaController),
    MemoryController(MemoryController),
    Bridge(Bridge),
    CommunicationsController(CommunicationController),
    GenericSystemPeripheral(GenericSystemPeripheral),
    InputDeviceController(InputDeviceController),
    DockingStation(DockingStation),
    Processor(Processor),
    SerialBusController(SerialBusController),
    WirelessController(WirelessController),
    IntelligentController,
    SatelliteCommunicationsController(SatelliteCommunicationsController),
    EncryptionController(EncryptionController),
    SignalProcessingController(SignalProcessingController),
    ProcessingAccelerators(ProcessingAccelerators),
    NonEssentialInstrumentation,
    CoProcessor,
    UnassignedClass
}

impl Display for AnyPciSubclass {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            AnyPciSubclass::DevicesBuiltBeforeClassCodes(subclass) => f.write_str(&subclass.to_string()),
            AnyPciSubclass::MassStorageController(subclass) => f.write_str(&subclass.to_string()),
            AnyPciSubclass::NetworkController(subclass) => f.write_str(&subclass.to_string()),
            AnyPciSubclass::DisplayController(subclass) => f.write_str(&subclass.to_string()),
            AnyPciSubclass::MultimediaController(subclass) => f.write_str(&subclass.to_string()),
            AnyPciSubclass::MemoryController(subclass) => f.write_str(&subclass.to_string()),
            AnyPciSubclass::Bridge(subclass) => f.write_str(&subclass.to_string()),
            AnyPciSubclass::CommunicationsController(subclass) => f.write_str(&subclass.to_string()),
            AnyPciSubclass::GenericSystemPeripheral(subclass) => f.write_str(&subclass.to_string()),
            AnyPciSubclass::InputDeviceController(subclass) => f.write_str(&subclass.to_string()),
            AnyPciSubclass::DockingStation(subclass) => f.write_str(&subclass.to_string()),
            AnyPciSubclass::Processor(subclass) => f.write_str(&subclass.to_string()),
            AnyPciSubclass::SerialBusController(subclass) => f.write_str(&subclass.to_string()),
            AnyPciSubclass::WirelessController(subclass) => f.write_str(&subclass.to_string()),
            AnyPciSubclass::SatelliteCommunicationsController(subclass) => f.write_str(&subclass.to_string()),
            AnyPciSubclass::EncryptionController(subclass) => f.write_str(&subclass.to_string()),
            AnyPciSubclass::SignalProcessingController(subclass) => f.write_str(&subclass.to_string()),
            AnyPciSubclass::ProcessingAccelerators(subclass) => f.write_str(&subclass.to_string()),
            AnyPciSubclass::NonEssentialInstrumentation => f.write_str("NonEssentialInstrumentation"),
            AnyPciSubclass::IntelligentController => f.write_str("IntelligentController"),
            AnyPciSubclass::CoProcessor => f.write_str("CoProcessor"),
            AnyPciSubclass::UnassignedClass => f.write_str("UnassignedClass"),
        }
    }
}

#[derive(Debug, Display, FromRepr, PartialEq)]
#[repr(u8)]
pub enum DevicesBuiltBeforeClassCodes {
    NonVgaUnclassifiedDevice = 0x00,
    VgaCompatibleUnclassifiedDevice = 0x01,
    ImageCoprocessor = 0x05
}

#[derive(Default, Debug, Display, FromRepr, PartialEq)]
#[repr(u8)]
pub enum MassStorageController {
    ScsiStorageController = 0x00,
    IdeInterface = 0x01,
    FloppyDiskController = 0x02,
    IpiBusController = 0x03,
    RaidBusController = 0x04,
    AtaController = 0x05,
    SataController = 0x06,
    SerialAttachedScsiController = 0x07,
    NonVolatileMemoryController = 0x08,
    UniversalFlashStorageController = 0x09,
    #[default]
    MassStorageController = 0x80
}

#[derive(Default, Debug, Display, FromRepr, PartialEq)]
#[repr(u8)]
pub enum NetworkController {
    EthernetController = 0x00,
    TokenRingNetworkController = 0x01,
    FddiNetworkController = 0x02,
    AtmNetworkController = 0x03,
    IsdnController = 0x04,
    WorldFipController = 0x05,
    PicmgController = 0x06,
    InfinibandController = 0x07,
    FabricController = 0x08,
    #[default]
    NetworkController = 0x80,
}

#[derive(Default, Debug, Display, FromRepr, PartialEq)]
#[repr(u8)]
pub enum DisplayController {
    VgaCompatibleController = 0x00,
    XgaCompatibleController = 0x01,
    ThreeDController = 0x02,
    #[default]
    DisplayController = 0x80,
}

#[derive(Default, Debug, Display, FromRepr, PartialEq)]
#[repr(u8)]
pub enum MultimediaController {
    MultimediaVideoController = 0x00,
    MultimediaAudioController = 0x01,
    ComputerTelephonyDevice = 0x02,
    AudioDevice = 0x03,
    #[default]
    MultimediaController = 0x80,
}

#[derive(Default, Debug, Display, FromRepr, PartialEq)]
#[repr(u8)]
pub enum MemoryController {
    RamMemory = 0x00,
    FlashMemory = 0x01,
    Cxl = 0x02,
    #[default]
    MemoryController = 0x80,
}

#[derive(Default, Debug, Display, FromRepr, PartialEq)]
#[repr(u8)]
pub enum Bridge {
    HostBridge = 0x00,
    IsaBridge = 0x01,
    EisaBridge = 0x02,
    MicroChannelBridge = 0x03,
    PciBridge = 0x04,
    PcmciaBridge = 0x05,
    NuBusBridge = 0x06,
    CardBusBridge = 0x07,
    RaceWayBridge = 0x08,
    SemiTransparentPciToPciBridge = 0x09,
    InfiniBandToPciHostBridge = 0x0A,
    #[default]
    Bridge = 0x80,
}

#[derive(Default, Debug, Display, FromRepr, PartialEq)]
#[repr(u8)]
pub enum CommunicationController {
    SerialController = 0x00,
    ParallelController = 0x01,
    MultiportSerialController = 0x02,
    Modem = 0x03,
    GpibController = 0x04,
    SmartCardController = 0x05,
    #[default]
    CommunicationController = 0x80,
}

#[derive(Default, Debug, Display, FromRepr, PartialEq)]
#[repr(u8)]
pub enum GenericSystemPeripheral {
    Pic = 0x00,
    DmaController = 0x01,
    Timer = 0x02,
    Rtc = 0x03,
    PciHotPlugController = 0x04,
    SdHostController = 0x05,
    Iommu = 0x06,
    #[default]
    SystemPeripheral = 0x80,
    TimingCard = 0x99,
}

#[derive(Default, Debug, Display, FromRepr, PartialEq)]
#[repr(u8)]
pub enum InputDeviceController {
    KeyboardController = 0x00,
    DigitizerPen = 0x01,
    MouseController = 0x02,
    ScannerController = 0x03,
    GameportController = 0x04,
    #[default]
    InputDeviceController = 0x80,
}

#[derive(Default, Debug, Display, FromRepr, PartialEq)]
#[repr(u8)]
pub enum DockingStation {
    GenericDockingStation = 0x00,
    #[default]
    DockingStation = 0x80,
}

#[derive(Debug, Display, FromRepr, PartialEq)]
#[repr(u8)]
pub enum Processor {
    I386 = 0x00,
    I486 = 0x01,
    Pentium = 0x02,
    Alpha = 0x10,
    PowerPc = 0x20,
    Mips = 0x30,
    CoProcessor = 0x40,
}

#[derive(Default, Debug, Display, FromRepr, PartialEq)]
#[repr(u8)]
pub enum SerialBusController {
    /// IEEE 1394
    FireWire = 0x00,
    AccessBus = 0x01,
    Ssa = 0x02,
    UsbController = 0x03,
    FibreChannel = 0x04,
    Smbus = 0x05,
    InfiniBand = 0x06,
    IpmiInterface = 0x07,
    SercosInterface = 0x08,
    Canbus = 0x09,
    #[default]
    SerialBusController = 0x80
}

#[derive(Default, Debug, Display, FromRepr, PartialEq)]
#[repr(u8)]
pub enum WirelessController {
    IrdaController = 0x00,
    ConsumerIrController = 0x01,
    RfController = 0x10,
    Bluetooth = 0x11,
    Broadband = 0x12,
    Wifi8021aController = 0x20,
    Wifi8021bController = 0x21,
    #[default]
    WirelessController = 0x80
}

#[derive(Debug, Display, FromRepr, PartialEq)]
#[repr(u8)]
pub enum SatelliteCommunicationsController {
    SatelliteTvController = 0x01,
    SatelliteAudioCommunicationController = 0x02,
    SatelliteVoiceCommunicationController = 0x03,
    SatelliteDataCommunicationController = 0x04,
}

#[derive(Default, Debug, Display, FromRepr, PartialEq)]
#[repr(u8)]
pub enum EncryptionController {
    NetworkAndComputingEncryptionDevice = 0x00,
    EntertainmentEncryptionDevice = 0x10,
    #[default]
    EncryptionController = 0x80
}

#[derive(Default, Debug, Display, FromRepr, PartialEq)]
#[repr(u8)]
pub enum SignalProcessingController {
    DpioModule = 0x00,
    PerformanceCounters = 0x01,
    CommunicationSynchronizer = 0x10,
    SignalProcessingManagement = 0x20,
    #[default]
    SignalProcessingController = 0x80
}

#[derive(Default, Debug, Display, FromRepr, PartialEq)]
#[repr(u8)]
pub enum ProcessingAccelerators {
    #[default]
    ProcessingAccelerators = 0x00,
    /// SDXI
    SniaSmartDataAcceleratorInterfaceController = 0x01
}