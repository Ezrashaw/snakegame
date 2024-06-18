use core::{
    net::SocketAddrV4,
    ops::{Deref, DerefMut},
    ptr,
};

use super::{
    file::File,
    syscall::{syscall_res, SYS_connect, SYS_socket},
};
use crate::{file::OwnedFile, Result};

const AF_INET: u64 = 2;
const SOCK_STREAM: u64 = 1;

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
}

impl Socket {
    pub fn connect(addr: SocketAddrV4) -> Result<Self> {
        let socket = syscall_res!(SYS_socket, AF_INET, SOCK_STREAM, 0)?;

        let addr = SysSockAddr {
            family: AF_INET as u16,
            port: addr.port().to_be(),
            addr: addr.ip().to_bits().to_be(),
            _zero: [0; 8],
        };
        syscall_res!(
            SYS_connect,
            socket,
            ptr::from_ref(&addr),
            size_of::<SysSockAddr>()
        )?;

        Ok(Self {
            file: unsafe { OwnedFile::from_fd(socket as i32) },
        })
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
