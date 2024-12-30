// HashSet implementation using linear probing for collision resolution
pub struct HashSet<T> {
    elements: Vec<Option<T>>,
    size: usize,
}

impl<T> HashSet<T>
where
    T: Eq,
{
    pub fn new() -> Self {
        const INITIAL_CAPACITY: usize = 16;
        HashSet {
            elements: vec![None; INITIAL_CAPACITY],
            size: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        HashSet {
            elements: vec![None; capacity],
            size: 0,
        }
    }

    // Simple hash function using multiplication method
    fn hash(&self, value: &T) -> usize {
        let value_ptr = value as *const T as usize;
        let a: f64 = 0.6180339887; // (sqrt(5) - 1) / 2
        let hash = ((value_ptr as f64 * a) % 1.0 * self.elements.len() as f64) as usize;
        hash % self.elements.len()
    }

    pub fn insert(&mut self, value: T) -> bool {
        if self.size >= self.elements.len() / 2 {
            self.resize();
        }

        let mut index = self.hash(&value);
        let mut first_deleted = None;

        // Linear probing to handle collisions
        loop {
            match &self.elements[index] {
                None => {
                    let insert_index = first_deleted.unwrap_or(index);
                    self.elements[insert_index] = Some(value);
                    self.size += 1;
                    return true;
                }
                Some(existing_value) if existing_value == &value => {
                    return false; // Value already exists
                }
                _ => {
                    index = (index + 1) % self.elements.len();
                }
            }
        }
    }

    pub fn contains(&self, value: &T) -> bool {
        let mut index = self.hash(value);

        loop {
            match &self.elements[index] {
                None => return false,
                Some(existing_value) if existing_value == value => return true,
                _ => {
                    index = (index + 1) % self.elements.len();
                }
            }
        }
    }

    pub fn remove(&mut self, value: &T) -> bool {
        let mut index = self.hash(value);

        loop {
            match &self.elements[index] {
                None => return false,
                Some(existing_value) if existing_value == value => {
                    self.elements[index] = None;
                    self.size -= 1;
                    return true;
                }
                _ => {
                    index = (index + 1) % self.elements.len();
                }
            }
        }
    }

    fn resize(&mut self) {
        let old_elements = std::mem::replace(&mut self.elements, vec![None; self.elements.len() * 2]);
        self.size = 0;

        for element in old_elements.into_iter().flatten() {
            self.insert(element);
        }
    }

    // Iterator implementation for the HashSet
    pub fn iter(&self) -> HashSetIterator<T> {
        HashSetIterator {
            elements: &self.elements,
            index: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn clear(&mut self) {
        self.elements = vec![None; self.elements.len()];
        self.size = 0;
    }
}

// Iterator for HashSet
pub struct HashSetIterator<'a, T> {
    elements: &'a Vec<Option<T>>,
    index: usize,
}

impl<'a, T> Iterator for HashSetIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.elements.len() {
            if let Some(value) = &self.elements[self.index] {
                self.index += 1;
                return Some(value);
            }
            self.index += 1;
        }
        None
    }
}

// Implementing common set operations
impl<T: Eq + Clone> HashSet<T> {
    pub fn union(&self, other: &HashSet<T>) -> HashSet<T> {
        let mut result = self.clone();
        for value in other.iter() {
            result.insert(value.clone());
        }
        result
    }

    pub fn intersection(&self, other: &HashSet<T>) -> HashSet<T> {
        let mut result = HashSet::new();
        for value in self.iter() {
            if other.contains(value) {
                result.insert(value.clone());
            }
        }
        result
    }

    pub fn difference(&self, other: &HashSet<T>) -> HashSet<T> {
        let mut result = HashSet::new();
        for value in self.iter() {
            if !other.contains(value) {
                result.insert(value.clone());
            }
        }
        result
    }

    pub fn symmetric_difference(&self, other: &HashSet<T>) -> HashSet<T> {
        let mut result = HashSet::new();
        for value in self.iter() {
            if !other.contains(value) {
                result.insert(value.clone());
            }
        }
        for value in other.iter() {
            if !self.contains(value) {
                result.insert(value.clone());
            }
        }
        result
    }
}