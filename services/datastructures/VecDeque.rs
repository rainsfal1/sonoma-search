// A double-ended queue implementation using a ring buffer
pub struct VecDeque<T> {
    buffer: Vec<Option<T>>,
    head: usize,
    tail: usize,
    size: usize,
}

impl<T> VecDeque<T> {
    pub fn new() -> Self {
        const INITIAL_CAPACITY: usize = 8;
        VecDeque {
            buffer: vec![None; INITIAL_CAPACITY],
            head: 0,
            tail: 0,
            size: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        VecDeque {
            buffer: vec![None; capacity.max(1)],
            head: 0,
            tail: 0,
            size: 0,
        }
    }

    // Add element to the front
    pub fn push_front(&mut self, value: T) {
        if self.size == self.buffer.len() {
            self.grow();
        }

        self.head = if self.head == 0 {
            self.buffer.len() - 1
        } else {
            self.head - 1
        };
        
        self.buffer[self.head] = Some(value);
        self.size += 1;
    }

    // Add element to the back
    pub fn push_back(&mut self, value: T) {
        if self.size == self.buffer.len() {
            self.grow();
        }

        self.buffer[self.tail] = Some(value);
        self.tail = (self.tail + 1) % self.buffer.len();
        self.size += 1;
    }

    // Remove and return element from the front
    pub fn pop_front(&mut self) -> Option<T> {
        if self.size == 0 {
            return None;
        }

        let value = self.buffer[self.head].take();
        self.head = (self.head + 1) % self.buffer.len();
        self.size -= 1;

        if self.size <= self.buffer.len() / 4 && self.buffer.len() > 8 {
            self.shrink();
        }

        value
    }

    // Remove and return element from the back
    pub fn pop_back(&mut self) -> Option<T> {
        if self.size == 0 {
            return None;
        }

        self.tail = if self.tail == 0 {
            self.buffer.len() - 1
        } else {
            self.tail - 1
        };

        let value = self.buffer[self.tail].take();
        self.size -= 1;

        if self.size <= self.buffer.len() / 4 && self.buffer.len() > 8 {
            self.shrink();
        }

        value
    }

    // Get reference to front element
    pub fn front(&self) -> Option<&T> {
        if self.size == 0 {
            None
        } else {
            self.buffer[self.head].as_ref()
        }
    }

    // Get reference to back element
    pub fn back(&self) -> Option<&T> {
        if self.size == 0 {
            None
        } else {
            let index = if self.tail == 0 {
                self.buffer.len() - 1
            } else {
                self.tail - 1
            };
            self.buffer[index].as_ref()
        }
    }

    // Get mutable reference to front element
    pub fn front_mut(&mut self) -> Option<&mut T> {
        if self.size == 0 {
            None
        } else {
            self.buffer[self.head].as_mut()
        }
    }

    // Get mutable reference to back element
    pub fn back_mut(&mut self) -> Option<&mut T> {
        if self.size == 0 {
            None
        } else {
            let index = if self.tail == 0 {
                self.buffer.len() - 1
            } else {
                self.tail - 1
            };
            self.buffer[index].as_mut()
        }
    }

    // Get element at index
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.size {
            return None;
        }
        let real_index = (self.head + index) % self.buffer.len();
        self.buffer[real_index].as_ref()
    }

    // Get mutable reference to element at index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.size {
            return None;
        }
        let real_index = (self.head + index) % self.buffer.len();
        self.buffer[real_index].as_mut()
    }

    // Double the buffer size
    fn grow(&mut self) {
        let new_capacity = if self.buffer.len() == 0 {
            1
        } else {
            self.buffer.len() * 2
        };
        self.resize(new_capacity);
    }

    // Halve the buffer size
    fn shrink(&mut self) {
        let new_capacity = self.buffer.len() / 2;
        self.resize(new_capacity);
    }

    // Resize buffer and rearrange elements
    fn resize(&mut self, new_capacity: usize) {
        let mut new_buffer = vec![None; new_capacity];
        let mut current = self.head;
        
        // Copy elements to new buffer
        for i in 0..self.size {
            new_buffer[i] = self.buffer[current].take();
            current = (current + 1) % self.buffer.len();
        }

        self.buffer = new_buffer;
        self.head = 0;
        self.tail = self.size;
    }

    // Make the elements contiguous and return them as a slice
    pub fn make_contiguous(&mut self) -> &[T] {
        if self.size == 0 {
            return &[];
        }

        if self.head <= self.tail {
            // Elements are already contiguous
            unsafe {
                std::slice::from_raw_parts(
                    self.buffer[self.head].as_ref().unwrap() as *const T,
                    self.size,
                )
            }
        } else {
            // Need to make elements contiguous
            self.resize(self.buffer.len());
            unsafe {
                std::slice::from_raw_parts(
                    self.buffer[0].as_ref().unwrap() as *const T,
                    self.size,
                )
            }
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.size = 0;
        self.buffer = vec![None; 8]; // Reset to initial capacity
    }

    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }
}

// Iterator implementation
impl<T> IntoIterator for VecDeque<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { deque: self }
    }
}

pub struct IntoIter<T> {
    deque: VecDeque<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.deque.pop_front()
    }
}