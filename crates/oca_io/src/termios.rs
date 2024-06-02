use std::{mem::MaybeUninit, ptr};

macro_rules! set_bit {
    (fn $set:ident ($flag:ident) => $bit:ident) => {
        pub fn $set(&mut self, x: bool) {
            if x {
                self.0.$flag |= ::libc::$bit;
            } else {
                self.0.$flag &= !::libc::$bit;
            }
        }
    };
}

#[derive(Clone, Copy)]
pub struct Termios(libc::termios);

impl Termios {
    pub fn sys_get() -> Self {
        // SAFETY: `libc::termios` is composed entirely of integer types that can be init'd to zero
        let mut termios = Self(unsafe { MaybeUninit::zeroed().assume_init() });

        let res = unsafe {
            libc::ioctl(
                libc::STDIN_FILENO,
                libc::TCGETS,
                ptr::from_mut(&mut termios.0),
            )
        };
        assert_eq!(res, 0);

        termios
    }

    pub fn sys_set(&self) {
        let res = unsafe { libc::ioctl(libc::STDIN_FILENO, libc::TCSETS, ptr::from_ref(&self.0)) };
        assert_eq!(res, 0);
    }

    set_bit!(fn set_sig(c_lflag) => ISIG);
    set_bit!(fn set_canonical(c_lflag) => ICANON);
    set_bit!(fn set_echo(c_lflag) => ECHO);
    set_bit!(fn set_ixon(c_iflag) => IXON);
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
