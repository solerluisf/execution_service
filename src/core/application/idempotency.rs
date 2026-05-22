use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use crate::core::infrastructure::mutex_ext::MutexExt;

pub const DEFAULT_CAPACITY: usize = 100_000;

#[derive(Clone, Debug)]
struct Entry {
    result: String,
    last_accessed: std::time::Instant,
}

pub struct IdempotencyStore {
    processed: Mutex<HashMap<String, Entry>>,
    access_order: Mutex<VecDeque<String>>,
    capacity: usize,
}

impl IdempotencyStore {
    pub fn new() -> Self { Self::with_capacity(DEFAULT_CAPACITY) }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            processed: Mutex::new(HashMap::with_capacity(capacity)),
            access_order: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }
    pub fn is_processed(&self, key: &str) -> bool {
        let mut processed = self.processed.safe_lock();
        if let Some(entry) = processed.get_mut(key) {
            entry.last_accessed = std::time::Instant::now();
            drop(processed);
            self.update_access_order(key);
            true
        } else { false }
    }
    pub fn mark_processed(&self, key: String, result: String) {
        let mut processed = self.processed.safe_lock();
        let mut access_order = self.access_order.safe_lock();
        if !processed.contains_key(&key) && processed.len() >= self.capacity {
            if let Some(lru_key) = access_order.pop_back() {
                processed.remove(&lru_key);
            }
        }
        let entry = Entry { result, last_accessed: std::time::Instant::now() };
        if processed.contains_key(&key) {
            access_order.retain(|k| k != &key);
        }
        processed.insert(key.clone(), entry);
        access_order.push_front(key);
    }
    pub fn len(&self) -> usize { self.processed.safe_lock().len() }
    pub fn is_empty(&self) -> bool { self.processed.safe_lock().is_empty() }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn clear(&self) {
        self.processed.safe_lock().clear();
        self.access_order.safe_lock().clear();
    }
    fn update_access_order(&self, key: &str) {
        let mut access_order = self.access_order.safe_lock();
        if let Some(pos) = access_order.iter().position(|k| k == key) {
            access_order.remove(pos);
        }
        access_order.push_front(key.to_string());
    }
}

impl Default for IdempotencyStore {
    fn default() -> Self { Self::new() }
}
