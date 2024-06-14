use crate::ioctl::{ioctl, IoctlRequest};

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

#[derive(Clone, Copy)]
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
    pub fn sys_get() -> Self {
        let mut termios = Self {
            iflag: 0,
            oflag: 0,
            cflag: 0,
            lflag: 0,
            line: 0,
            cc: [0; 19],
        };

        ioctl(IoctlRequest::GetTermAttr(&mut termios));
        termios
    }

    pub fn sys_set(&self) {
        ioctl(IoctlRequest::SetTermAttr(self));
    }

    set_bit!(fn set_sig(lflag) => 0x1);
    set_bit!(fn set_canonical(lflag) => 0x2);
    set_bit!(fn set_echo(lflag) => 0x8);
    set_bit!(fn set_ixon(iflag) => 0x400);
}

pub fn init(f: impl FnOnce(&mut Termios)) -> Termios {
    let mut termios = Termios::sys_get();
    let termios_backup = termios;
    f(&mut termios);
    termios.sys_set();

    termios_backup
}

pub fn restore(termios: Termios) {
    termios.sys_set();
}
