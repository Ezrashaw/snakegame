use core::{
    net::SocketAddrV4,
    ops::{Deref, DerefMut},
    ptr, slice,
    time::Duration,
};

use crate::{
    Error,
    poll::{PollFd, poll},
    sys::syscall::{SYS_fcntl, SYS_getsockopt},
};

use super::{
    super::Result,
    file::{File, OwnedFile},
    syscall::{SYS_connect, SYS_socket, syscall_res},
};

const AF_INET: u16 = 2;
const SOCK_STREAM: u64 = 1;
const SOCK_NONBLOCK: u64 = 2048;
const O_NONBLOCK: usize = 2048;

const F_GETFL: u64 = 3;
const F_SETFL: u64 = 4;

#[repr(C)]
#[derive(Debug)]
struct SysSockAddr {
    family: u16,
    port: u16,
    addr: u32,
    _zero: [u8; 8],
}

pub struct Socket {
    file: OwnedFile,
    conn: bool,
}

impl Socket {
    pub fn connect(addr: SocketAddrV4, block: bool) -> Result<Self> {
        // FIXME: shouldn't allow to pass less than u64, since that puts garbage into the registers
        // and we _never_ want that.
        let socket = syscall_res!(
            SYS_socket,
            u64::from(AF_INET),
            SOCK_STREAM | SOCK_NONBLOCK,
            0
        )?;

        let addr = SysSockAddr {
            family: AF_INET,
            port: addr.port().to_be(),
            addr: addr.ip().to_bits().to_be(),
            _zero: [0; 8],
        };
        let res = syscall_res!(
            SYS_connect,
            socket,
            ptr::from_ref(&addr),
            size_of::<SysSockAddr>()
        );

        match res {
            // Success, somehow socket connected _immediately_
            Ok(_) => (),
            // Maybe success: socket hasn't done anything yet
            Err(Error::Syscall(115)) => {
                if !block {
                    return Ok(Self {
                        file: unsafe { OwnedFile::from_fd(socket.try_into()?) },
                        conn: false,
                    });
                }

                let poll_fd = PollFd::new(socket.try_into()?, PollFd::OUT);
                let r = poll(&mut [poll_fd], Some(Duration::from_secs(5)))?;

                if r == 0 {
                    // ETIMEDOUT
                    return Err(Error::Syscall(110));
                }

                let mut so_error: u32 = 0;
                let len: u32 = 4;
                syscall_res!(
                    SYS_getsockopt,
                    socket,
                    0x1, // SOL_SOCKET
                    0x4, // SO_ERROR
                    &raw mut so_error,
                    &raw const len
                )?;

                // success
                if so_error != 0 {
                    // ECONNREFUSED
                    return Err(Error::Syscall(111));
                }
            }

            // Failure
            Err(err) => return Err(err),
        }

        let mut flags = syscall_res!(SYS_fcntl, socket, F_GETFL, 0)?;
        flags &= !O_NONBLOCK;
        syscall_res!(SYS_fcntl, socket, F_SETFL, flags)?;

        Ok(Self {
            file: unsafe { OwnedFile::from_fd(socket.try_into()?) },
            conn: true,
        })
    }

    pub fn sock_finish_conn(&mut self) -> Result<bool> {
        if self.conn {
            return Ok(true);
        }

        let poll_fd = PollFd::new(self.as_fd(), PollFd::OUT);
        let r = poll(&mut [poll_fd], Some(Duration::from_secs(0)))?;

        if r == 0 {
            // Nothing yet happened
            return Ok(false);
        }

        let mut so_error: u32 = 0;
        let len: u32 = 4;
        syscall_res!(
            SYS_getsockopt,
            self.as_fd() as u64,
            0x1, // SOL_SOCKET
            0x4, // SO_ERROR
            &raw mut so_error,
            &raw const len
        )?;

        // success
        if so_error != 0 {
            // ECONNREFUSED
            return Err(Error::Syscall(111));
        }

        let mut flags = syscall_res!(SYS_fcntl, self.as_fd() as u64, F_GETFL, 0)?;
        flags &= !O_NONBLOCK;
        syscall_res!(SYS_fcntl, self.as_fd() as u64, F_SETFL, flags)?;

        self.conn = true;
        Ok(true)
    }

    pub const fn is_conn(&self) -> bool {
        self.conn
    }

    pub fn poll(&self) -> Result<Option<bool>> {
        let mut poll_fd = PollFd::new(self.as_fd(), PollFd::IN | PollFd::RDHUP);
        match poll(slice::from_mut(&mut poll_fd), Some(Duration::ZERO))? {
            0 => Ok(Some(false)),
            1 => {
                if poll_fd.has_socket_close() {
                    Ok(None)
                } else {
                    Ok(Some(true))
                }
            }
            _ => unreachable!(),
        }
    }
}

impl Deref for Socket {
    type Target = File;

    fn deref(&self) -> &Self::Target {
        &self.file
    }
}

impl DerefMut for Socket {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.file
    }
}
