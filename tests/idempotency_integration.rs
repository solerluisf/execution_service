#[cfg(test)]
mod idempotency_integration {
    use execution_service::core::application::idempotency::IdempotencyStore;

    #[test]
    fn first_check_returns_false() {
        let store = IdempotencyStore::new();
        assert!(!store.is_processed("key1"));
    }

    #[test]
    fn mark_then_check_returns_true() {
        let store = IdempotencyStore::new();
        store.mark_processed("key1".to_string(), "result1".to_string());
        assert!(store.is_processed("key1"));
    }

    #[test]
    fn different_keys_are_independent() {
        let store = IdempotencyStore::new();
        store.mark_processed("key1".to_string(), "result1".to_string());
        assert!(store.is_processed("key1"));
        assert!(!store.is_processed("key2"));
    }

    #[test]
    fn capacity_evicts_lru() {
        let store = IdempotencyStore::with_capacity(2);
        store.mark_processed("key1".to_string(), "r1".to_string());
        store.mark_processed("key2".to_string(), "r2".to_string());
        store.mark_processed("key3".to_string(), "r3".to_string());
        // key1 should be evicted (LRU)
        assert!(!store.is_processed("key1"));
        assert!(store.is_processed("key2"));
        assert!(store.is_processed("key3"));
    }
}
