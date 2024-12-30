// HashMap implementation using linear probing for collision resolution
pub struct HashMap<K, V> {
    entries: Vec<Option<(K, V)>>,
    size: usize,
}

impl<K, V> HashMap<K, V> 
where 
    K: Eq,
{
    pub fn new() -> Self {
        const INITIAL_CAPACITY: usize = 16;
        HashMap {
            entries: vec![None; INITIAL_CAPACITY],
            size: 0,
        }
    }

    // Simple hash function using multiplication method
    fn hash(&self, key: &K) -> usize {
        let key_ptr = key as *const K as usize;
        let a: f64 = 0.6180339887; // (sqrt(5) - 1) / 2
        let hash = ((key_ptr as f64 * a) % 1.0 * self.entries.len() as f64) as usize;
        hash % self.entries.len()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.size >= self.entries.len() / 2 {
            self.resize();
        }

        let mut index = self.hash(&key);
        let mut first_deleted = None;

        // Linear probing to handle collisions
        loop {
            match &self.entries[index] {
                None => {
                    let insert_index = first_deleted.unwrap_or(index);
                    self.entries[insert_index] = Some((key, value));
                    self.size += 1;
                    return None;
                }
                Some((existing_key, _)) if existing_key == &key => {
                    let old_value = std::mem::replace(&mut self.entries[index], Some((key, value)));
                    return old_value.map(|(_, v)| v);
                }
                _ => {
                    index = (index + 1) % self.entries.len();
                }
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut index = self.hash(key);

        loop {
            match &self.entries[index] {
                None => return None,
                Some((k, v)) if k == key => return Some(v),
                _ => {
                    index = (index + 1) % self.entries.len();
                }
            }
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let mut index = self.hash(key);

        loop {
            match &self.entries[index] {
                None => return None,
                Some((k, _)) if k == key => {
                    let entry = self.entries[index].take();
                    self.size -= 1;
                    return entry.map(|(_, v)| v);
                }
                _ => {
                    index = (index + 1) % self.entries.len();
                }
            }
        }
    }

    fn resize(&mut self) {
        let old_entries = std::mem::replace(&mut self.entries, vec![None; self.entries.len() * 2]);
        self.size = 0;

        for entry in old_entries.into_iter().flatten() {
            self.insert(entry.0, entry.1);
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}