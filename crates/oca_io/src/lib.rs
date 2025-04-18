#![feature(maybe_uninit_slice, maybe_uninit_write_slice)]
#![warn(clippy::nursery, clippy::pedantic)]
#![allow(
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]
#![cfg_attr(not(doc), no_std)]

#[cfg(not(target_os = "linux"))]
compile_error!("This program only runs on Linux");

mod error;
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

pub mod timer {
    pub use crate::term::timer::*;
}

pub mod termios {
    pub use crate::term::termios::*;
}

pub mod file {
    pub use crate::sys::file::*;
}

pub mod socket {
    pub use crate::sys::socket::*;
}

pub type Result<T> = core::result::Result<T, error::Error>;

pub use error::Error;
pub use high::{cbuf::CircularBuffer, string::StaticString, svec::StaticVec};
pub use sys::exit;
pub use sys::ioctl::get_termsize;
