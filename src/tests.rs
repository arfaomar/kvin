#[cfg(test)]
mod tests {
    use crate::{
        BorrowedValue, Key, KvStore, Kvin, KvinError, RAW_HEADER_SIZE, RawHeader, Value,
        deserialize_borrowed, deserialize_unsafe, serialize_unsafe, serialize_value,
    };

    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_put_and_get() {
        let mut store = Kvin::new();

        store
            .put(Key::from("language"), Value::from("Rust"))
            .unwrap();
        store.put(Key::from(1), Value::from(true)).unwrap();

        // overwrite → KeyExists
        assert!(matches!(
            store.put(Key::from("language"), Value::from("Go")),
            Err(KvinError::KeyExists)
        ));

        assert_eq!(
            store.get("language").unwrap(),
            Some(BorrowedValue::Text("Rust"))
        );
        assert_eq!(store.get(1).unwrap(), Some(BorrowedValue::Bool(true)));

        assert_eq!(
            store.get_owned("language").unwrap(),
            Some(Value::Text("Rust".into()))
        );
        assert_eq!(store.get_owned(1).unwrap(), Some(Value::Bool(true)));

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
        assert!(matches!(
            store.update(&Key::from(999), Value::from("nothing")),
            Err(KvinError::KeyNotFound)
        ));
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

        let (borrowed, _len) = deserialize_borrowed(&buf[off..]).unwrap();
        let decoded = borrowed.to_owned();

        assert_eq!(decoded, original);
    }

    #[test]
    fn test_checksum_detection() {
        let mut buf = Vec::new();
        let off = serialize_value(&mut buf, &Value::from("hello"));

        buf[off + RAW_HEADER_SIZE + 1] ^= 0xFF;

        let err = deserialize_borrowed(&buf[off..]).unwrap_err();

        assert!(matches!(err, KvinError::ChecksumMismatch));
    }

    #[test]
    fn test_unsafe_header_roundtrip() {
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
        let out = unsafe { deserialize_unsafe(&buf).unwrap() };

        // Copy packed fields safely into locals

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

        // KEYS
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

        // VALUES
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

        // ENTRIES
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

    #[test]
    fn test_persistence_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        // initial store
        let mut s = Kvin::new();
        s.put(Key::from("a"), Value::from(1)).unwrap();
        s.put(Key::from("b"), Value::from(true)).unwrap();
        s.put(Key::from(3), Value::from("hello")).unwrap();

        s.save_to_dir(path).unwrap();

        // load a fresh store from disk
        let s2 = Kvin::load_from_dir(path).unwrap();

        // correctness of get / get_owned
        assert_eq!(s2.get("a").unwrap(), Some(BorrowedValue::Int(1)));
        assert_eq!(s2.get("b").unwrap(), Some(BorrowedValue::Bool(true)));
        assert_eq!(s2.get(3).unwrap(), Some(BorrowedValue::Text("hello")));

        assert_eq!(s2.get_owned("a").unwrap(), Some(Value::Int(1)));
        assert_eq!(s2.get_owned("b").unwrap(), Some(Value::Bool(true)));
        assert_eq!(s2.get_owned(3).unwrap(), Some(Value::Text("hello".into())));

        // correctness of keys() and values()
        let mut ks: Vec<String> = s2.keys().map(|k| format!("{:?}", k.unwrap())).collect();
        ks.sort();
        assert_eq!(
            ks,
            vec![
                "Int(3)".to_string(),
                "Text(\"a\")".to_string(),
                "Text(\"b\")".to_string()
            ]
        );

        let mut vs: Vec<String> = s2.values().map(|v| format!("{:?}", v.unwrap())).collect();
        vs.sort();
        assert_eq!(
            vs,
            vec![
                "Bool(true)".to_string(),
                "Int(1)".to_string(),
                "Text(\"hello\")".to_string()
            ]
        );

        // correctness of iter()
        let entries: Vec<_> = s2
            .iter()
            .map(|e| {
                let e = e.unwrap();
                (format!("{:?}", e.key), format!("{:?}", e.value))
            })
            .collect();
        assert_eq!(entries.len(), 3);
    }

    #[test]
    fn test_load_missing_files_returns_empty() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        // no files inside => must return empty Kvin, not error
        let s = Kvin::load_from_dir(path).unwrap();

