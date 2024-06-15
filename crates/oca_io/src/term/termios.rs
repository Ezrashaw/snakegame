use crate::sys::ioctl::{ioctl, IoctlRequest, STDIN_FD};

pub fn init(f: impl FnOnce(&mut Termios)) -> Result<Termios, crate::Error> {
    let mut termios = Termios::sys_get()?;
    let termios_backup = termios;
    f(&mut termios);
    termios.sys_set()?;

    Ok(termios_backup)
}

macro_rules! set_bit {
    (fn $set:ident ($flag:ident) => $bit:literal) => {
        pub fn $set(&mut self, x: bool) {
            if x {
                self.$flag |= $bit;
            } else {
                self.$flag &= !$bit;
            }
        }
    };
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct Termios {
    iflag: u32,   /* input mode flags */
    oflag: u32,   /* output mode flags */
    cflag: u32,   /* control mode flags */
    lflag: u32,   /* local mode flags */
    line: u8,     /* line discipline */
    cc: [u8; 19], /* control characters */
}

impl Termios {
    pub fn sys_get() -> Result<Self, crate::Error> {
        let mut termios = Self::default();
        ioctl(STDIN_FD, IoctlRequest::GetTermAttr(&mut termios))?;

        Ok(termios)
    }

    pub fn sys_set(&self) -> Result<(), crate::Error> {
        ioctl(STDIN_FD, IoctlRequest::SetTermAttr(self))
    }

    set_bit!(fn set_sig(lflag) => 0x1);
    set_bit!(fn set_canonical(lflag) => 0x2);
    set_bit!(fn set_echo(lflag) => 0x8);
    set_bit!(fn set_ixon(iflag) => 0x400);
}
