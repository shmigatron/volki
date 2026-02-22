//! Instant — monotonic clock via clock_gettime.

use super::duration::Duration;
use crate::core::volkiwithstds::sys::syscalls;

/// A measurement of a monotonically non-decreasing clock.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
    secs: u64,
    nanos: u32,
}

impl Instant {
    /// Returns the current instant.
    pub fn now() -> Self {
        let mut ts = syscalls::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        unsafe {
            syscalls::clock_gettime(syscalls::CLOCK_MONOTONIC, &mut ts);
        }
        Self {
            secs: ts.tv_sec as u64,
            nanos: ts.tv_nsec as u32,
        }
    }

    /// Returns the elapsed duration since this instant.
    pub fn elapsed(&self) -> Duration {
        let now = Self::now();
        now.duration_since(*self)
    }

    /// Returns the duration between two instants.
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        let (secs, nanos) = if self.nanos >= earlier.nanos {
            (self.secs - earlier.secs, self.nanos - earlier.nanos)
        } else {
            (
                self.secs - earlier.secs - 1,
                self.nanos + 1_000_000_000 - earlier.nanos,
            )
        };
        Duration::new(secs, nanos)
    }
}

impl core::ops::Add<Duration> for Instant {
    type Output = Instant;
    fn add(self, dur: Duration) -> Instant {
        let mut secs = self.secs + dur.as_secs();
        let mut nanos = self.nanos + dur.subsec_nanos();
        if nanos >= 1_000_000_000 {
            secs += 1;
            nanos -= 1_000_000_000;
        }
        Instant { secs, nanos }
    }
}

impl core::ops::Sub for Instant {
    type Output = Duration;
    fn sub(self, other: Instant) -> Duration {
        self.duration_since(other)
    }
}
