use core::{
    fmt::{self, Write as _},
    mem::MaybeUninit,
    ops::Deref,
};

use crate::StaticVec;

#[macro_export]
macro_rules! format {
    (len $len:literal, $($args:tt)+) => {{
        let mut s = ::oca_io::StaticString::<$len>::new();
        write!(s, $($args)+).unwrap();
        s
    }};
}

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

    #[must_use]
    pub const fn as_svec(&self) -> &StaticVec<u8, N> {
        &self.0
    }

    #[must_use]
    pub fn as_svec_mut(&mut self) -> &mut StaticVec<u8, N> {
        &mut self.0
    }
}

impl<const N: usize> Deref for StaticString<N> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<const N: usize> fmt::Write for StaticString<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let remaining = self.0.remaining_mut();
        assert!(s.len() <= remaining.len());

        MaybeUninit::copy_from_slice(&mut remaining[0..s.len()], s.as_bytes());
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
        assert!(sstring.as_svec_mut().remaining_mut().len() == 5);

        sstring.write_str("1234").unwrap();
        assert!(sstring.len() == 4);
        assert!(sstring.as_svec_mut().remaining_mut().len() == 1);
    }

    #[test]
    fn test_format_macro() {
        let fmt = {
            let mut s = StaticString::<3>::new();
            write!(s, "{:0>3}", 56).unwrap();
            s
        };
        assert!(fmt.as_str() == "056", "{:?}", fmt.as_str());
    }
}
