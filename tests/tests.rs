#[cfg(test)]
mod tests {
    use kvin::{
        BorrowedValue, Key, KvStore, Kvin, RAW_HEADER_SIZE, Value, deserialize_borrowed,
        serialize_value,
    };

    #[test]
    fn test_put_and_get() {
        let mut store = Kvin::new();

        // put() now returns Result<()>
        store
            .put(Key::from("language"), Value::from("Rust"))
            .unwrap();
        store.put(Key::from(1), Value::from(true)).unwrap();

        // Attempt to overwrite existing key should fail
        assert!(store.put(Key::from("language"), Value::from("Go")).is_err());

        // get()
        assert_eq!(
            store.get("language").unwrap(),
            Some(BorrowedValue::Text("Rust"))
        );
        assert_eq!(store.get(1).unwrap(), Some(BorrowedValue::Bool(true)));

        // get_owned()
        assert_eq!(
            store.get_owned("language").unwrap(),
            Some(Value::Text("Rust".into()))
        );
        assert_eq!(store.get_owned(1).unwrap(), Some(Value::Bool(true)));

        // get_borrowed()
        assert_eq!(
            store.get_borrowed(&Key::from("language")).unwrap(),
            Some(BorrowedValue::Text("Rust"))
        );
    }

    #[test]
    fn test_update_existing() {
        let mut store = Kvin::new();

        store.put(Key::from(42), Value::from("life")).unwrap();
        store
            .update(&Key::from(42), Value::from("universe"))
            .unwrap();

        assert_eq!(
            store.get(42).unwrap(),
            Some(BorrowedValue::Text("universe"))
        );
        assert_eq!(
            store.get_owned(42).unwrap(),
            Some(Value::Text("universe".into()))
        );
    }

    #[test]
    fn test_update_non_existing() {
        let mut store = Kvin::new();
        // previously returned false, now returns Err
        assert!(
            store
                .update(&Key::from(999), Value::from("nothing"))
                .is_err()
        );
    }

    #[test]
    fn test_delete() {
        let mut store = Kvin::new();

        store.put(Key::from("remove"), Value::from(123)).unwrap();

        assert_eq!(store.delete("remove").unwrap(), Some(Value::Int(123)));

        assert_eq!(store.get("remove").unwrap(), None);
        assert_eq!(store.get_owned("remove").unwrap(), None);
    }

    #[test]
    fn test_roundtrip_serialization() {
        let mut buf = Vec::new();
        let original = Value::from("Hello Kvin!");
        let off = serialize_value(&mut buf, &original);

        let (borrowed, _len) = deserialize_borrowed(&buf[off..]).expect("deserialize failed");
        let decoded = borrowed.to_owned();

        assert_eq!(decoded, original);
    }

    #[test]
    fn test_checksum_detection() {
        let mut buf = Vec::new();
        let off = serialize_value(&mut buf, &Value::from("hello"));

        buf[off + RAW_HEADER_SIZE + 1] ^= 0xFF;

        let err = deserialize_borrowed(&buf[off..]).unwrap_err();
        assert!(err.to_string().contains("checksum mismatch"));
    }

    #[test]
    fn test_unsafe_header_roundtrip() {
        use kvin::{RAW_HEADER_SIZE, RawHeader, deserialize_unsafe, serialize_unsafe};

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
        let out = unsafe { deserialize_unsafe(&buf).expect("deserialize failed") };

        // Copy packed fields to avoid unaligned references
        let h_total_len = h.total_len;
        let h_checksum = h.checksum;
        let h_tag = h.tag;

        let out_total_len = out.total_len;
        let out_checksum = out.checksum;
        let out_tag = out.tag;

        assert_eq!(out_total_len, h_total_len);
        assert_eq!(out_checksum, h_checksum);
        assert_eq!(out_tag, h_tag);
    }

    #[test]
    fn test_iter_keys_values() {
        let mut s = Kvin::new();

        s.put(Key::from("a"), Value::from(1)).unwrap();
        s.put(Key::from("b"), Value::from(true)).unwrap();
        s.put(Key::from(3), Value::from("x")).unwrap();

        let mut ks: Vec<String> = s.keys().map(|k| format!("{:?}", k.unwrap())).collect();
        ks.sort();
        assert_eq!(
            ks,
            vec![
                "Int(3)".to_string(),
                "Text(\"a\")".to_string(),
                "Text(\"b\")".to_string()
            ]
        );

        let mut vs: Vec<String> = s.values().map(|v| format!("{:?}", v.unwrap())).collect();
        vs.sort();
        assert_eq!(
            vs,
            vec![
                "Bool(true)".to_string(),
                "Int(1)".to_string(),
                "Text(\"x\")".to_string()
            ]
        );

        let mut entries: Vec<(String, String)> = s
            .iter()
            .map(|e| {
                let e = e.unwrap();
                (format!("{:?}", e.key), format!("{:?}", e.value))
            })
            .collect();
        entries.sort();
        assert_eq!(entries.len(), 3);
    }

    #[test]
    fn iter_entries_zero_copy() {
        let mut s = Kvin::new();

        s.put(Key::from("a"), Value::from(1)).unwrap();
        s.put(Key::from(2), Value::from("x")).unwrap();

        let got: Vec<String> = s
            .iter()
            .map(|res| {
                let e = res.unwrap();
                format!("{:?}:{:?}", e.key, e.value)
            })
            .collect();

        assert!(
            got.iter()
                .any(|l| l.contains("Int(2)") && l.contains("Text(\"x\")"))
        );
        assert!(
            got.iter()
                .any(|l| l.contains("Text(\"a\")") && l.contains("Int(1)"))
        );
    }

    #[test]
    fn keys_and_values_iter() {
        let mut store = Kvin::new();

        store.put(Key::from("lang"), Value::from("Rust")).unwrap();
        store.put(Key::from("ver"), Value::from(2025_i64)).unwrap();
        store.put(Key::from("safe"), Value::from(true)).unwrap();

        let mut keys: Vec<String> = store.keys().map(|k| format!("{:?}", k.unwrap())).collect();
        keys.sort();

        let mut values: Vec<String> = store
            .values()
            .map(|v| format!("{:?}", v.unwrap()))
            .collect();
        values.sort();

        assert_eq!(
            keys,
            vec![
                r#"Text("lang")"#.to_string(),
                r#"Text("safe")"#.to_string(),
                r#"Text("ver")"#.to_string(),
            ]
        );
        assert_eq!(
            values,
            vec![
                r#"Bool(true)"#.to_string(),
                r#"Int(2025)"#.to_string(),
                r#"Text("Rust")"#.to_string(),
            ]
        );
    }
}
