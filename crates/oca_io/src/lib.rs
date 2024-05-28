#[cfg(not(all(target_os = "linux")))]
compile_error!("This program only runs on Linux");

use std::{os::fd::AsRawFd, ptr, time::Duration};

mod cbuf;
pub mod network;
pub mod termios;

pub use cbuf::CircularBuffer;

pub fn poll_file(fd: &impl AsRawFd, timeout: Option<Duration>) -> bool {
    let mut poll_fd = libc::pollfd {
        fd: fd.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };

    let time_spec = timeout.map(|tout| libc::timespec {
        tv_sec: tout.as_secs() as i64,
        tv_nsec: tout.subsec_nanos().into(),
    });

    let res = unsafe {
        libc::ppoll(
            ptr::from_mut(&mut poll_fd),
            1,
            // VERY IMPORTANT: take the reference with `as_ref`, not in a closure with
            // ptr::from_ref because the reference's (represented as a raw pointer) lifetime is
            // bound to the closure, not the libc call. Otherwise this is UB... oops. This was okay
            // in debug mode, but release mode optimized it into UB.
            time_spec.as_ref().map_or(ptr::null(), ptr::from_ref),
            ptr::null::<libc::sigset_t>(),
        )
    };

    match res {
        -1 => panic!("libc call failed"),
        0 => false,
        1 => {
            assert!(poll_fd.revents == libc::POLLIN);
            true
        }
        _ => unreachable!(),
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
