#[cfg(test)]
mod tests {
    use kvin::{BorrowedValue, Key, KvStore, Kvin, Value};

    #[test]
    fn test_put_and_get() {
        let mut store = Kvin::new();
        assert!(store.put(Key::from("language"), Value::from("Rust")));
        assert!(store.put(Key::from(1), Value::from(true)));

        // Attempt to overwrite existing key should fail
        assert!(!store.put(Key::from("language"), Value::from("Go")));

        // 🔹 Check get() (returns owned Value)
        assert_eq!(
            store.get(&Key::from("language")),
            Some(Value::Text("Rust".into()))
        );
        assert_eq!(store.get(&Key::from(1)), Some(Value::Bool(true)));

        // 🔹 Check get_owned() explicitly (same behavior)
        assert_eq!(
            store.get_owned(&Key::from("language")),
            Some(Value::Text("Rust".into()))
        );
        assert_eq!(store.get_owned(&Key::from(1)), Some(Value::Bool(true)));

        // 🔹 Check borrowed version (zero-copy access)
        assert_eq!(
            store.get_borrowed(&Key::from("language")),
            Some(BorrowedValue::Text("Rust"))
        );
    }

    #[test]
    fn test_update_existing() {
        let mut store = Kvin::new();
        assert!(store.put(Key::from(42), Value::from("life")));
        assert!(store.update(&Key::from(42), Value::from("universe")));

        // Both get() and get_owned() must see the new value
        assert_eq!(
            store.get(&Key::from(42)),
            Some(Value::Text("universe".into()))
        );
        assert_eq!(
            store.get_owned(&Key::from(42)),
            Some(Value::Text("universe".into()))
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

        // Should now be gone in both get() and get_owned()
        assert_eq!(store.get(&Key::from("remove")), None);
        assert_eq!(store.get_owned(&Key::from("remove")), None);
    }

    #[test]
    fn test_roundtrip_serialization() {
        use kvin::{deserialize_borrowed, serialize_value};

        let mut buf = Vec::new();
        let original = Value::from("Hello Kvin!");
        let off = serialize_value(&mut buf, &original);

        let (borrowed, _len) = deserialize_borrowed(&buf[off..]).expect("deserialize failed");
        let decoded = borrowed.to_owned();

        assert_eq!(decoded, original);
    }
}
