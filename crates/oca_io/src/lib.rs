use core::slice;
use std::{os::fd::AsRawFd, ptr, time::Duration};

mod cbuf;
pub mod network;
pub mod termios;

pub use cbuf::CircularBuffer;

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
    let res = unsafe {
        libc::poll(
            fds.as_mut_ptr().cast(),
            fds.len() as libc::nfds_t,
            timeout.map_or(-1, |t| t.as_millis() as i32),
        )
    };

    if res == -1 {
        panic!("libc call failed");
    }

    res.try_into().unwrap()
}

#[repr(transparent)]
pub struct PollFd(libc::pollfd);

impl PollFd {
    pub fn new_socket(fd: &impl AsRawFd) -> Self {
        Self(libc::pollfd {
            fd: fd.as_raw_fd(),
            events: libc::POLLIN | libc::POLLHUP,
            revents: 0,
        })
    }

    pub fn new_read(fd: &impl AsRawFd) -> Self {
        Self(libc::pollfd {
            fd: fd.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        })
    }

    pub fn fd(&self) -> i32 {
        self.0.fd
    }

    pub fn is_sock_closed(&self) -> bool {
        self.0.revents == (libc::POLLIN | libc::POLLHUP)
    }

    pub fn is_read(&self) -> bool {
        self.0.revents == libc::POLLIN
    }
}

#[must_use]
pub fn get_termsize() -> (u16, u16) {
    let mut win_size = libc::winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let res = unsafe {
        libc::ioctl(
            libc::STDOUT_FILENO,
            libc::TIOCGWINSZ,
            ptr::from_mut(&mut win_size),
        )
    };
    assert_eq!(res, 0);
    (win_size.ws_col, win_size.ws_row)
}
