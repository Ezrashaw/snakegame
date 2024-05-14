use std::ptr;

use super::syscall::{syscall3, SYS_ioctl};

const NCCS: usize = 19;

// c_lflag bits
const ISIG: u32 = 0b0001;
const ICANON: u32 = 0b0010;
const ECHO: u32 = 0b1000;

pub const STDIN_FD: u64 = 0;
pub const STDOUT_FD: u64 = 1;

macro_rules! getset_bit {
    (fn $get:ident $set:ident ($flag:ident) => $bit:expr) => {
        #[allow(unused)] // TODO: remove this
        pub const fn $get(&self) -> bool {
            self.$flag & $bit != 0
        }

        pub fn $set(&mut self, x: bool) {
            if x {
                self.$flag |= $bit;
            } else {
                self.$flag &= !$bit;
            }
        }
    };
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct Termios {
    iflag: u32,     /* input mode flags */
    oflag: u32,     /* output mode flags */
    cflag: u32,     /* control mode flags */
    lflag: u32,     /* local mode flags */
    line: u8,       /* line discipline */
    cc: [u8; NCCS], /* control characters */
}

impl Termios {
    pub fn sys_get() -> Self {
        let mut termios = Self::default();

        // SAFETY: I am following the ioctl syscall documentation/source.
        let syscall = unsafe {
            syscall3(
                SYS_ioctl,
                STDIN_FD,
                0x5401, // TCGETS
                ptr::from_mut(&mut termios) as u64,
            )
        };
        assert_eq!(syscall, 0);

        termios
    }

    pub fn sys_set(&self) {
        // SAFETY: I am following the ioctl syscall documentation/source.
        let syscall = unsafe {
            syscall3(
                SYS_ioctl,
                STDIN_FD,
                0x5402, // TCSETS
                ptr::from_ref(self) as u64,
            )
        };
        assert_eq!(syscall, 0);
    }

    getset_bit!(fn get_sig set_sig (lflag) => ISIG );
    getset_bit!(fn get_canonical set_canonical (lflag) => ICANON );
    getset_bit!(fn get_echo set_echo (lflag) => ECHO );
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
