use std::ptr;

macro_rules! getset_bit {
    (fn $get:ident $set:ident ($flag:ident) => $bit:ident) => {
        #[allow(unused)] // TODO: remove this
        pub const fn $get(&self) -> bool {
            self.0.$flag & ::libc::$bit != 0
        }

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
        let mut termios = Self(libc::termios {
            c_iflag: 0,
            c_oflag: 0,
            c_cflag: 0,
            c_lflag: 0,
            c_line: 0,
            c_cc: [0; libc::NCCS],
            c_ispeed: 0,
            c_ospeed: 0,
        });

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

    getset_bit!(fn get_sig set_sig (c_lflag) => ISIG );
    getset_bit!(fn get_canonical set_canonical (c_lflag) => ICANON );
    getset_bit!(fn get_echo set_echo (c_lflag) => ECHO );
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
