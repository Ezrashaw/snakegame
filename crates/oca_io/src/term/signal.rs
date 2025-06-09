use core::{mem, ptr};

use crate::{
    Result,
    file::{File, OwnedFile},
    sys::syscall::{SYS_rt_sigprocmask, SYS_signalfd4, syscall_res},
};

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Signal {
    Interrupt = 2,
    Terminate = 15,
    WindowChange = 28,
}

pub struct SignalFile(OwnedFile);

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
        syscall_res!(
            SYS_rt_sigprocmask,
            0x0, // SIG_BLOCK
            ptr::from_ref(&sigmask),
            ptr::from_mut(&mut oldset),
            mem::size_of_val(&sigmask)
        )?;
        assert!(oldset == 0);

        let signalfd = syscall_res!(
            SYS_signalfd4,
            -1,
            ptr::from_ref(&sigmask),
            8, // sigmask is eight bytes
            0x0
        )?;

        Ok(Self(unsafe { OwnedFile::from_fd(signalfd.try_into()?) }))
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

    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Clippy bug; this function can't be constant
    pub fn as_file(&self) -> &File {
        &self.0
    }
}
