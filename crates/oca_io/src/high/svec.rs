use core::{mem::MaybeUninit, ops::Deref};

pub struct StaticVec<T: Copy, const N: usize> {
    buf: [MaybeUninit<T>; N],
    len: usize,
}

impl<T: Copy, const N: usize> StaticVec<T, N> {
    pub const fn new() -> Self {
        Self {
            buf: [MaybeUninit::uninit(); N],
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
            Some(unsafe { self.buf[self.len].assume_init() })
        } else {
            None
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub fn remaining_mut(&mut self) -> &mut [MaybeUninit<T>] {
        &mut self.buf[self.len..]
    }

    pub unsafe fn set_len(&mut self, len: usize) -> bool {
        if len < N {
            self.len = len;
            true
        } else {
            false
        }
    }
}

impl<T: Copy, const N: usize> Deref for StaticVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        // FIX: write a safety comment
        unsafe { MaybeUninit::slice_assume_init_ref(&self.buf[0..self.len]) }
    }
}

// #[cfg(test)]
// mod tests {
//
