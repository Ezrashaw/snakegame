#![allow(non_upper_case_globals)]

use core::arch::asm;

pub const SYS_poll: u64 = 7;
pub const SYS_ioctl: u64 = 16;
pub const SYS_fcntl: u64 = 72;

/// Two argument syscall on x86-64 Linux.
///
/// Linux calling convention states that:
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

            out("rcx") _, // the kernel clears %rcx and r11
            out("r11") _, // ^^^
            lateout("rax") ret,
            options(nomem, nostack)
        );
    }
    ret
}

/// Three argument syscall on x86-64 Linux.
///
/// Linux calling convention states that:
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

            out("rcx") _, // the kernel clears %rcx and r11
            out("r11") _, // ^^^
            lateout("rax") ret,
            options(nomem, nostack)
        );
    }
    ret
}
