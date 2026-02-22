//! Duration type — represents a span of time.

/// A duration of time (seconds + nanoseconds).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Duration {
    secs: u64,
    nanos: u32,
}

const NANOS_PER_SEC: u32 = 1_000_000_000;
const MILLIS_PER_SEC: u64 = 1_000;
const NANOS_PER_MILLI: u32 = 1_000_000;

impl Duration {
    /// Zero duration.
    pub const ZERO: Duration = Duration { secs: 0, nanos: 0 };

    /// Create a new Duration from seconds and additional nanoseconds.
    pub const fn new(secs: u64, nanos: u32) -> Self {
        let extra_secs = (nanos / NANOS_PER_SEC) as u64;
        Self {
            secs: secs + extra_secs,
            nanos: nanos % NANOS_PER_SEC,
        }
    }

    /// Create a Duration from whole seconds.
    pub const fn from_secs(secs: u64) -> Self {
        Self { secs, nanos: 0 }
    }

    /// Create a Duration from milliseconds.
    pub const fn from_millis(millis: u64) -> Self {
        Self {
            secs: millis / MILLIS_PER_SEC,
            nanos: ((millis % MILLIS_PER_SEC) as u32) * NANOS_PER_MILLI,
        }
    }

    /// Create a Duration from nanoseconds.
    pub const fn from_nanos(nanos: u64) -> Self {
        Self {
            secs: nanos / (NANOS_PER_SEC as u64),
            nanos: (nanos % (NANOS_PER_SEC as u64)) as u32,
        }
    }

    /// Returns the number of whole seconds.
    pub const fn as_secs(&self) -> u64 {
        self.secs
    }

    /// Returns the fractional nanoseconds part.
    pub const fn subsec_nanos(&self) -> u32 {
        self.nanos
    }

    /// Returns total duration in milliseconds.
    pub const fn as_millis(&self) -> u128 {
        (self.secs as u128) * (MILLIS_PER_SEC as u128) + (self.nanos / NANOS_PER_MILLI) as u128
    }

    /// Returns total duration in nanoseconds.
    pub const fn as_nanos(&self) -> u128 {
        (self.secs as u128) * (NANOS_PER_SEC as u128) + self.nanos as u128
    }

    /// Checked subtraction.
    pub fn checked_sub(self, rhs: Duration) -> Option<Duration> {
        if self.secs > rhs.secs || (self.secs == rhs.secs && self.nanos >= rhs.nanos) {
            let (secs, nanos) = if self.nanos >= rhs.nanos {
                (self.secs - rhs.secs, self.nanos - rhs.nanos)
            } else {
                (
                    self.secs - rhs.secs - 1,
                    self.nanos + NANOS_PER_SEC - rhs.nanos,
                )
            };
            Some(Duration { secs, nanos })
        } else {
            None
        }
    }
}

impl core::ops::Add for Duration {
    type Output = Duration;
    fn add(self, rhs: Duration) -> Duration {
        let mut secs = self.secs + rhs.secs;
        let mut nanos = self.nanos + rhs.nanos;
        if nanos >= NANOS_PER_SEC {
            secs += 1;
            nanos -= NANOS_PER_SEC;
        }
        Duration { secs, nanos }
    }
}

impl core::ops::Sub for Duration {
    type Output = Duration;
    fn sub(self, rhs: Duration) -> Duration {
        self.checked_sub(rhs)
            .expect("overflow when subtracting durations")
    }
}

impl core::fmt::Display for Duration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.nanos == 0 {
            write!(f, "{}s", self.secs)
        } else {
            write!(f, "{}.{:09}s", self.secs, self.nanos)
        }
    }
}
