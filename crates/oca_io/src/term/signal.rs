use std::{
    fs::File,
    io::{self, Read},
    mem,
    os::fd::{AsRawFd, FromRawFd, RawFd},
    ptr,
};

use crate::sys::syscall::{syscall, SYS_rt_sigprocmask, SYS_signalfd4};

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Signal {
    Interrupt = 2,
    Terminate = 15,
    WindowChange = 28,
}

pub struct SignalFile {
    fd: File,
}

impl SignalFile {
    pub fn new(signals: &[Signal]) -> Self {
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
        assert!(oldset == 0);
        assert!(res == 0);

        let signalfd = syscall!(
            SYS_signalfd4,
            (-1i64) as u64,
            ptr::from_ref(&sigmask) as u64,
            8, // signmask is eight bytes
            0x0
        ) as i64;
        assert!(signalfd >= 0);
        let fd = unsafe { File::from_raw_fd(signalfd as i32) };
        Self { fd }
    }

    pub fn get_signal(&mut self) -> io::Result<Signal> {
        let mut buf = [0u8; 128];
        let n = self.fd.read(&mut buf)?;
        assert!(n == 128);
        let sig: u8 = u32::from_ne_bytes(buf[0..4].try_into().unwrap())
            .try_into()
            .unwrap();

        Ok(unsafe { mem::transmute::<u8, Signal>(sig) })
    }
}

impl AsRawFd for SignalFile {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}
