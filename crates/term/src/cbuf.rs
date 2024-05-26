use std::{fmt, mem::MaybeUninit};

/// A circular buffer implementation.
// TODO: use this for the snake's tail, replace `VecDeque`
pub struct CircularBuffer<T: Copy + fmt::Debug, const N: usize> {
    buf: [MaybeUninit<T>; N],
    back: usize,
    front: usize,
    full: bool,
}

impl<T: Copy + fmt::Debug, const N: usize> CircularBuffer<T, N> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        assert!(N > 0);
        Self {
            // SAFETY: We are never assume uninit'ed memory to be `T`. We are assuming it to be
            //         `MaybeUninit` which is always safe.
            buf: unsafe { MaybeUninit::uninit().assume_init() },
            back: 0,
            front: 0,
            full: false,
        }
    }

    pub fn write(&mut self, item: T) {
        assert!(!self.full, "buffer full");
        self.buf[self.front] = MaybeUninit::new(item);
        self.front = (self.front + 1) % N;
        if self.front == self.back {
            self.full = true;
        }
    }

    pub fn read(&mut self) -> Option<T> {
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

    #[allow(clippy::len_without_is_empty)]
    #[cfg(test)]
    pub const fn len(&self) -> usize {
        if self.full {
            N
        } else if self.front < self.back {
            self.front + (N - self.back)
        } else {
            self.front - self.back
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::CircularBuffer;

    #[test]
    fn test1() {
        let _ = CircularBuffer::<i32, 35>::new();
        assert_eq!(size_of::<CircularBuffer::<i32, 35>>(), 160);
    }

    #[test]
    fn test2() {
        let mut cbuf = CircularBuffer::<i32, 5>::new();
        cbuf.write(4);
        cbuf.write(-5);
        assert_eq!(cbuf.len(), 2);

        assert_eq!(cbuf.read(), Some(4));
        assert_eq!(cbuf.len(), 1);

        assert_eq!(cbuf.read(), Some(-5));
        assert_eq!(cbuf.len(), 0);

        assert_eq!(cbuf.read(), None);
        assert_eq!(cbuf.read(), None);
        assert_eq!(cbuf.read(), None);
        assert_eq!(cbuf.len(), 0);
    }

    #[test]
    #[should_panic(expected = "buffer full")]
    fn test3() {
        let mut cbuf = CircularBuffer::<i32, 3>::new();
        cbuf.write(4);
        cbuf.write(-5);
        cbuf.write(-5);
        assert_eq!(cbuf.len(), 3);
        cbuf.write(-5);
    }

    #[test]
    fn test4() {
        let mut cbuf = CircularBuffer::<i32, 3>::new();
        cbuf.write(1);
        cbuf.write(2);
        cbuf.write(3); // buffer full
        assert_eq!(cbuf.len(), 3);
        assert_eq!(cbuf.read(), Some(1)); // leave one space
        assert_eq!(cbuf.len(), 2);
        cbuf.write(4);
        assert_eq!(cbuf.len(), 3);
    }
}
