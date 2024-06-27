/// Lightweight ring buffer implementation.
pub struct Ring_buffer_type<T> {
    Buffer: Vec<T>,
    Head: usize,
    Tail: usize,
    Full: bool,
}

impl<T: Copy> Ring_buffer_type<T> {
    /// Create a new ring buffer with the specified capacity.
    pub fn New(Capacity: usize) -> Self {
        Ring_buffer_type {
            Buffer: Vec::with_capacity(Capacity),
            Head: 0,
            Tail: 0,
            Full: false,
        }
    }

    /// Add an element to the buffer.
    pub fn Push(&mut self, Item: T) -> bool {
        if self.Is_full() {
            return false;
        }

        if self.Buffer.len() < self.Buffer.capacity() {
            self.Buffer.push(Item);
        } else {
            self.Buffer[self.Tail] = Item;
        }

        self.Tail = (self.Tail + 1) % self.Get_capacity();

        if self.Tail == self.Head {
            self.Full = true;
        }

        true
    }

    /// Remove an element from the buffer.
    pub fn Pop(&mut self) -> Option<T> {
        if self.Is_empty() {
            return None;
        }
        let Item = self.Buffer[self.Head]; // The `Copy` trait is required here
        self.Head = (self.Head + 1) % self.Buffer.capacity();
        self.Full = false;
        Some(Item)
    }

    /// Check if the buffer is empty.
    pub fn Is_empty(&self) -> bool {
        !self.Full && (self.Head == self.Tail)
    }

    /// Check if the buffer is full.
    pub fn Is_full(&self) -> bool {
        self.Full
    }

    /// Get the capacity of the buffer.
    pub fn Get_capacity(&self) -> usize {
        self.Buffer.capacity()
    }

    /// Get the number of elements that are currently in the buffer.
    pub fn Get_used_space(&self) -> usize {
        if self.Full {
            self.Get_capacity()
        } else if self.Tail >= self.Head {
            self.Tail - self.Head
        } else {
            self.Get_capacity() - self.Head + self.Tail
        }
    }

    /// Get the number of free elements in the buffer.
    pub fn Get_free_space(&self) -> usize {
        self.Get_capacity() - self.Get_used_space()
    }
}

#[cfg(test)]
mod Tests {
    use super::*;

