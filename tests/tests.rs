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

        // get() now returns BorrowedValue (borrowed access)
        assert_eq!(store.get("language"), Some(BorrowedValue::Text("Rust")));
        assert_eq!(store.get(1), Some(BorrowedValue::Bool(true)));

        // get_owned() explicitly clones into Value
        assert_eq!(
            store.get_owned("language"),
            Some(Value::Text("Rust".into()))
        );
        assert_eq!(store.get_owned(1), Some(Value::Bool(true)));

        // Check get_borrowed() directly (same as get)
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
        assert_eq!(store.get(42), Some(BorrowedValue::Text("universe")));
        assert_eq!(store.get_owned(42), Some(Value::Text("universe".into())));
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

        assert_eq!(store.delete("remove"), Some(Value::Int(123)));

        // Should now be gone in both get() and get_owned()
        assert_eq!(store.get("remove"), None);
        assert_eq!(store.get_owned("remove"), None);
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

#[test]
#[should_panic(expected = "checksum mismatch")]
fn test_checksum_detection() {
    let mut buf = Vec::new();
    let off = kvin::serialize_value(&mut buf, &kvin::Value::from("hello"));
    // corrupt one byte in the body
    buf[off + kvin::RAW_HEADER_SIZE + 1] ^= 0xFF;
    // should panic now
    let _ = kvin::deserialize_borrowed(&buf[off..]);
}

#[test]
fn test_unsafe_header_roundtrip() {
    use kvin::{RAW_HEADER_SIZE, RawHeader, serialize_unsafe};
    let h = RawHeader {
        total_len: 42,
        checksum: 0xABCD1234,
        tag: 7,
    };
    let mut buf = Vec::new();
    unsafe {
        serialize_unsafe(&mut buf, &h);
    }
    assert_eq!(buf.len(), RAW_HEADER_SIZE);
    let out = unsafe { kvin::deserialize_unsafe(&buf).unwrap() };

    // Copy packed fields safely into locals
    let total_len_out = out.total_len;
    let checksum_out = out.checksum;
    let tag_out = out.tag;

    let total_len_h = h.total_len;
    let checksum_h = h.checksum;
    let tag_h = h.tag;

    assert_eq!(total_len_out, total_len_h);
    assert_eq!(checksum_out, checksum_h);
    assert_eq!(tag_out, tag_h);
}

#[test]
#[should_panic(expected = "checksum mismatch")]
fn test_checksum_mismatch_panics() {
    let mut buf = Vec::new();
    let off = kvin::serialize_value(&mut buf, &kvin::Value::from("hello"));
    // corrupt one byte
    buf[off + kvin::RAW_HEADER_SIZE + 1] ^= 0xFF;
    let _ = kvin::deserialize_borrowed(&buf[off..]);
}
