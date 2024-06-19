#![allow(non_upper_case_globals)]

#[cfg(target_arch = "x86_64")]
pub use x86::*;

#[cfg(target_arch = "aarch64")]
pub use aarch64::*;

/// Perform a syscall call using native assembly instructions. Return a
/// [`oca_io::Result`](`crate::Result`) representing the result of the syscall (negative values
/// are errors, positive values are success).
///
/// This macro calls into the platform-specific [`syscall`] macro, and wraps the result with
/// [`oca_io::Error::from_syscall_ret`](`crate::Error::from_syscall_ret`).
macro_rules! syscall_res {
    ($($tok:tt)*) => {
        crate::Error::from_syscall_ret(crate::sys::syscall::syscall!($($tok)*))
    };
}

pub(crate) use syscall_res;

#[cfg(target_arch = "x86_64")]
mod x86 {
    pub const SYS_read: u64 = 0;
    pub const SYS_write: u64 = 1;
    pub const SYS_close: u64 = 3;
    pub const SYS_rt_sigprocmask: u64 = 14;
    pub const SYS_ioctl: u64 = 16;
    pub const SYS_socket: u64 = 41;
    pub const SYS_connect: u64 = 42;
    pub const SYS_ppoll: u64 = 271;
    pub const SYS_signalfd4: u64 = 289;

    /// Syscall on x86-64 Linux.
    ///
    /// The Linux kernel puts the return value in %rax, clobbers %rcx and %r11, and expects
    /// arguments in the following order:
    ///
    /// 0. %rax (syscall number)
    /// 1. %rdi
    /// 2. %rsi
    /// 3. %rdx
    /// 4. %r10
    macro_rules! syscall {
        ($id:ident, $arg1:expr) => {{
            let mut ret: isize;
            unsafe {
                core::arch::asm!(
                    "syscall",
                    in("rax") $id,

                    in("rdi") $arg1,

                    out("rcx") _, // the kernel clobbers %rcx and r11
                    out("r11") _, // ^^^
                    lateout("rax") ret,
                    options(nostack, preserves_flags)
                );
            }
            ret
        }};
        ($id:ident, $arg1:expr, $arg2:expr, $arg3:expr) => {{
            let mut ret: isize;
            unsafe {
                core::arch::asm!(
                    "syscall",
                    in("rax") $id,

                    in("rdi") $arg1,
                    in("rsi") $arg2,
                    in("rdx") $arg3,

                    out("rcx") _, // the kernel clobbers %rcx and r11
                    out("r11") _, // ^^^
                    lateout("rax") ret,
                    options(nostack, preserves_flags)
                );
            }
            ret
        }};
        ($id:ident, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr) => {{
            let mut ret: isize;
            unsafe {
                core::arch::asm!(
                    "syscall",
                    in("rax") $id,

                    in("rdi") $arg1,
                    in("rsi") $arg2,
                    in("rdx") $arg3,
                    in("r10") $arg4,

                    out("rcx") _, // the kernel clobbers %rcx and r11
                    out("r11") _, // ^^^
                    lateout("rax") ret,
                    options(nostack, preserves_flags)
                );
            }
            ret
        }};
    }

    pub(crate) use syscall;
}

#[cfg(target_arch = "aarch64")]
mod aarch64 {
    pub const SYS_ioctl: u64 = 29;
    pub const SYS_ppoll: u64 = 73;
    pub const SYS_signalfd4: u64 = 47;
    pub const SYS_rt_sigprocmask: u64 = 135;

    /// Syscall on aarch64 Linux.
    ///
    /// The Linux kernel puts the return value in %x0 and expects arguments in the following order:
    ///
    /// 0. %w8 (syscall number)
    /// 1. %x0
    /// 2. %x1
    /// 3. %x2
    /// 4. %x3
    macro_rules! syscall {
        ($id:ident, $arg1:expr, $arg2:expr, $arg3:expr) => {{
            let mut ret: i64;
            unsafe {
                core::arch::asm!(
                    "svc 0",
                    in("w8") $id,

                    in("x0") $arg1,
                    in("x1") $arg2,
                    in("x2") $arg3,

                    lateout("x0") ret,
                    options(nostack, preserves_flags)
                );
            }
            ret
        }};
        ($id:ident, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr) => {{
            let mut ret: i64;
            unsafe {
                core::arch::asm!(
                    "svc 0",
                    in("w8") $id,

                    in("x0") $arg1,
                    in("x1") $arg2,
                    in("x2") $arg3,
                    in("x3") $arg4,

                    lateout("x0") ret,
                    options(nostack, preserves_flags)
                );
            }
            ret
        }};
    }

    pub(crate) use syscall;
}
