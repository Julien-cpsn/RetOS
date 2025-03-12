use alloc::format;
use alloc::string::String;
use core::sync::atomic::{AtomicU32, Ordering};

static TICKS: AtomicU32 = AtomicU32::new(0);

/// This tick interrupt handler is assumed to be called once per millisecond
pub fn tick_handler() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}

pub struct MilliSecondClock;

impl MilliSecondClock {

    pub fn now() -> u32 {
        TICKS.load(Ordering::Relaxed)
    }

    pub fn seconds() -> u32 {
        Self::now() / 10
    }

    pub fn minutes() -> u32 {
        Self::seconds() / 60
    }

    pub fn hours() -> u32 {
        Self::minutes() / 60
    }

    pub fn format() -> String {
        let now = MilliSecondClock::now();
        let seconds = now / 10;
        let minutes = seconds / 60;
        let hours = minutes / 60;
        format!("{:02}:{:02}:{:02}.{:02}", hours % 60, minutes % 60, seconds % 60, now % 100)
    }
}