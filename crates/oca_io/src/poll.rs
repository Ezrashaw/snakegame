use core::{ptr, slice, time::Duration};
use std::os::fd::AsRawFd;

use crate::syscall::{syscall, SYS_ppoll};

pub fn poll_read_fd(fd: &impl AsRawFd, timeout: Option<Duration>) -> bool {
    let mut poll_fd = PollFd::new_read(fd);
    match poll(slice::from_mut(&mut poll_fd), timeout) {
        0 => false,
        1 => {
            assert!(poll_fd.is_read());
            true
        }
        _ => unreachable!(),
    }
}

pub fn poll(fds: &mut [PollFd], timeout: Option<Duration>) -> u32 {
    #[repr(C)]
    struct TimeSpec {
        tv_sec: u64,
        tv_nsec: u64,
    }

    let time_spec = timeout.map(|tout| TimeSpec {
        tv_sec: tout.as_secs(),
        tv_nsec: tout.subsec_nanos().into(),
    });

    let res = syscall!(
        SYS_ppoll,
        fds.as_mut_ptr() as u64,
        fds.len() as u64,
        // VERY IMPORTANT: take the reference with `as_ref`, not in a closure with
        // ptr::from_ref because the reference's (represented as a raw pointer) lifetime is
        // bound to the closure, not the libc call. Otherwise this is UB... oops. This was okay
        // in debug mode, but release mode optimized it into UB.
        time_spec.as_ref().map_or(ptr::null(), ptr::from_ref) as u64,
        ptr::null::<()>() as u64
    ) as i64;
    assert!(res != -1);

    res.try_into().unwrap()
}

#[repr(C)]
#[derive(Debug)]
pub struct PollFd {
    fd: i32,
    events: u16,
    revents: u16,
}

impl PollFd {
    const IN: u16 = 0x1;
    const RDHUP: u16 = 0x2000;

    pub fn new_socket(fd: &impl AsRawFd) -> Self {
        Self {
            fd: fd.as_raw_fd(),
            events: Self::IN | Self::RDHUP,
            revents: 0,
        }
    }

    pub fn new_read(fd: &impl AsRawFd) -> Self {
        Self {
            fd: fd.as_raw_fd(),
            events: Self::IN,
            revents: 0,
        }
    }

    #[must_use]
    pub const fn revents(&self) -> u16 {
        self.revents
    }

    #[must_use]
    pub const fn fd(&self) -> i32 {
        self.fd
    }

    #[must_use]
    pub const fn has_socket_close(&self) -> bool {
        (self.revents & Self::RDHUP) != 0
    }

    #[must_use]
    pub const fn has_read(&self) -> bool {
        (self.revents & Self::IN) != 0
    }

    #[must_use]
    pub const fn is_read(&self) -> bool {
        self.revents == Self::IN
    }
}
