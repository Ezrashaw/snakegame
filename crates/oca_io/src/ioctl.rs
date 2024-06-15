use crate::{
    syscall::{syscall, SYS_ioctl},
    termios::Termios,
};
use core::ptr;

const TCSETS: u64 = 0x5402;
const TCGETS: u64 = 0x5401;
const TIOCGWINSZ: u64 = 0x5413;
const STDIN_FD: u64 = 0x0;

pub enum IoctlRequest<'a> {
    SetTermAttr(&'a Termios),
    GetTermAttr(&'a mut Termios),
    GetWinSize(&'a mut WinSize),
}

pub fn ioctl(req: IoctlRequest) {
    let res = match req {
        IoctlRequest::SetTermAttr(termios) => {
            syscall!(SYS_ioctl, STDIN_FD, TCSETS, ptr::from_ref(termios) as u64)
        }
        IoctlRequest::GetTermAttr(termios) => {
            syscall!(SYS_ioctl, STDIN_FD, TCGETS, ptr::from_mut(termios) as u64)
        }
        IoctlRequest::GetWinSize(win_size) => syscall!(
            SYS_ioctl,
            STDIN_FD,
            TIOCGWINSZ,
            ptr::from_mut(win_size) as u64
        ),
    };
    assert!(res == 0);
}

#[repr(C)]
#[derive(Default)]
pub struct WinSize {
    rows: u16,
    cols: u16,
    xpixels: u16,
    ypixels: u16,
}

#[must_use]
pub fn get_termsize() -> (u16, u16) {
    let mut winsize = WinSize::default();
    ioctl(IoctlRequest::GetWinSize(&mut winsize));

    (winsize.cols, winsize.rows)
}
