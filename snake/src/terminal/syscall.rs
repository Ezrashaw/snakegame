#![allow(non_upper_case_globals)]

#[cfg(target_arch = "x86_64")]
pub use x86::*;

#[cfg(target_arch = "x86_64")]
mod x86 {
    use std::arch::asm;

    pub const SYS_poll: u64 = 7;
    pub const SYS_ioctl: u64 = 16;
    pub const SYS_fcntl: u64 = 72;

    /// Two argument syscall on x86-64 Linux.
    ///
    /// Linux calling convention states that (ret value: %rax):
    ///
    /// **Arguments**:
    ///
    /// 0. %rax (syscall number)
    /// 1. %rdi
    /// 2. %rsi
    pub unsafe fn syscall2(id: u64, arg0: u64, arg1: u64) -> u64 {
        let mut ret;
        unsafe {
            asm!(
                "syscall",
                in("rax") id,

                // syscall arguments
                in("rdi") arg0,
                in("rsi") arg1,

                out("rcx") _, // the kernel clobbers %rcx and r11
                out("r11") _, // ^^^
                lateout("rax") ret,
                options(nostack, preserves_flags)
            );
        }
        ret
    }

    /// Three argument syscall on x86-64 Linux.
    ///
    /// Linux calling convention states that (ret value: %rax):
    ///
    /// **Arguments**:
    ///
    /// 0. %rax (syscall number)
    /// 1. %rdi
    /// 2. %rsi
    /// 3. %rdx
    pub unsafe fn syscall3(id: u64, arg0: u64, arg1: u64, arg2: u64) -> u64 {
        let mut ret;
        unsafe {
            asm!(
                "syscall",
                in("rax") id,

                // syscall arguments
                in("rdi") arg0,
                in("rsi") arg1,
                in("rdx") arg2,

                out("rcx") _, // the kernel clobbers %rcx and r11
                out("r11") _, // ^^^
                lateout("rax") ret,
                options(nostack, preserves_flags)
            );
        }
        ret
    }
}

#[cfg(target_arch = "aarch64")]
mod aarch64 {
    use std::arch::asm;

    /// Two argument syscall on aarch64 Linux.
    ///
    /// Linux calling convention states that (ret value: %x0):
    ///
    /// **Arguments**:
    ///
    /// 0. %w8 (syscall number)
    /// 1. %x0
    /// 2. %x1
    pub unsafe fn syscall2(id: u64, arg0: u64, arg1: u64) -> u64 {
        let mut ret;
        unsafe {
            asm!(
                "syscall",
                in("w8") id,

                // syscall arguments
                in("x0") arg0,
                in("x1") arg1,

                lateout("x0") ret,
                options(nostack, preserves_flags)
            );
        }
        ret
    }

    /// Two argument syscall on aarch64 Linux.
    ///
    /// Linux calling convention states that (ret value: %x0):
    ///
    /// **Arguments**:
    ///
    /// 0. %w8 (syscall number)
    /// 1. %x0
    /// 2. %x1
    /// 2. %x2
    pub unsafe fn syscall3(id: u64, arg0: u64, arg1: u64, arg2: u64) -> u64 {
        let mut ret;
        unsafe {
            asm!(
                "syscall",
                in("w8") id,

                // syscall arguments
                in("x0") arg0,
                in("x1") arg1,
                in("x2") arg2,

                lateout("x0") ret,
                options(nostack)
            );
        }
        ret
    }
}
