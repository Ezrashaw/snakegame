use core::{
    fmt,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    slice,
};

/// A vector of constant capacity.
///
/// This type is identical to [`Vec`], except for the fact that it cannot dynamically re-allocate.
/// Unlike [`Vec`], it is allocated on the stack, not heap.
pub struct StaticVec<T, const N: usize> {
    buf: [MaybeUninit<T>; N],
    len: usize,
}

impl<T: Copy, const N: usize> StaticVec<T, N> {
    /// Copy a slice onto the end of the vector.
    ///
    /// This function will likely be more efficent than calling [`Self::push`] in a loop.
    ///
    /// If the vector is full, this function does nothing and [`false`] is returned.
    /// Otherwise, `true` is returned.
    pub fn push_slice(&mut self, s: &[T]) -> bool {
        let remaining = self.remaining_mut();
        if s.len() > remaining.len() {
            return false;
        }

        remaining[0..s.len()].write_copy_of_slice(s);
        unsafe { self.set_len(self.len() + s.len()) };
        true
    }
}

impl<T, const N: usize> StaticVec<T, N> {
    /// Create a new [`StaticVec`] with no items.
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            buf: [const { MaybeUninit::uninit() }; N],
            len: 0,
        }
    }

    /// Push an element to the end of the vector.
    ///
    /// If the vector is full, this function does nothing and [`false`] is returned.
    /// Otherwise, `true` is returned.
    pub fn push(&mut self, item: T) -> bool {
        if self.len < N {
            self.buf[self.len] = MaybeUninit::new(item);
            self.len += 1;
            true
        } else {
            false
        }
    }

    /// Remove and return an element from the end of the vector.
    ///
    /// If the vector is empty, this function does nothing and [`None`] is returned.
    /// Otherwise, [`Some`] is returned.
    pub fn pop(&mut self) -> Option<T> {
        if self.len > 0 {
            self.len -= 1;
            Some(unsafe { self.buf[self.len].assume_init_read() })
        } else {
            None
        }
    }

    /// Extracts a slice containing the entire vector.
    ///
    /// This function is identical to [`Vec::as_slice`].
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: `assume_init_ref` requires that each item of the slice is actually initialized.
        //         It is a safety variant of this type that every element from index 0 to
        //         `self.len` is properly initialized.
        unsafe { self.buf[0..self.len].assume_init_ref() }
    }

    /// Extracts a mutable slice containing the entire vector.
    ///
    /// This function is identical to [`Vec::as_mut_slice`].
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: See the identical comment on `as_slice`; this type guarantees initialization
        //         for all elements up to self.len.
        unsafe { self.buf[0..self.len].assume_init_mut() }
    }

    /// Returns the number of (initialized) elements in the vector.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the length of the vector is 0---the vector is empty.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a mutable slice into the uninitialized portion of the vector.
    ///
    /// This memory is _not_ used by the vector. This function should be used to initialize items
    /// before increasing the length with [`Self::set_len`].
    pub fn remaining_mut(&mut self) -> &mut [MaybeUninit<T>] {
        &mut self.buf[self.len..]
    }

    /// Sets the initialized length of the vector.
    ///
    /// If the new length of the vector is less than the current length, note that the "lost"
    /// elements are _not_ [`Drop`]ped.
    ///
    /// # SAFETY
    ///
    /// - This function is safe if `len` is less than or equal to the current length.
    /// - Otherwise, the "new" elements must have already been initialized (using
    ///   [`Self::remaining_mut`]).
    pub unsafe fn set_len(&mut self, len: usize) {
        self.len = len;
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
