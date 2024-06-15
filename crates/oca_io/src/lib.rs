#![warn(clippy::nursery)]

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
pub use ioctl::get_termsize;
