use crate::devices::network::manager::NETWORK_MANAGER;
use alloc::vec::Vec;
use crossbeam_queue::SegQueue;
use spin::Lazy;

pub static PENDING_NETWORK_IRQS: Lazy<SegQueue<u8>> = Lazy::new(SegQueue::new);

pub fn process_pending_network_irqs() {
    // Drain into a local vec so we minimize locking time
    let mut drained = Vec::new();

    while let Some(line) = PENDING_NETWORK_IRQS.pop() {
        drained.push(line);
    }

    // try to acquire the lock; if cannot, requeue and exit quickly
    if let Some(mut manager) = NETWORK_MANAGER.try_lock() {
        for line in drained {
            manager.handle_interrupt(line);
        }

        manager.poll_interfaces();
    }
    else {
        // requeue and wait for next tick or explicit wake
        for line in drained {
            PENDING_NETWORK_IRQS.push(line);
        }
    }
}