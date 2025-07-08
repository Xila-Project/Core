/// Lightweight ring buffer implementation.
pub struct Ring_buffer_type<T> {
    buffer: Vec<T>,
    head: usize,
    tail: usize,
    full: bool,
}

impl<T: Copy> Ring_buffer_type<T> {
    /// Create a new ring buffer with the specified capacity.
    pub fn New(Capacity: usize) -> Self {
        Ring_buffer_type {
            buffer: Vec::with_capacity(Capacity),
            head: 0,
            tail: 0,
            full: false,
        }
    }

    /// Add an element to the buffer.
    pub fn Push(&mut self, Item: T) -> bool {
        if self.is_full() {
            return false;
        }

        if self.buffer.len() < self.buffer.capacity() {
            self.buffer.push(Item);
        } else {
            self.buffer[self.tail] = Item;
        }

        self.tail = (self.tail + 1) % self.get_capacity();

        if self.tail == self.head {
            self.full = true;
        }

        true
    }

    /// Remove an element from the buffer.
    pub fn Pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let Item = self.buffer[self.head]; // The `Copy` trait is required here
        self.head = (self.head + 1) % self.buffer.capacity();
        self.full = false;
        Some(Item)
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        !self.full && (self.head == self.tail)
    }

    /// Check if the buffer is full.
    pub fn is_full(&self) -> bool {
        self.full
    }

    /// Get the capacity of the buffer.
    pub fn get_capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Get the number of elements that are currently in the buffer.
    pub fn get_used_space(&self) -> usize {
        if self.full {
            self.get_capacity()
        } else if self.tail >= self.head {
            self.tail - self.head
        } else {
            self.get_capacity() - self.head + self.tail
        }
    }

    /// Get the number of free elements in the buffer.
    pub fn get_free_space(&self) -> usize {
        self.get_capacity() - self.get_used_space()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer() {
        let mut Ring_buffer = Ring_buffer_type::New(3);
        assert!(Ring_buffer.is_empty());
        assert!(!Ring_buffer.is_full());
        assert_eq!(Ring_buffer.get_capacity(), 3);
        assert_eq!(Ring_buffer.get_used_space(), 0);
        assert_eq!(Ring_buffer.head, 0);
        assert_eq!(Ring_buffer.tail, 0);

        assert!(Ring_buffer.Push(1));
        assert!(!Ring_buffer.is_empty());
        assert!(!Ring_buffer.is_full());
        assert_eq!(Ring_buffer.get_used_space(), 1);
        assert_eq!(Ring_buffer.head, 0);
        assert_eq!(Ring_buffer.tail, 1);

        assert!(Ring_buffer.Push(2));
        assert!(!Ring_buffer.is_empty());
        assert!(!Ring_buffer.is_full());
        assert_eq!(Ring_buffer.get_used_space(), 2);
        assert_eq!(Ring_buffer.head, 0);
        assert_eq!(Ring_buffer.tail, 2);

        assert!(Ring_buffer.Push(3));
        assert!(!Ring_buffer.is_empty());
        assert!(Ring_buffer.is_full());
        assert_eq!(Ring_buffer.get_used_space(), 3);
        assert_eq!(Ring_buffer.head, 0);
        assert_eq!(Ring_buffer.tail, 0);

        assert!(!Ring_buffer.Push(4));
        assert!(!Ring_buffer.is_empty());
        assert!(Ring_buffer.is_full());
        assert_eq!(Ring_buffer.get_used_space(), 3);
        assert_eq!(Ring_buffer.head, 0);
        assert_eq!(Ring_buffer.tail, 0);

        assert_eq!(Ring_buffer.Pop(), Some(1));
        assert!(!Ring_buffer.is_empty());
        assert!(!Ring_buffer.is_full());
        assert_eq!(Ring_buffer.get_used_space(), 2);
        assert_eq!(Ring_buffer.head, 1);
        assert_eq!(Ring_buffer.tail, 0);

        assert_eq!(Ring_buffer.Pop(), Some(2));
        assert!(!Ring_buffer.is_empty());
        assert!(!Ring_buffer.is_full());
        assert_eq!(Ring_buffer.get_used_space(), 1);
        assert_eq!(Ring_buffer.head, 2);
        assert_eq!(Ring_buffer.tail, 0);

        assert_eq!(Ring_buffer.Pop(), Some(3));
        assert!(Ring_buffer.is_empty());
        assert!(!Ring_buffer.is_full());
        assert_eq!(Ring_buffer.get_used_space(), 0);
        assert_eq!(Ring_buffer.head, 0);
        assert_eq!(Ring_buffer.tail, 0);

        assert_eq!(Ring_buffer.Pop(), None);
        assert!(Ring_buffer.is_empty());
        assert!(!Ring_buffer.is_full());
        assert_eq!(Ring_buffer.get_capacity(), 3);
        assert_eq!(Ring_buffer.get_used_space(), 0);
        assert_eq!(Ring_buffer.head, 0);
        assert_eq!(Ring_buffer.tail, 0);

        Ring_buffer.Push(4);
        assert!(!Ring_buffer.is_empty());
        assert!(!Ring_buffer.is_full());
        assert_eq!(Ring_buffer.get_used_space(), 1);
        assert_eq!(Ring_buffer.head, 0);
        assert_eq!(Ring_buffer.tail, 1);

        Ring_buffer.Push(5);
        assert!(!Ring_buffer.is_empty());
        assert!(!Ring_buffer.is_full());
        assert_eq!(Ring_buffer.get_used_space(), 2);
        assert_eq!(Ring_buffer.head, 0);
        assert_eq!(Ring_buffer.tail, 2);

        Ring_buffer.Push(6);
        assert!(!Ring_buffer.is_empty());
        assert!(Ring_buffer.is_full());
        assert_eq!(Ring_buffer.get_used_space(), 3);
        assert_eq!(Ring_buffer.head, 0);
        assert_eq!(Ring_buffer.tail, 0);

        assert_eq!(Ring_buffer.Pop(), Some(4));
        assert!(!Ring_buffer.is_empty());
        assert!(!Ring_buffer.is_full());
        assert_eq!(Ring_buffer.get_used_space(), 2);
        assert_eq!(Ring_buffer.head, 1);
        assert_eq!(Ring_buffer.tail, 0);

        assert_eq!(Ring_buffer.Pop(), Some(5));
        assert!(!Ring_buffer.is_empty());
        assert!(!Ring_buffer.is_full());
        assert_eq!(Ring_buffer.get_used_space(), 1);
        assert_eq!(Ring_buffer.head, 2);
        assert_eq!(Ring_buffer.tail, 0);

        assert_eq!(Ring_buffer.Pop(), Some(6));
        assert!(Ring_buffer.is_empty());
        assert!(!Ring_buffer.is_full());
        assert_eq!(Ring_buffer.get_used_space(), 0);
        assert_eq!(Ring_buffer.head, 0);
        assert_eq!(Ring_buffer.tail, 0);

        assert_eq!(Ring_buffer.Pop(), None);
        assert!(Ring_buffer.is_empty());
        assert!(!Ring_buffer.is_full());
        assert_eq!(Ring_buffer.get_capacity(), 3);
        assert_eq!(Ring_buffer.get_used_space(), 0);
        assert_eq!(Ring_buffer.head, 0);
        assert_eq!(Ring_buffer.tail, 0);
    }
}
