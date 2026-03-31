//! Generic in-memory store with ID generation, lookup, and persistence.
//! All domain registries build on this foundation.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use serde::{Serialize, de::DeserializeOwned};
use chrono::Utc;
use crate::persistence::fs::{self, StoreSnapshot, PersistResult};

/// Thread-safe in-memory store for domain records with optional file persistence.
#[derive(Clone)]
pub struct DomainStore<T: Clone + Send + Sync> {
    records: Arc<RwLock<HashMap<String, T>>>,
    prefix: String,
    counter: Arc<RwLock<u64>>,
    persist_path: Option<PathBuf>,
}

impl<T: Clone + Send + Sync> DomainStore<T> {
    /// Create a new in-memory store (no persistence).
    pub fn new(prefix: &str) -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
            prefix: prefix.to_string(),
            counter: Arc::new(RwLock::new(0)),
            persist_path: None,
        }
    }

    /// Create a store with file-backed persistence.
    pub fn with_persistence(prefix: &str, path: PathBuf) -> Self
    where T: DeserializeOwned
    {
        let store = Self {
            records: Arc::new(RwLock::new(HashMap::new())),
            prefix: prefix.to_string(),
            counter: Arc::new(RwLock::new(0)),
            persist_path: Some(path.clone()),
        };

        if let Ok(Some(snapshot)) = fs::load_snapshot::<T>(&path) {
            *store.records.write().unwrap() = snapshot.records;
            *store.counter.write().unwrap() = snapshot.counter;
        }

        store
    }

    /// Generate a unique ID with the store's prefix.
    pub fn next_id(&self) -> String {
        let mut counter = self.counter.write().unwrap();
        *counter += 1;
        let ts = Utc::now().format("%Y%m%d%H%M%S");
        format!("{}-{}-{}", self.prefix, *counter, ts)
    }

    /// Insert a record with a given ID.
    pub fn insert(&self, id: &str, record: T) {
        let mut store = self.records.write().unwrap();
        store.insert(id.to_string(), record);
    }

    /// Insert and return the generated ID. Auto-saves if persistence is configured.
    pub fn insert_new(&self, record: T) -> String
    where T: Serialize
    {
        let id = self.next_id();
        self.insert(&id, record);
        self.auto_save();
        id
    }

    /// Get a record by ID.
    pub fn get(&self, id: &str) -> Option<T> {
        let store = self.records.read().unwrap();
        store.get(id).cloned()
    }

    /// Update a record by ID. Returns true if found.
    pub fn update(&self, id: &str, record: T) -> bool
    where T: Serialize
    {
        let mut store = self.records.write().unwrap();
        if store.contains_key(id) {
            store.insert(id.to_string(), record);
            drop(store);
            self.auto_save();
            true
        } else {
            false
        }
    }

    /// Find all records matching a predicate.
    pub fn find_all<F>(&self, predicate: F) -> Vec<(String, T)>
    where
        F: Fn(&T) -> bool,
    {
        let store = self.records.read().unwrap();
        store
            .iter()
            .filter(|(_, v)| predicate(v))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Count all records.
    pub fn count(&self) -> usize {
        self.records.read().unwrap().len()
    }

    /// Save current state to the configured file path.
    pub fn save(&self) -> PersistResult<()>
    where T: Serialize
    {
        if let Some(ref path) = self.persist_path {
            let snapshot = StoreSnapshot {
                prefix: self.prefix.clone(),
                counter: *self.counter.read().unwrap(),
                records: self.records.read().unwrap().clone(),
            };
            fs::save_snapshot(path, &snapshot)
        } else {
            Ok(())
        }
    }

    /// Load state from the configured file path.
    pub fn load(&self) -> PersistResult<bool>
    where T: DeserializeOwned
    {
        if let Some(ref path) = self.persist_path {
            match fs::load_snapshot::<T>(path)? {
                Some(snapshot) => {
                    *self.records.write().unwrap() = snapshot.records;
                    *self.counter.write().unwrap() = snapshot.counter;
                    Ok(true)
                }
                None => Ok(false),
            }
        } else {
            Ok(false)
        }
    }

    fn auto_save(&self)
    where T: Serialize
    {
        if self.persist_path.is_some() {
            if let Err(e) = self.save() {
                eprintln!("[PERSIST_WARNING] auto-save failed for '{}': {}", self.prefix, e);
            }
        }
    }

    /// Whether persistence is configured for this store.
    pub fn is_persistent(&self) -> bool {
        self.persist_path.is_some()
    }
}
