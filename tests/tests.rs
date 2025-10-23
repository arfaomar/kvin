#[cfg(test)]
mod tests {
    use kvin::{Kvin, Key, Value, KvStore};

#[test]
    fn test_put_and_get() {
        let mut store = Kvin::new();
        store.put(Key::from("language"), Value::from("Rust"));
        store.put(Key::from(1), Value::from(true));

        assert_eq!(
            store.get(&Key::from("language")),
            Some(&Value::Text("Rust".into()))
        );
        assert_eq!(store.get(&Key::from(1)), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_update_existing() {
        let mut store = Kvin::new();
        store.put(Key::from(42), Value::from("life"));
        assert!(store.update(&Key::from(42), Value::from("universe")));
        assert_eq!(
            store.get(&Key::from(42)),
            Some(&Value::Text("universe".into()))
        );
    }

    #[test]
    fn test_update_non_existing() {
        let mut store = Kvin::new();
        assert!(!store.update(&Key::from(999), Value::from("nothing")));
    }

    #[test]
    fn test_delete() {
        let mut store = Kvin::new();
        store.put(Key::from("remove"), Value::from(123));
        assert_eq!(store.delete(&Key::from("remove")), Some(Value::Int(123)));
        assert_eq!(store.get(&Key::from("remove")), None);
    }
}