    #[test]
    fn Test_ring_buffer() {
        let mut Ring_buffer = Ring_buffer_type::New(3);
        assert!(Ring_buffer.Is_empty());
        assert!(!Ring_buffer.Is_full());
        assert_eq!(Ring_buffer.Get_capacity(), 3);
        assert_eq!(Ring_buffer.Get_used_space(), 0);
        assert_eq!(Ring_buffer.Head, 0);
        assert_eq!(Ring_buffer.Tail, 0);

        assert!(Ring_buffer.Push(1));
        assert!(!Ring_buffer.Is_empty());
        assert!(!Ring_buffer.Is_full());
        assert_eq!(Ring_buffer.Get_used_space(), 1);
        assert_eq!(Ring_buffer.Head, 0);
        assert_eq!(Ring_buffer.Tail, 1);

        assert!(Ring_buffer.Push(2));
        assert!(!Ring_buffer.Is_empty());
        assert!(!Ring_buffer.Is_full());
        assert_eq!(Ring_buffer.Get_used_space(), 2);
        assert_eq!(Ring_buffer.Head, 0);
        assert_eq!(Ring_buffer.Tail, 2);

        assert!(Ring_buffer.Push(3));
        assert!(!Ring_buffer.Is_empty());
        assert!(Ring_buffer.Is_full());
        assert_eq!(Ring_buffer.Get_used_space(), 3);
        assert_eq!(Ring_buffer.Head, 0);
        assert_eq!(Ring_buffer.Tail, 0);

        assert!(!Ring_buffer.Push(4));
        assert!(!Ring_buffer.Is_empty());
        assert!(Ring_buffer.Is_full());
        assert_eq!(Ring_buffer.Get_used_space(), 3);
        assert_eq!(Ring_buffer.Head, 0);
        assert_eq!(Ring_buffer.Tail, 0);

        assert_eq!(Ring_buffer.Pop(), Some(1));
        assert!(!Ring_buffer.Is_empty());
        assert!(!Ring_buffer.Is_full());
        assert_eq!(Ring_buffer.Get_used_space(), 2);
        assert_eq!(Ring_buffer.Head, 1);
        assert_eq!(Ring_buffer.Tail, 0);

        assert_eq!(Ring_buffer.Pop(), Some(2));
        assert!(!Ring_buffer.Is_empty());
        assert!(!Ring_buffer.Is_full());
        assert_eq!(Ring_buffer.Get_used_space(), 1);
        assert_eq!(Ring_buffer.Head, 2);
        assert_eq!(Ring_buffer.Tail, 0);

        assert_eq!(Ring_buffer.Pop(), Some(3));
        assert!(Ring_buffer.Is_empty());
        assert!(!Ring_buffer.Is_full());
        assert_eq!(Ring_buffer.Get_used_space(), 0);
        assert_eq!(Ring_buffer.Head, 0);
        assert_eq!(Ring_buffer.Tail, 0);

        assert_eq!(Ring_buffer.Pop(), None);
        assert!(Ring_buffer.Is_empty());
        assert!(!Ring_buffer.Is_full());
        assert_eq!(Ring_buffer.Get_capacity(), 3);
        assert_eq!(Ring_buffer.Get_used_space(), 0);
        assert_eq!(Ring_buffer.Head, 0);
        assert_eq!(Ring_buffer.Tail, 0);

        Ring_buffer.Push(4);
        assert!(!Ring_buffer.Is_empty());
        assert!(!Ring_buffer.Is_full());
        assert_eq!(Ring_buffer.Get_used_space(), 1);
        assert_eq!(Ring_buffer.Head, 0);
        assert_eq!(Ring_buffer.Tail, 1);

        Ring_buffer.Push(5);
        assert!(!Ring_buffer.Is_empty());
        assert!(!Ring_buffer.Is_full());
        assert_eq!(Ring_buffer.Get_used_space(), 2);
        assert_eq!(Ring_buffer.Head, 0);
        assert_eq!(Ring_buffer.Tail, 2);

        Ring_buffer.Push(6);
        assert!(!Ring_buffer.Is_empty());
        assert!(Ring_buffer.Is_full());
        assert_eq!(Ring_buffer.Get_used_space(), 3);
        assert_eq!(Ring_buffer.Head, 0);
        assert_eq!(Ring_buffer.Tail, 0);

        assert_eq!(Ring_buffer.Pop(), Some(4));
        assert!(!Ring_buffer.Is_empty());
        assert!(!Ring_buffer.Is_full());
        assert_eq!(Ring_buffer.Get_used_space(), 2);
        assert_eq!(Ring_buffer.Head, 1);
        assert_eq!(Ring_buffer.Tail, 0);

        assert_eq!(Ring_buffer.Pop(), Some(5));
        assert!(!Ring_buffer.Is_empty());
        assert!(!Ring_buffer.Is_full());
        assert_eq!(Ring_buffer.Get_used_space(), 1);
        assert_eq!(Ring_buffer.Head, 2);
        assert_eq!(Ring_buffer.Tail, 0);

        assert_eq!(Ring_buffer.Pop(), Some(6));
        assert!(Ring_buffer.Is_empty());
        assert!(!Ring_buffer.Is_full());
        assert_eq!(Ring_buffer.Get_used_space(), 0);
        assert_eq!(Ring_buffer.Head, 0);
        assert_eq!(Ring_buffer.Tail, 0);

        assert_eq!(Ring_buffer.Pop(), None);
        assert!(Ring_buffer.Is_empty());
        assert!(!Ring_buffer.Is_full());
        assert_eq!(Ring_buffer.Get_capacity(), 3);
        assert_eq!(Ring_buffer.Get_used_space(), 0);
        assert_eq!(Ring_buffer.Head, 0);
        assert_eq!(Ring_buffer.Tail, 0);
    }
}
