use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub key: String,
    pub value: f64,
    pub timestamp: u64,
}

#[derive(Clone, Default)]
pub struct Store {
    data: Arc<Mutex<HashMap<String, Vec<Record>>>>,
}

impl Store {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, key: &str, value: f64, timestamp: u64) -> Result<(), &'static str> {
        let key = key.trim();
        if key.is_empty() || key.len() > 128 {
            return Err("key must contain between 1 and 128 characters");
        }
        if !value.is_finite() {
            return Err("value must be finite");
        }

        let mut data = self.data.lock().map_err(|_| "store lock poisoned")?;
        let records = data.entry(key.to_string()).or_default();
        records.push(Record {
            key: key.to_string(),
            value,
            timestamp,
        });
        records.sort_unstable_by_key(|record| record.timestamp);
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Vec<Record>, &'static str> {
        let key = key.trim();
        let data = self.data.lock().map_err(|_| "store lock poisoned")?;
        Ok(data.get(key).cloned().unwrap_or_default())
    }

    pub fn range(&self, key: &str, start: u64, end: u64) -> Result<Vec<Record>, &'static str> {
        if start > end {
            return Err("start must be less than or equal to end");
        }
        let records = self.get(key)?;
        Ok(records
            .into_iter()
            .filter(|record| record.timestamp >= start && record.timestamp <= end)
            .collect())
    }

    pub fn count(&self) -> Result<usize, &'static str> {
        let data = self.data.lock().map_err(|_| "store lock poisoned")?;
        Ok(data.values().map(Vec::len).sum())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_orders_records_and_counts() {
        let store = Store::new();
        store.insert("cpu", 80.0, 2_000).unwrap();
        store.insert("cpu", 75.5, 1_000).unwrap();
        let records = store.get("cpu").unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].timestamp, 1_000);
        assert_eq!(store.count().unwrap(), 2);
    }

    #[test]
    fn range_filters_inclusively() {
        let store = Store::new();
        for timestamp in [1_000, 2_000, 3_000] {
            store.insert("cpu", timestamp as f64, timestamp).unwrap();
        }
        let records = store.range("cpu", 1_500, 3_000).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].timestamp, 2_000);
        assert_eq!(records[1].timestamp, 3_000);
    }

    #[test]
    fn normalizes_keys_consistently() {
        let store = Store::new();
        store.insert(" cpu ", 42.0, 1_000).unwrap();
        assert_eq!(store.get(" cpu ").unwrap().len(), 1);
        assert_eq!(store.range(" cpu ", 0, 2_000).unwrap().len(), 1);
    }

    #[test]
    fn rejects_invalid_inputs() {
        let store = Store::new();
        assert!(store.insert("", 1.0, 1).is_err());
        assert!(store.insert("cpu", f64::NAN, 1).is_err());
        assert!(store.range("cpu", 2, 1).is_err());
    }

    #[test]
    fn empty_store_is_safe() {
        let store = Store::new();
        assert_eq!(store.count().unwrap(), 0);
        assert!(store.get("missing").unwrap().is_empty());
    }
}
