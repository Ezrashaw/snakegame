use core::{
    mem::MaybeUninit,
    ops::{Add, Sub},
    ptr,
    time::Duration,
};

use crate::{
    file::{File, OwnedFile},
    sys::syscall::{syscall_res, SYS_clock_gettime, SYS_timerfd_create, SYS_timerfd_settime},
    Result,
};

const CLOCK_MONOTONIC: usize = 1;
const NSEC_PER_SEC: u64 = 1_000_000_000;

#[repr(C)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct TimeSpec {
    seconds: u64,
    nanoseconds: u64,
}

impl TimeSpec {
    pub fn now() -> Result<Self> {
        let mut spec = MaybeUninit::<Self>::uninit();
        syscall_res!(SYS_clock_gettime, CLOCK_MONOTONIC, ptr::from_mut(&mut spec))?;

        // SAFETY: The `clock_gettime` syscall is guaranteed to initalize this structure.
        Ok(unsafe { spec.assume_init() })
    }
}

impl From<Duration> for TimeSpec {
    fn from(value: Duration) -> Self {
        Self {
            seconds: value.as_secs(),
            nanoseconds: value.subsec_nanos().into(),
        }
    }
}

impl Add<Duration> for TimeSpec {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        let mut seconds = self.seconds + rhs.as_secs();

        let mut nanoseconds = u64::from(rhs.subsec_nanos()) + self.nanoseconds;
        if nanoseconds >= NSEC_PER_SEC {
            nanoseconds -= NSEC_PER_SEC;
            seconds += 1;
        }

        Self {
            seconds,
            nanoseconds,
        }
    }
}

impl Sub<Self> for TimeSpec {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        if rhs >= self {
            return Duration::ZERO;
        }

        let (nsec, overflow) = self.nanoseconds.overflowing_sub(rhs.nanoseconds);
        let nsec = if overflow {
            NSEC_PER_SEC - (u64::MAX - nsec + 1)
        } else {
            nsec
        };
        let seconds = self.seconds - rhs.seconds;

        Duration::new(seconds, nsec as u32)
    }
}

#[repr(C)]
pub struct TimerSpec {
    interval: TimeSpec,
    initial: TimeSpec,
}

impl TimerSpec {
    #[must_use]
    pub fn new(initial: Option<Duration>, interval: Option<Duration>) -> Self {
        Self {
            interval: interval.unwrap_or_default().into(),
            initial: initial.unwrap_or_default().into(),
        }
    }
}

pub struct TimerFile(OwnedFile);

impl TimerFile {
    pub fn new() -> Result<Self> {
        let fd = syscall_res!(
            SYS_timerfd_create,
            CLOCK_MONOTONIC,
            0 // flags
        )?;

        Ok(Self(unsafe { OwnedFile::from_fd(fd as i32) }))
    }

    pub fn set(&mut self, spec: &TimerSpec) -> Result<()> {
        syscall_res!(
            SYS_timerfd_settime,
            self.0.as_fd(),
            0, // flags
            ptr::from_ref(spec),
            ptr::null_mut::<TimerSpec>()
        )
        .map(|_| ())
    }

    pub fn read(&mut self) -> Result<u64> {
        let mut bytes = [0u8; 8];
        self.0.read(&mut bytes)?;
        Ok(u64::from_ne_bytes(bytes))
    }

    pub fn as_file(&mut self) -> &mut File {
        &mut self.0
    }
}
