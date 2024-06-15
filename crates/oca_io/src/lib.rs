#![warn(clippy::nursery)]

#[cfg(not(target_os = "linux"))]
compile_error!("This program only runs on Linux");

mod high;
mod sys;
mod term;

pub mod network {
    pub use crate::high::network::*;
}

pub mod poll {
    pub use crate::sys::poll::*;
}

pub mod signal {
    pub use crate::term::signal::*;
}

pub mod termios {
    pub use crate::term::termios::*;
}

pub use high::cbuf::CircularBuffer;
pub use sys::ioctl::get_termsize;
