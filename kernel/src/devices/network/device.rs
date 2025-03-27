use alloc::vec;
use alloc::vec::Vec;
use core::marker::PhantomData;
use crossbeam_queue::SegQueue;
use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::time::Instant;
use crate::devices::network_controller::{NETWORK_CONTROLLER};

pub static RECEIVED_FRAMES: SegQueue<Vec<u8>> = SegQueue::new();

pub struct NetworkDevice;

impl Device for NetworkDevice {
    type RxToken<'a> = PhyRxToken<'a> where Self: 'a;
    type TxToken<'a> = PhyTxToken<'a> where Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        //println!("recv");

        match RECEIVED_FRAMES.pop() {
            None => None,
            Some(frame) =>  Some((
                PhyRxToken { frame: frame, _phantom: PhantomData },
                PhyTxToken { frame: None, _phantom: PhantomData }
            )),
        }
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        //println!("send");

        Some(PhyTxToken { frame: None, _phantom: PhantomData })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut capabilities = DeviceCapabilities::default();
        capabilities.medium = Medium::Ethernet;
        capabilities.max_burst_size = Some(1);
        capabilities.max_transmission_unit = 2048;
        capabilities
    }
}

pub struct PhyRxToken<'a> {
    frame: Vec<u8>,
    _phantom: PhantomData<&'a ()>,
}
pub struct PhyTxToken<'a> {
    frame: Option<Vec<u8>>,
    _phantom: PhantomData<&'a ()>,
}
impl<'a> RxToken for PhyRxToken<'a> {
    fn consume<R, F>(mut self, f: F) -> R where F: FnOnce(&[u8]) -> R {
        //println!("consume rx");
        f(&mut self.frame)
    }
}

impl<'a> TxToken for PhyTxToken<'a> {
    fn consume<R, F>(self, len: usize, f: F) -> R where F: FnOnce(&mut [u8]) -> R {
        //println!("consume tx");
        let mut buffer = match self.frame {
            None => vec![0u8; len],
            Some(frame) => frame
        };

        let result = f(&mut buffer);

        NETWORK_CONTROLLER.get().unwrap().send_sync(&buffer);

        result
    }
}