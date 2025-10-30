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

        // Use get_owned() because get() panics in Lab 3
        assert_eq!(
            store.get_owned(&Key::from("language")),
            Some(Value::Text("Rust".into()))
        );
        assert_eq!(store.get_owned(&Key::from(1)), Some(Value::Bool(true)));

        // Borrowed variant should return same data zero-copy
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
