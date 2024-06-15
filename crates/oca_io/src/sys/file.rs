use core::fmt;
use std::os::fd::AsRawFd;

use super::{
    ioctl::{self, STDIN_FD},
    syscall::{syscall, SYS_read, SYS_write},
};

/// A handle to a Unix file.
///
/// This structure is an abstraction over a Unix file descriptor. This type places no safety
/// restrictions on the contained file descriptor, and so makes no guarantee that the file
/// descriptor will remain open throughout the lifetime of this structure.
pub struct File(i32);

impl File {
    /// Constructs a handle representing the current process's standard input.
    ///
    /// This function returns [`None`] if the standard input does not exist or does not point to a
    /// terminal device. This can happen if the standard input has been closed or if it has been
    /// redirected to a file.
    pub fn stdin() -> Option<Self> {
        // Check to see if stdin is a tty. This might not be true if stdin has been closed or
        // redirected to a file. Note also that closing stdin doesn't preclude the current process
        // from still having a controlling terminal (we could open("/dev/tty") and check that file
        // descriptor instead of stdin), but this approach is *probably* what the user expects.
        if !ioctl::isatty(STDIN_FD) {
            return None;
        }

        // If stdin is a terminal, then return a wrapped file descriptor.
        Some(Self(STDIN_FD))
    }

    /// Constructs a handle representing the given file descriptor.
    ///
    /// There are no restrictions nor safety invariants associated with the argument to this
    /// function. An invalid file descriptor simply causes all reads/writes to fail with the
    /// approriate error.
    pub fn from_fd(fd: i32) -> Self {
        Self(fd)
    }

    /// Write the specified bytes into the [`File`].
    ///
    /// This function returns the number of bytes read, or an error.
    pub fn write(&mut self, bytes: &[u8]) -> Result<usize, crate::Error> {
        // Use the write syscall to write `bytes` (with length) to our file descriptor.
        let res = syscall!(SYS_write, self.0, bytes.as_ptr(), bytes.len());

        // The write syscall returns either a positive value representing the number of bytes
        // written, or a negative value representing an error.
        if res < 0 {
            // Wrap negative values into our `Error::Syscall` wrapper, remembering to normalize the
            // error value.
            Err(crate::Error::Syscall(res.unsigned_abs()))
        } else {
            Ok(res as usize)
        }
    }

    pub fn read(&mut self, bytes: &mut [u8]) -> Result<usize, crate::Error> {
        // Use the read syscall to read from our file descriptor into `bytes` (making sure not to
        // overrun).
        let res = syscall!(SYS_read, self.0, bytes.as_ptr(), bytes.len());

        // The read syscall returns either a positive value representing the number of bytes read,
        // or a negative value representing an error.
        if res < 0 {
            // Wrap negative values into our `Error::Syscall` wrapper, remembering to normalize the
            // error value.
            Err(crate::Error::Syscall(res.unsigned_abs()))
        } else {
            Ok(res as usize)
        }
    }
}

impl fmt::Write for File {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s.as_bytes()).map(|_| ()).map_err(|_| fmt::Error)
    }
}

// TODO: get rid of this
impl AsRawFd for File {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.0
    }
}
