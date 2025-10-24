#[cfg(test)]
mod tests {
    use kvin::{Key, KvStore, Kvin, Value};

    #[test]
    fn test_put_and_get() {
        let mut store = Kvin::new();
        assert!(store.put(Key::from("language"), Value::from("Rust")));
        assert!(store.put(Key::from(1), Value::from(true)));

        // Versuch, existierenden Schlüssel zu überschreiben
        assert!(!store.put(Key::from("language"), Value::from("Go")));
        assert_eq!(
            store.get(&Key::from("language")),
            Some(&Value::Text("Rust".into()))
        );
        assert_eq!(store.get(&Key::from(1)), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_update_existing() {
        let mut store = Kvin::new();
        assert!(store.put(Key::from(42), Value::from("life")));
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
        assert!(store.put(Key::from("remove"), Value::from(123)));
        assert_eq!(store.delete(&Key::from("remove")), Some(Value::Int(123)));
        assert_eq!(store.get(&Key::from("remove")), None);
    }
}
