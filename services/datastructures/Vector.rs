// Vector implementation
pub struct Vector<T> {
    data: Box<[T]>,
    len: usize,
    capacity: usize,
}

impl<T> Vector<T> {
    pub fn new() -> Self {
        const INITIAL_CAPACITY: usize = 8;
        Vector {
            data: Box::new([]),
            len: 0,
            capacity: INITIAL_CAPACITY,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Vector {
            data: Box::new([]),
            len: 0,
            capacity,
        }
    }

    pub fn push(&mut self, element: T) {
        if self.len == self.capacity {
            self.grow();
        }

        if self.len == 0 {
            let mut new_data = vec![].into_boxed_slice();
            std::mem::swap(&mut self.data, &mut new_data);
        }

        let mut new_data = Vec::with_capacity(self.capacity);
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.data.as_ptr(),
                new_data.as_mut_ptr(),
                self.len,
            );
            std::ptr::write(new_data.as_mut_ptr().add(self.len), element);
            new_data.set_len(self.len + 1);
        }
        self.data = new_data.into_boxed_slice();
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            let mut new_data = Vec::with_capacity(self.capacity);
            unsafe {
                let value = std::ptr::read(self.data.as_ptr().add(self.len));
                std::ptr::copy_nonoverlapping(
                    self.data.as_ptr(),
                    new_data.as_mut_ptr(),
                    self.len,
                );
                new_data.set_len(self.len);
                self.data = new_data.into_boxed_slice();
                Some(value)
            }
        }
    }

    fn grow(&mut self) {
        self.capacity = if self.capacity == 0 {
            1
        } else {
            self.capacity * 2
        };
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe {
                Some(&*self.data.as_ptr().add(index))
            }
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

// Implementing Drop for proper cleanup
impl<T> Drop for Vector<T> {
    fn drop(&mut self) {
        // The Box will handle dropping the elements
    }
}
