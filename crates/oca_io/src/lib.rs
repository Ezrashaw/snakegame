#[cfg(not(target_os = "linux"))]
compile_error!("This program only runs on Linux");

mod cbuf;
mod ioctl;
pub mod network;
pub mod poll;
pub mod signal;
mod syscall;
pub mod termios;

pub use cbuf::CircularBuffer;

#[repr(C)]
#[derive(Default)]
pub(crate) struct WinSize {
    rows: u16,
    cols: u16,
    xpixels: u16,
    ypixels: u16,
}

#[must_use]
pub fn get_termsize() -> (u16, u16) {
    let mut winsize = WinSize::default();
    ioctl::ioctl(ioctl::IoctlRequest::GetWinSize(&mut winsize));

    (winsize.cols, winsize.rows)
}