        assert_eq!(s.get("x").unwrap(), None);
        assert_eq!(s.keys().count(), 0);
        assert_eq!(s.values().count(), 0);
    }

    #[test]
    fn test_corrupted_values_file_checksum_error() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        // Create a valid store and save
        let mut s = Kvin::new();
        s.put(Key::from("k"), Value::from("hello")).unwrap();
        s.save_to_dir(path).unwrap();

        // Corrupt the values file by flipping a byte in the body portion
        let values_path = path.join("kvin.values");
        let mut bytes = fs::read(&values_path).unwrap();
        // flip any byte ≥ header size + 1
        if bytes.len() > RAW_HEADER_SIZE + 1 {
            bytes[RAW_HEADER_SIZE + 1] ^= 0xFF;
        }
        fs::write(&values_path, &bytes).unwrap();

        // Loading must detect corruption either immediately (preferred)
        // or as soon as a value is parsed.
        let err = Kvin::load_from_dir(path).unwrap_err();

        match err {
            KvinError::ChecksumMismatch
            | KvinError::MalformedEntry
            | KvinError::IncompleteBody
            | KvinError::CorruptedIndex => { /* acceptable */ }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn test_corrupted_index_file_fails_to_load() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        // Create valid store and save
        let mut s = Kvin::new();
        s.put(Key::from("a"), Value::from(1)).unwrap();
        s.save_to_dir(path).unwrap();

        // Corrupt the index file by truncating it
        let index_path = path.join("kvin.index");
        let idx = fs::read(&index_path).unwrap();
        let truncated = &idx[..idx.len() / 2]; // half the bytes
        fs::write(&index_path, truncated).unwrap();

        let err = Kvin::load_from_dir(path).unwrap_err();

        match err {
            KvinError::MalformedEntry | KvinError::IncompleteBody | KvinError::CorruptedIndex => { /* acceptable */
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn test_autosave_triggers_at_threshold() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        // Open persistent store (autosave enabled)
        let mut store = Kvin::open(path).unwrap();

        // Lower threshold so autosave triggers quickly
        store.set_autosave_threshold(Some(2));

        // 1st mutation
        store.put(Key::from("a"), Value::from(1)).unwrap();
        // 2nd mutation triggers autosave
        store.put(Key::from("b"), Value::from(2)).unwrap();

        // Both files must exist because autosave triggered
        let values_path = path.join("kvin.values");
        let index_path = path.join("kvin.index");

        assert!(values_path.exists(), "autosave did not write values file");
        assert!(index_path.exists(), "autosave did not write index file");

        // Load fresh store and verify correctness
        let s2 = Kvin::load_from_dir(path).unwrap();
        assert_eq!(s2.get("a").unwrap(), Some(BorrowedValue::Int(1)));
        assert_eq!(s2.get("b").unwrap(), Some(BorrowedValue::Int(2)));
    }

    #[test]
    fn test_autosave_persists_updates_and_deletes() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        let mut store = Kvin::open(path).unwrap();
        store.set_autosave_threshold(Some(2));

        store.put(Key::from("k"), Value::from("old")).unwrap();
        store.update(&Key::from("k"), Value::from("new")).unwrap(); // triggers autosave

        let s2 = Kvin::load_from_dir(path).unwrap();
        assert_eq!(s2.get("k").unwrap(), Some(BorrowedValue::Text("new")));

        store.delete("k").unwrap(); // 1
        store.put(Key::from("x"), Value::from(9)).unwrap(); // 2 → autosave

        let s3 = Kvin::load_from_dir(path).unwrap();
        assert_eq!(s3.get("k").unwrap(), None);
        assert_eq!(s3.get("x").unwrap(), Some(BorrowedValue::Int(9)));
    }

    #[test]
    fn test_autosave_failure_returns_error() {

        #[cfg(unix)]
        let bad_path = std::path::Path::new("/dev/null/kvindir");

        #[cfg(windows)]
        let bad_path = std::path::Path::new("NUL\\kvindir");

        // Opening should succeed if load_from_dir returns Ok(Kvin::new()),
        // because open() does NOT write yet.
        let mut store = Kvin::open(bad_path).unwrap_or_else(|_| {
            // If open() fails due to the directory being invalid,
            // fall back to a manually constructed persistent store.
            let mut s = Kvin::new();
            s.persistence_dir = Some(bad_path.to_path_buf());
            s.autosave_threshold = Some(1); // force autosave next op
            s
        });

        // Force autosave every operation
        store.set_autosave_threshold(Some(1));

        // Now any mutating operation should attempt autosave → fail
        let err = store.put(Key::from("x"), Value::from(1)).unwrap_err();

        match err {
            KvinError::AutosaveFailed(_) => { /* success */ }
            other => panic!("expected AutosaveFailed, got {:?}", other),
        }
    }

    #[test]
    fn test_manual_compaction() {
        let mut s = Kvin::new();

        // Create lots of overwrite garbage
        s.put(Key::from("k"), Value::from("v1")).unwrap();
        s.update(&Key::from("k"), Value::from("v2")).unwrap();
        s.update(&Key::from("k"), Value::from("v3")).unwrap();

        let before = s.buf.len();
        assert!(before > 0);

        // Manual compaction must shrink the buffer
        s.compact().unwrap();
        let after = s.buf.len();

        assert!(after < before);
        assert_eq!(s.get("k").unwrap(), Some(BorrowedValue::Text("v3")));
    }

    #[test]
    fn test_auto_compaction_triggers() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        let mut s = Kvin::open(path).unwrap();
        s.set_autosave_threshold(Some(1));
        s.set_compaction_ratio(Some(1));

        // Write large garbage entries to inflate the buffer
        s.put(Key::from("k"), Value::from("short")).unwrap();

        // Add a much bigger value
        s.update(&Key::from("k"), Value::from("this is a much longer string"))
            .unwrap();

        // Add another long one (creates more garbage)
        s.update(
            &Key::from("k"),
            Value::from("even longer string than before!!!!"),
        )
        .unwrap();

        // Record the inflated size
        let inflated_size = s.buf.len();

        // Trigger autosave + compaction
        s.update(&Key::from("k"), Value::from("final")).unwrap();

        let compacted_size = s.buf.len();

        assert!(
            compacted_size < inflated_size,
            "compacted size {}, inflated size {}",
            compacted_size,
            inflated_size
        );

        // verify correctness
        assert_eq!(s.get("k").unwrap(), Some(BorrowedValue::Text("final")));
    }

    #[test]
    fn test_auto_compaction_no_trigger_with_large_ratio() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        let mut s = Kvin::open(path).unwrap();
        s.set_autosave_threshold(Some(1));
        s.set_compaction_ratio(Some(10_000)); // impossible to reach

        s.put(Key::from("k"), Value::from("v1")).unwrap();
        let before = s.buf.len();

        s.update(&Key::from("k"), Value::from("v2")).unwrap();
        let after = s.buf.len();

        // No compaction should have occurred
        assert!(after > before);
    }

    #[test]
    fn test_compaction_preserves_data() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        let mut s = Kvin::open(path).unwrap();
        s.set_autosave_threshold(Some(1));
        s.set_compaction_ratio(Some(1)); // compact aggressively

        s.put(Key::from(1), Value::from("A")).unwrap();
        s.update(&1.into(), Value::from("B")).unwrap();
        s.update(&1.into(), Value::from("C")).unwrap();

        let s2 = Kvin::load_from_dir(path).unwrap();
        assert_eq!(s2.get(1).unwrap(), Some(BorrowedValue::Text("C")));
    }

    #[test]
    fn test_compaction_after_delete() {
        let mut s = Kvin::new();

        s.put(Key::from("a"), Value::from(1)).unwrap();
        s.put(Key::from("b"), Value::from(2)).unwrap();
        s.delete("a").unwrap();

        let before = s.buf.len();
        s.compact().unwrap();
        let after = s.buf.len();

        assert!(after < before); // removed stale A-entry
        assert_eq!(s.get("b").unwrap(), Some(BorrowedValue::Int(2)));
        assert_eq!(s.get("a").unwrap(), None);
    }

    #[test]
    fn test_auto_compaction_persists_on_next_autosave() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        let mut s = Kvin::open(path).unwrap();
        s.set_autosave_threshold(Some(1)); // every mutation triggers autosave
        s.set_compaction_ratio(Some(1)); // aggressive

        // Make log “dirty” with overwrites (creates garbage).
        s.put(Key::from("k"), Value::from("short")).unwrap();
        s.update(&Key::from("k"), Value::from("this is a much longer string"))
            .unwrap();
        s.update(
            &Key::from("k"),
            Value::from("even longer string than before!!!!"),
        )
        .unwrap();

        let inflated_mem = s.buf.len();
        assert!(inflated_mem > 0);

        // This update triggers autosave, then compaction (in-memory).
        s.update(&Key::from("k"), Value::from("final")).unwrap();

        let compacted_mem = s.buf.len();
        assert!(
            compacted_mem < inflated_mem,
            "expected in-memory compaction to shrink buf (before {}, after {})",
            inflated_mem,
            compacted_mem
        );
        assert_eq!(s.get("k").unwrap(), Some(BorrowedValue::Text("final")));

        // Measure disk size AFTER the autosave that triggered compaction.
        // This disk state corresponds to the save that happened BEFORE compaction.
        let values_path = path.join("kvin.values");
        let disk_after_save_before_compact = fs::metadata(&values_path).unwrap().len() as usize;

        // Force another autosave with a small write. This writes the *compacted* buffer.
        s.put(Key::from("x"), Value::from(1_i64)).unwrap();

        let disk_after_next_save = fs::metadata(&values_path).unwrap().len() as usize;

        // This is the only safe expectation:
        // the next save should not be larger by “a lot” (it should reflect compacted log + small new entry).
        // In most cases it should shrink, but depending on format overhead it could be close.
        assert!(
            disk_after_next_save <= disk_after_save_before_compact + 128,
            "expected next autosave to persist compacted state (before {}, after {})",
            disk_after_save_before_compact,
            disk_after_next_save
        );

        // Reload correctness.
        let s2 = Kvin::load_from_dir(path).unwrap();
        assert_eq!(s2.get("k").unwrap(), Some(BorrowedValue::Text("final")));
        assert_eq!(s2.get("x").unwrap(), Some(BorrowedValue::Int(1)));
    }
}
