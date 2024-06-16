use core::{mem, ptr};
use std::os::fd::{AsRawFd, RawFd};

use crate::{
    file::File,
    sys::syscall::{syscall, SYS_rt_sigprocmask, SYS_signalfd4},
    Result,
};

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Signal {
    Interrupt = 2,
    Terminate = 15,
    WindowChange = 28,
}

pub struct SignalFile(File);

impl SignalFile {
    pub fn new(signals: &[Signal]) -> Result<Self> {
        assert!(!signals.is_empty());

        let mut sigmask = 0u64;
        for &sig in signals {
            let mask = 1 << (sig as u8 - 1);
            assert!(sigmask & mask == 0); // checking for duplicate signals in slice
            sigmask |= mask;
        }

        let mut oldset = 0u64;
        let res = syscall!(
            SYS_rt_sigprocmask,
            0x0, // SIG_BLOCK
            ptr::from_ref(&sigmask) as u64,
            ptr::from_mut(&mut oldset) as u64,
            mem::size_of_val(&sigmask) as u64
        );
        crate::Error::from_syscall_ret(res)?;
        assert!(oldset == 0);

        let signalfd = syscall!(
            SYS_signalfd4,
            (-1i64) as u64,
            ptr::from_ref(&sigmask) as u64,
            8, // signmask is eight bytes
            0x0
        );
        crate::Error::from_syscall_ret(signalfd)?;

        Ok(Self(File::from_fd(signalfd as i32)))
    }

    pub fn get_signal(&mut self) -> Result<Signal> {
        let mut buf = [0u8; 128];
        let n = self.0.read(&mut buf)?;
        assert!(n == 128);
        let sig: u8 = u32::from_ne_bytes(buf[0..4].try_into().unwrap())
            .try_into()
            .unwrap();

        Ok(unsafe { mem::transmute::<u8, Signal>(sig) })
    }
}

impl AsRawFd for SignalFile {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}
