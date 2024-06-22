use core::{ptr, time::Duration};

use crate::{
    file::{File, OwnedFile},
    sys::syscall::{syscall_res, SYS_timerfd_create, SYS_timerfd_settime},
    Result,
};

const CLOCK_MONOTONIC: usize = 1;

#[repr(C)]
pub struct TimeSpec {
    seconds: u64,
    nanoseconds: u64,
}

impl From<Duration> for TimeSpec {
    fn from(value: Duration) -> Self {
        Self {
            seconds: value.as_secs(),
            nanoseconds: value.subsec_nanos().into(),
        }
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
