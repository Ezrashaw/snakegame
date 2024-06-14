use std::{
    ffi::CString,
    fs::File,
    io::{self, Read},
    mem,
    os::fd::{AsRawFd, FromRawFd, RawFd},
    process, ptr,
};

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
    pub fn new(signals: &[Signal]) -> SignalFile {
        assert!(!signals.is_empty());

        let mut sigmask = 0u64;
        for &sig in signals {
            let mask = 1 << (sig as u8 - 1);
            assert!(sigmask & mask == 0); // checking for duplicate signals in slice
            sigmask |= mask;
        }

        let mut oldset = 0u64;
        let res = unsafe {
            libc::syscall(
                libc::SYS_rt_sigprocmask,
                libc::SIG_BLOCK,
                ptr::from_ref(&sigmask),
                ptr::from_mut(&mut oldset),
                mem::size_of_val(&sigmask),
            )
        };
        assert!(oldset == 0);
        if res != 0 {
            let msg = CString::new("libc call failed: ").unwrap();
            unsafe {
                libc::perror(msg.as_ptr());
            }
            process::exit(1);
        }
        assert!(res == 0);

        let signalfd = unsafe {
            libc::syscall(
                libc::SYS_signalfd4,
                -1,
                &sigmask,
                mem::size_of_val(&sigmask),
                0x0,
            )
        };
        assert!(signalfd >= 0);
        let fd = unsafe { File::from_raw_fd(signalfd as i32) };
        Self { fd }
    }

    pub fn get_signal(&mut self) -> io::Result<Signal> {
        let mut buf = [0u8; 128];
        self.fd.read(&mut buf)?;
        let sig: u8 = u32::from_ne_bytes(buf[0..4].try_into().unwrap())
            .try_into()
            .unwrap();

        Ok(unsafe { mem::transmute(sig) })
    }
}

impl AsRawFd for SignalFile {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}
