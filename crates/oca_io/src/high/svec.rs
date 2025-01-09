use core::{
    fmt,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    slice,
};

pub struct StaticVec<T, const N: usize> {
    buf: [MaybeUninit<T>; N],
    len: usize,
}

impl<T: Copy, const N: usize> StaticVec<T, N> {
    pub fn push_slice(&mut self, s: &[T]) {
        let remaining = self.remaining_mut();
        assert!(s.len() <= remaining.len());

        MaybeUninit::copy_from_slice(&mut remaining[0..s.len()], s);
        unsafe { self.set_len(self.len() + s.len()) };
    }
}

impl<T, const N: usize> StaticVec<T, N> {
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            buf: [const { MaybeUninit::uninit() }; N],
            len: 0,
        }
    }

    pub fn push(&mut self, item: T) -> bool {
        if self.len < N {
            self.buf[self.len] = MaybeUninit::new(item);
            self.len += 1;
            true
        } else {
            false
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len > 0 {
            self.len -= 1;
            Some(unsafe { self.buf[self.len].assume_init_read() })
        } else {
            None
        }
    }

    pub fn as_slice(&self) -> &[T] {
        // FIX: write a safety comment
        unsafe { MaybeUninit::slice_assume_init_ref(&self.buf[0..self.len]) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // FIX: write a safety comment
        unsafe { MaybeUninit::slice_assume_init_mut(&mut self.buf[0..self.len]) }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn remaining_mut(&mut self) -> &mut [MaybeUninit<T>] {
        &mut self.buf[self.len..]
    }

    pub unsafe fn set_len(&mut self, len: usize) -> bool {
        if len <= N {
            self.len = len;
            true
        } else {
            false
        }
    }
}

impl<T, const N: usize> Deref for StaticVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const N: usize> DerefMut for StaticVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a StaticVec<T, N> {
    type Item = &'a T;

    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl<T: fmt::Debug, const N: usize> fmt::Debug for StaticVec<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}

impl<T, const N: usize> Drop for StaticVec<T, N> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}
