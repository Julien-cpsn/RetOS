use alloc::format;
use alloc::string::String;
use core::sync::atomic::{AtomicUsize, Ordering};
use smoltcp::time::{Duration, Instant};

static TICKS: AtomicUsize = AtomicUsize::new(0);

/// This tick interrupt handler is assumed to be called once per millisecond
pub fn tick_handler() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}

pub struct Clock;

impl Clock {
    pub fn now() -> Instant {
        Instant::from_millis(TICKS.load(Ordering::SeqCst) as i64 * 16)
    }

    pub fn format() -> String {
        let now = Clock::now();
        let seconds = now.secs();
        let minutes = seconds / 60;
        let hours = minutes / 60;
        format!("{:02}:{:02}:{:02}.{:02}", hours % 60, minutes % 60, seconds % 60, now.total_millis() % 60)
    }

    pub fn elapsed(instant: Instant) -> Duration {
        Clock::now() - instant
    }
}

pub fn sleep(seconds: u64) {
    let start = Clock::now();
    let duration = Duration::from_secs(seconds);
    while Clock::elapsed(start) < duration {}
}