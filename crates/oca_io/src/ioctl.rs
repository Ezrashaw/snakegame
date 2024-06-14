use std::ptr;

use crate::{
    syscall::{syscall3, SYS_ioctl},
    termios::Termios,
    WinSize,
};

const TCSETS: u64 = 0x5402;
const TCGETS: u64 = 0x5401;
const TIOCGWINSZ: u64 = 0x5413;
const STDIN_FD: u64 = 0x0;

pub(crate) enum IoctlRequest<'a> {
    SetTermAttr(&'a Termios),
    GetTermAttr(&'a mut Termios),
    GetWinSize(&'a mut WinSize),
}

pub(crate) fn ioctl(req: IoctlRequest) {
    let res = match req {
        IoctlRequest::SetTermAttr(termios) => unsafe {
            syscall3(SYS_ioctl, STDIN_FD, TCSETS, ptr::from_ref(termios) as u64)
        },
        IoctlRequest::GetTermAttr(termios) => unsafe {
            syscall3(SYS_ioctl, STDIN_FD, TCGETS, ptr::from_mut(termios) as u64)
        },
        IoctlRequest::GetWinSize(win_size) => unsafe {
            syscall3(
                SYS_ioctl,
                STDIN_FD,
                TIOCGWINSZ,
                ptr::from_mut(win_size) as u64,
            )
        },
    };
    assert!(res == 0);
}
