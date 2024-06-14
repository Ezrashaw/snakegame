#![allow(non_upper_case_globals)]

#[cfg(target_arch = "x86_64")]
pub use x86::*;

#[cfg(target_arch = "aarch64")]
pub use aarch64::*;

#[cfg(target_arch = "x86_64")]
mod x86 {
    use std::arch::asm;

    pub const SYS_rt_sigprocmask: u64 = 14;
    pub const SYS_ioctl: u64 = 16;
    pub const SYS_ppoll: u64 = 271;
    pub const SYS_signalfd4: u64 = 289;

    /// Two argument syscall on x86-64 Linux.
    ///
    /// Linux calling convention states that (ret value: %rax):
    ///
    /// **Arguments**:
    ///
    /// 0. %rax (syscall number)
    /// 1. %rdi
    /// 2. %rsi
    #[allow(unused)]
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

    /// Four argument syscall on x86-64 Linux.
    ///
    /// Linux calling convention states that (ret value: %rax):
    ///
    /// **Arguments**:
    ///
    /// 0. %rax (syscall number)
    /// 1. %rdi
    /// 2. %rsi
    /// 3. %rdx
    /// 3. %r10
    pub unsafe fn syscall4(id: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
        let mut ret;
        unsafe {
            asm!(
                "syscall",
                in("rax") id,

                // syscall arguments
                in("rdi") arg0,
                in("rsi") arg1,
                in("rdx") arg2,
                in("r10") arg3,

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

    pub const SYS_fcntl: u64 = 25;
    pub const SYS_ioctl: u64 = 29;
    pub const SYS_ppoll: u64 = 73;

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
                "svc 0",
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

    /// Three argument syscall on aarch64 Linux.
    ///
    /// Linux calling convention states that (ret value: %x0):
    ///
    /// **Arguments**:
    ///
    /// 0. %w8 (syscall number)
    /// 1. %x0
    /// 2. %x1
    /// 3. %x2
    pub unsafe fn syscall3(id: u64, arg0: u64, arg1: u64, arg2: u64) -> u64 {
        let mut ret;
        unsafe {
            asm!(
                "svc 0",
                in("w8") id,

                // syscall arguments
                in("x0") arg0,
                in("x1") arg1,
                in("x2") arg2,

                lateout("x0") ret,
                options(nostack, preserves_flags)
            );
        }
        ret
    }

    /// Four argument syscall on aarch64 Linux.
    ///
    /// Linux calling convention states that (ret value: %x0):
    ///
    /// **Arguments**:
    ///
    /// 0. %w8 (syscall number)
    /// 1. %x0
    /// 2. %x1
    /// 2. %x2
    /// 3. %x3
    pub unsafe fn syscall4(id: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
        let mut ret;
        unsafe {
            asm!(
                "svc 0",
                in("w8") id,

                // syscall arguments
                in("x0") arg0,
                in("x1") arg1,
                in("x2") arg2,
                in("x3") arg3,

                lateout("x0") ret,
                options(nostack, preserves_flags)
            );
        }
        ret
    }
}
