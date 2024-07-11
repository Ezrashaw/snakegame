use core::{
    fmt::{self, Write as _},
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use crate::StaticVec;

pub struct StaticString<const N: usize>(StaticVec<u8, N>);

impl<const N: usize> StaticString<N> {
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self(StaticVec::new())
    }

    #[must_use]
    pub fn replace(&self, from: u8, to: &str) -> Self {
        let mut new = Self::new();
        for &byte in &self.0 {
            if byte == from {
                let _ = new.write_str(to);
            } else {
                new.0.push(byte);
            }
        }

        new
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(self.0.as_slice()) }
    }
}

impl<const N: usize> Deref for StaticString<N> {
    type Target = StaticVec<u8, N>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> DerefMut for StaticString<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const N: usize> fmt::Write for StaticString<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let remaining = self.0.remaining_mut();
        assert!(s.len() <= remaining.len());

        MaybeUninit::clone_from_slice(&mut remaining[0..s.len()], s.as_bytes());
        unsafe { self.0.set_len(self.0.len() + s.len()) };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::fmt::Write;

    use crate::StaticString;

    #[test]
    fn test1() {
        let mut sstring = StaticString::<5>::new();
        assert!(sstring.len() == 0);
        assert!(sstring.remaining_mut().len() == 5);

        sstring.write_str("1234").unwrap();
        assert!(sstring.len() == 4);
        assert!(sstring.remaining_mut().len() == 1);
    }
}
