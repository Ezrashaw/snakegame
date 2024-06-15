use std::mem::MaybeUninit;

/// A circular buffer implementation.
pub struct CircularBuffer<T: Copy, const N: usize> {
    buf: [MaybeUninit<T>; N],
    back: usize,
    front: usize,
    full: bool,
}

impl<T: Copy, const N: usize> CircularBuffer<T, N> {
    #[allow(clippy::new_without_default)]
    #[must_use]
    pub fn new() -> Self {
        assert!(N > 0);
        Self {
            // SAFETY: We never assume uninit'ed memory to be `T`. We are assuming it to be
            //         `MaybeUninit` which is always safe.
            buf: unsafe { MaybeUninit::uninit().assume_init() },
            back: 0,
            front: 0,
            full: false,
        }
    }

    pub fn push(&mut self, item: T) {
        assert!(!self.full, "buffer full");
        self.buf[self.front] = MaybeUninit::new(item);
        self.front = (self.front + 1) % N;
        if self.front == self.back {
            self.full = true;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        // if the pointers are equal, there is nothing to read/the buffer is empty.
        if (self.back == self.front) && !self.full {
            return None;
        } else if self.full {
            self.full = false;
        }
        // SAFETY: We know that at self.back, there is always a valid value to be read (except if
        //         self.back == self.front).
        let item = unsafe { (self.buf[self.back]).assume_init_read() };
        self.back = (self.back + 1) % N;
        Some(item)
    }

    pub fn clear(&mut self) {
        self.full = false;
        self.back = self.front;
    }

    pub const fn len(&self) -> usize {
        if self.full {
            N
        } else if self.front < self.back {
            self.front + (N - self.back)
        } else {
            self.front - self.back
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn iter(&self) -> CBufIter<'_, T, N> {
        CBufIter {
            buf: self,
            back: self.back,
            front: self.front,
            full: self.full,
        }
    }
}

pub struct CBufIter<'a, T: Copy, const N: usize> {
    buf: &'a CircularBuffer<T, N>,
    back: usize,
    front: usize,
    full: bool,
}

impl<T: Copy, const N: usize> Iterator for CBufIter<'_, T, N> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.back == self.buf.front {
            if self.full {
                self.full = false;
            } else {
                return None;
            }
        }

        let item = unsafe { (self.buf.buf[self.back]).assume_init_read() };
        self.back = (self.back + 1) % N;
        Some(item)
    }
}

impl<T: Copy, const N: usize> DoubleEndedIterator for CBufIter<'_, T, N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front == self.buf.back {
            if self.full {
                self.full = false;
            } else {
                return None;
            }
        }

        self.front = if self.front == 0 {
            N - 1
        } else {
            self.front - 1
        };
        let item = unsafe { (self.buf.buf[self.front]).assume_init_read() };
        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, mem::size_of};

    use crate::CircularBuffer;

    #[test]
    fn test1() {
        let buf = CircularBuffer::<i32, 35>::new();
        assert!(buf.is_empty());
        assert!(buf.iter().count() == 0);
        assert_eq!(size_of::<CircularBuffer::<i32, 35>>(), 160);
    }

    fn assert_iter<T: Copy + Debug + PartialEq, const N: usize>(
        buf: &CircularBuffer<T, N>,
        expected: &[T],
    ) {
        let mut expected = expected.to_vec();

        let vals = buf.iter().collect::<Vec<T>>();
        assert!(vals == expected);

        expected.reverse();
        let vals = buf.iter().rev().collect::<Vec<T>>();
        println!("ITER: {vals:?}");
        println!("EXPT: {expected:?}");
        assert!(vals == expected);
    }

    #[test]
    fn test2() {
        let mut cbuf = CircularBuffer::<i32, 5>::new();
        cbuf.push(4);
        cbuf.push(-5);
        assert_eq!(cbuf.len(), 2);
        assert_iter(&cbuf, &[4, -5]);

        assert_eq!(cbuf.pop(), Some(4));
        assert_eq!(cbuf.len(), 1);
        assert_iter(&cbuf, &[-5]);

        assert_eq!(cbuf.pop(), Some(-5));
        assert_eq!(cbuf.len(), 0);
        assert_iter(&cbuf, &[]);

        assert_eq!(cbuf.pop(), None);
        assert_eq!(cbuf.pop(), None);
        assert_eq!(cbuf.pop(), None);
        assert_eq!(cbuf.len(), 0);
        assert_iter(&cbuf, &[]);
    }

    #[test]
    #[should_panic(expected = "buffer full")]
    fn test3() {
        let mut cbuf = CircularBuffer::<i32, 3>::new();
        cbuf.push(4);
        cbuf.push(-5);
        cbuf.push(-5);
        assert_eq!(cbuf.len(), 3);
        assert_iter(&cbuf, &[4, -5, -5]);
        cbuf.push(-5);
    }

    #[test]
    fn test4() {
        let mut cbuf = CircularBuffer::<i32, 3>::new();
        cbuf.push(1);
        cbuf.push(2);
        cbuf.push(3); // buffer full
        assert_eq!(cbuf.len(), 3);
        assert_iter(&cbuf, &[1, 2, 3]);
        assert_eq!(cbuf.pop(), Some(1)); // leave one space
        assert_eq!(cbuf.len(), 2);
        assert_iter(&cbuf, &[2, 3]);
        cbuf.push(4);
        assert_eq!(cbuf.len(), 3);
        assert_iter(&cbuf, &[2, 3, 4]);
    }
}
