use core::hint::unreachable_unchecked;

use syscall::{syscall, SYS_exit};

pub mod file;
pub mod ioctl;
pub mod poll;
pub mod socket;
pub mod syscall;

pub fn exit(status: i32) -> ! {
    syscall!(SYS_exit, status);

    unsafe { unreachable_unchecked() }
}
