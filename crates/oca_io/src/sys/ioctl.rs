use crate::{
    sys::syscall::{syscall, SYS_ioctl},
    termios::Termios,
    Result,
};
use core::ptr;

const TCSETS: u64 = 0x5402;
const TCGETS: u64 = 0x5401;
const TIOCGWINSZ: u64 = 0x5413;
pub const STDIN_FD: i32 = 0x0;

pub enum IoctlRequest<'a> {
    SetTermAttr(&'a Termios),
    GetTermAttr(&'a mut Termios),
    GetWinSize(&'a mut WinSize),
}

pub fn ioctl(fd: i32, req: IoctlRequest) -> Result<()> {
    let syscall_ret = match req {
        IoctlRequest::SetTermAttr(termios) => {
            syscall!(SYS_ioctl, fd, TCSETS, ptr::from_ref(termios) as u64)
        }
        IoctlRequest::GetTermAttr(termios) => {
            syscall!(SYS_ioctl, fd, TCGETS, ptr::from_mut(termios) as u64)
        }
        IoctlRequest::GetWinSize(win_size) => {
            syscall!(SYS_ioctl, fd, TIOCGWINSZ, ptr::from_mut(win_size) as u64)
        }
    };

    crate::Error::from_syscall_ret(syscall_ret).map(|_| ())
}

pub fn isatty(fd: i32) -> bool {
    let mut termios = Termios::default();
    ioctl(fd, IoctlRequest::GetTermAttr(&mut termios)).is_ok()
}

#[repr(C)]
#[derive(Default)]
pub struct WinSize {
    rows: u16,
    cols: u16,
    xpixels: u16,
    ypixels: u16,
}

pub fn get_termsize() -> Result<(u16, u16)> {
    let mut winsize = WinSize::default();
    ioctl(STDIN_FD, IoctlRequest::GetWinSize(&mut winsize))?;

    Ok((winsize.cols, winsize.rows))
}
