//! Tests for DomainStore — in-memory store with persistence.

use voteos::domain::store::DomainStore;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestRecord {
    name: String,
    value: u32,
}

#[test]
fn test_insert_and_get() {
    let store: DomainStore<TestRecord> = DomainStore::new("test");
    let id = store.insert_new(TestRecord { name: "alice".into(), value: 42 });
    assert!(id.starts_with("test-"));

    let record = store.get(&id).expect("record should exist");
    assert_eq!(record.name, "alice");
    assert_eq!(record.value, 42);
}

#[test]
fn test_update() {
    let store: DomainStore<TestRecord> = DomainStore::new("test");
    let id = store.insert_new(TestRecord { name: "bob".into(), value: 1 });

    let updated = store.update(&id, TestRecord { name: "bob".into(), value: 99 });
    assert!(updated);

    let record = store.get(&id).expect("record should exist");
    assert_eq!(record.value, 99);
}

#[test]
fn test_find_all() {
    let store: DomainStore<TestRecord> = DomainStore::new("test");
    store.insert_new(TestRecord { name: "alice".into(), value: 10 });
    store.insert_new(TestRecord { name: "bob".into(), value: 20 });
    store.insert_new(TestRecord { name: "alice".into(), value: 30 });

    let results = store.find_all(|r| r.name == "alice");
    assert_eq!(results.len(), 2);
}

#[test]
fn test_count() {
    let store: DomainStore<TestRecord> = DomainStore::new("test");
    assert_eq!(store.count(), 0);
    store.insert_new(TestRecord { name: "a".into(), value: 1 });
    store.insert_new(TestRecord { name: "b".into(), value: 2 });
    assert_eq!(store.count(), 2);
}

#[test]
fn test_unique_ids() {
    let store: DomainStore<TestRecord> = DomainStore::new("test");
    let id1 = store.insert_new(TestRecord { name: "a".into(), value: 1 });
    let id2 = store.insert_new(TestRecord { name: "b".into(), value: 2 });
    assert_ne!(id1, id2);
}

#[test]
fn test_persistence_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("test_store.json");

    // Write
    {
        let store: DomainStore<TestRecord> = DomainStore::with_persistence("ptest", path.clone());
        store.insert_new(TestRecord { name: "persisted".into(), value: 777 });
        store.save().expect("save should succeed");
    }

    // Read back
    {
        let store: DomainStore<TestRecord> = DomainStore::with_persistence("ptest", path);
        assert_eq!(store.count(), 1);
        let all = store.find_all(|_| true);
        assert_eq!(all[0].1.name, "persisted");
        assert_eq!(all[0].1.value, 777);
    }
}
