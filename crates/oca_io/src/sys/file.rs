#![warn(clippy::pedantic, clippy::nursery)]

use core::{
    fmt,
    ops::{Deref, DerefMut},
};

use super::{
    ioctl::{self, STDIN_FD},
    syscall::{syscall, syscall_res, SYS_close, SYS_read, SYS_write},
};
use crate::{high::svec::StaticVec, Error, Result};

pub struct OwnedFile(File);

impl OwnedFile {
    /// Wrap a raw file descriptor in the [`OwnedFile`] abstraction.
    ///
    /// # SAFETY
    ///
    /// The file descriptor must be open and properly owned.
    #[must_use]
    pub const unsafe fn from_fd(fd: i32) -> Self {
        Self(File(fd))
    }
}

impl Drop for OwnedFile {
    fn drop(&mut self) {
        syscall!(SYS_close, self.0.as_fd());
    }
}

impl Deref for OwnedFile {
    type Target = File;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OwnedFile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

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
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # use oca_io::file::File;
    /// use core::fmt::Write as _;
    ///
    /// let Some(mut stdin) = File::stdin() else {
    ///     # return Ok(()); // Since this is a doctest, we do actually expect stdin to be unusable.
    ///     // For some reason, we couldn't get a handle to the standard input. Return an error.
    ///     return Err("stdin not an open descriptor to a terminal device".into());
    /// };
    ///
    /// // Now we can do whatever we want with stdin...
    /// writeln!(stdin, "Hello, World!")?;
    /// # Err("expected stdin to be not a tty".into())
    /// # }
    /// ```
    #[must_use]
    pub fn stdin() -> Option<Self> {
        // Check to see if stdin is a tty. This might not be true if stdin has been closed or
        // redirected to a file. Note also that closing stdin doesn't preclude the current process
        // from still having a controlling terminal (we could open /dev/tty instead of this) and
        // check that file descriptor instead of stdin), but this approach is *probably* what the
        // user expects.
        ioctl::isatty(STDIN_FD).then_some(Self(STDIN_FD))
    }

    /// Constructs a handle representing the given file descriptor.
    ///
    /// There are no restrictions nor safety invariants associated with the argument to this
    /// function. An invalid file descriptor simply causes all reads/writes to fail with the
    /// approriate error.
    #[must_use]
    pub const fn from_fd(fd: i32) -> Self {
        Self(fd)
    }

    /// Returns the raw file descriptor that underlies this structure.
    #[must_use]
    pub const fn as_fd(&self) -> i32 {
        self.0
    }

    /// Write the specified bytes into the [`File`].
    ///
    /// This function returns the number of bytes read, or an error.
    ///
    /// # Errors
    ///
    /// This function may return a [`Error::Syscall`]. Callers should be prepared for *any* value
    /// within [`Error::Syscall`], however programs can reasonably expect (but not rely on) the
    /// error value being sane. See your `write(2)` manpage for possible values.
    pub fn write(&mut self, bytes: &[u8]) -> Result<usize> {
        // Use the write syscall to write `bytes` (with length) to our file descriptor.
        syscall_res!(SYS_write, self.0, bytes.as_ptr(), bytes.len())
    }

    /// Read up to the specified number of bytes from the [`File`].
    ///
    /// This function returns the number of bytes read, or an error.
    ///
    /// # Errors
    ///
    /// This function may return a [`Error::Syscall`]. Callers should be prepared for *any* value
    /// within [`Error::Syscall`], however programs can reasonably expect (but not rely on) the
    /// error value being sane. See your `read(2)` manpage for possible values.
    pub fn read(&mut self, bytes: &mut [u8]) -> Result<usize> {
        // Use the read syscall to read from our file descriptor into `bytes` (making sure not to
        // overrun).
        syscall_res!(SYS_read, self.0, bytes.as_ptr(), bytes.len())
    }

    /// Read up to `count` bytes (from the file descriptor) into the uninitialized region of a
    /// [`StaticVec`].
    ///
    /// # Errors
    ///
    /// This function may return a [`Error::Syscall`]. Callers should be prepared for *any* value
    /// within [`Error::Syscall`], however programs can reasonably expect (but not rely on) the
    /// error value being sane. See your `read(2)` manpage for possible values.
    ///
    /// Additionally, this function will return [`Error::BufferFull`] if `count` is less than the
    /// remaining space in the [`StaticVec`].
    pub fn read_uninit<const N: usize>(
        &mut self,
        bytes: &mut StaticVec<u8, N>,
        count: usize,
    ) -> Result<usize> {
        let remaining = bytes.remaining_mut();
        if count > remaining.len() {
            return Err(Error::BufferFull);
        }
        let len = syscall_res!(SYS_read, self.0, remaining.as_ptr(), count)?;
        // SAFETY: This is safe because all the bytes were written by the `read` syscall.
        unsafe { bytes.set_len(bytes.len() + len) };
        Ok(len)
    }
}

impl fmt::Write for File {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s.as_bytes()).map(|_| ()).map_err(|_| fmt::Error)
    }
}
