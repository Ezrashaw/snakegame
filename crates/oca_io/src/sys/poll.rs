use core::{ptr, slice, time::Duration};

use super::syscall::{syscall_res, SYS_ppoll};
use crate::{file::File, term::timer::TimeSpec, Result};

pub fn poll_read_fd(fd: &File, timeout: Option<Duration>) -> Result<bool> {
    let mut poll_fd = PollFd::new(fd.as_fd(), PollFd::IN);
    match poll(slice::from_mut(&mut poll_fd), timeout)? {
        0 => Ok(false),
        1 => {
            assert!(poll_fd.is_read());
            Ok(true)
        }
        _ => unreachable!(),
    }
}

pub fn poll(fds: &mut [PollFd], timeout: Option<Duration>) -> Result<usize> {
    syscall_res!(
        SYS_ppoll,
        fds.as_mut_ptr() as u64,
        fds.len() as u64,
        // VERY IMPORTANT: take the reference with `as_ref`, not in a closure with
        // ptr::from_ref because the reference's (represented as a raw pointer) lifetime is
        // bound to the closure, not the libc call. Otherwise this is UB... oops. This was okay
        // in debug mode, but release mode optimized it into UB.
        timeout
            .map(TimeSpec::from)
            .as_ref()
            .map_or(ptr::null(), ptr::from_ref) as u64,
        ptr::null::<()>() as u64
    )
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PollFd {
    fd: i32,
    events: u16,
    revents: u16,
}

impl PollFd {
    pub const IN: u16 = 0x1;
    pub const RDHUP: u16 = 0x2000;

    #[must_use]
    pub const fn new(fd: i32, events: u16) -> Self {
        Self {
            fd,
            events,
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

    #[must_use]
    pub const fn is_error(&self) -> bool {
        self.revents != Self::IN && self.revents != 0
    }
}
