use core::{fmt, ops::Deref};

use super::svec::StaticVec;

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

    pub fn clear(&mut self) {
        unsafe { self.0.set_len(0) };
    }

    // modified from Rust source
    pub fn push(&mut self, ch: char) {
        match ch.len_utf8() {
            1 => {
                self.0.push(ch as u8);
            }
            _ => self.0.push_slice(ch.encode_utf8(&mut [0; 4]).as_bytes()),
        }
    }

    pub fn extend(&mut self, iter: impl Iterator<Item = char>) {
        for ch in iter {
            self.push(ch);
        }
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

impl<const N: usize> fmt::Display for StaticString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<const N: usize> fmt::Debug for StaticString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl<const N: usize> fmt::Write for StaticString<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.push_slice(s.as_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::fmt::Write as _;

    use super::StaticString;

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
