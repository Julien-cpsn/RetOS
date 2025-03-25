use crate::devices::drivers::e1000::E1000;
use crate::devices::drivers::rtl8139::RTL8139;
use alloc::boxed::Box;
use alloc::format;
use alloc::vec::Vec;
use core::ops::DerefMut;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_util::task::AtomicWaker;
use futures_util::Stream;
use futures_util::Sink;
use goolog::{info};
use spin::{Mutex, Once};
use strum::Display;
use x86_64::instructions::interrupts;
use crate::interrupts::idt::{network_packet_handler, register_interrupt};
use crate::interrupts::interrupt::InterruptIndex;

const GOOLOG_TARGET: &str = "NETWORK";

pub static NETWORK_CONTROLLER: Once<Mutex<NetworkController>> = Once::new();
pub static NETWORK_CONTROLLER_WAKER: AtomicWaker = AtomicWaker::new();


#[derive(Debug, Display)]
pub enum NetworkController {
    RTL8139(Box<RTL8139>),
    E1000(Box<E1000>),
}

pub fn register_network_controller(network_controller: NetworkController, interrupt_line: u32) {
    info!("Device detected: {network_controller}");
    let mac = network_controller.mac().map(|byte| format!("{:02X}", byte)).join(":");
    info!("MAC address: {mac}");

    NETWORK_CONTROLLER.call_once(|| Mutex::new(network_controller));
    register_interrupt(
        0x10 + 2 * interrupt_line,
        InterruptIndex::NetworkPacket,
        network_packet_handler,
    );
}

impl NetworkController {
    pub fn mac(&self) -> [u8; 6] {
        match self {
            NetworkController::RTL8139(rtl8139) => rtl8139.mac,
            NetworkController::E1000(e1000) => e1000.mac,
        }
    }
    
    pub fn receive_sync(&mut self) -> Option<Vec<u8>> {
        match self {
            NetworkController::RTL8139(rtl839) => rtl839.try_recv_sync(),
            NetworkController::E1000(e1000) => e1000.try_recv_sync()
        }
    }
    
    pub fn send_sync(&mut self, buffer: &[u8]) {
        match self {
            NetworkController::RTL8139(rtl839) => rtl839.send_sync(buffer),
            NetworkController::E1000(e1000) => e1000.send_sync(buffer)
        }
    }
    
    pub fn end_interrupt(&mut self) {
        match self {
            NetworkController::RTL8139(rlt8139) => rlt8139.on_interrupt(),
            NetworkController::E1000(e1000) => e1000.on_interrupt(),
        }
    }
}

impl Stream for NetworkController {
    type Item = Vec<u8>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let frames = match self.deref_mut() {
            NetworkController::RTL8139(rtl8139) => &mut rtl8139.frames,
            NetworkController::E1000(e1000) => &mut e1000.frames,
        };

        if let Some(x) = frames.pop() {
            return Poll::Ready(Some(x));
        }

        NETWORK_CONTROLLER_WAKER.register(cx.waker());

        match frames.pop() {
            Some(x) => {
                NETWORK_CONTROLLER_WAKER.take();
                Poll::Ready(Some(x))
            }
            None => Poll::Pending,
        }
    }
}

impl Sink<Vec<u8>> for NetworkController {
    type Error = ();

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(mut self: Pin<&mut Self>, item: Vec<u8>) -> Result<(), Self::Error> {
        let tx =match &mut *self {
            NetworkController::RTL8139(rtl8139) => &mut rtl8139.tx,
            NetworkController::E1000(e1000) => &mut e1000.tx
        };
        tx.push(item);
        Ok(())
    }

    fn poll_flush(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        interrupts::without_interrupts(|| {
            let tx = match self.deref_mut() {
                NetworkController::RTL8139(rtl8139) => &mut rtl8139.tx,
                NetworkController::E1000(e1000) => &mut e1000.tx
            };

            let items = core::mem::take(tx);
            for item in items {
                // Ca be better
                match &mut *self {
                    NetworkController::RTL8139(rtl8139) => rtl8139.send_sync(&item),
                    NetworkController::E1000(e1000) => e1000.send_sync(&item),
                };
            }
        });
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_flush(cx)
    }
}