use kvin::{
    Key, KvStore, Kvin, RAW_HEADER_SIZE, RawHeader, Value, deserialize_borrowed,
    deserialize_unsafe, serialize_unsafe, serialize_value,
};

fn main() {
    println!("=== Example 3: Unsafe Serialization & Checksum Demo ===\n");

    // --- Part 1: Normal store usage (safe)
    let mut store = Kvin::new();
    store.put(Key::from("language"), Value::from("Rust"));
    store.put(Key::from(1), Value::from(true));

    println!("Stored values:");
    println!("  get(language)  -> {:?}", store.get("language"));
    println!("  get_owned(1)   -> {:?}", store.get_owned(1));

    // --- Part 2: Demonstrate unsafe RawHeader serialization
    println!("\n--- RawHeader serialization with unsafe pointers ---");

    let header = RawHeader {
        total_len: 42,
        checksum: 0xDEADBEEF,
        tag: 2,
    };

    // Safe manual version
    let mut safe_bytes = Vec::new();
    safe_bytes.extend_from_slice(&header.total_len.to_le_bytes());
    safe_bytes.extend_from_slice(&header.checksum.to_le_bytes());
    safe_bytes.push(header.tag);
    println!("Safe header bytes:   {:?}", safe_bytes);

    // Unsafe pointer version
    let mut unsafe_buf = Vec::new();
    unsafe {
        serialize_unsafe(&mut unsafe_buf, &header);
    }
    println!("Unsafe header bytes: {:?}", unsafe_buf);

    // Unsafe deserialization back into a struct
    let parsed = unsafe { deserialize_unsafe(&unsafe_buf) }.unwrap();

    // Copy the packed fields into locals to avoid unaligned references
    let total_len = parsed.total_len;
    let checksum = parsed.checksum;
    let tag = parsed.tag;

    println!(
        "Deserialized RawHeader -> total_len={}, checksum=0x{:X}, tag={}",
        total_len, checksum, tag
    );

    // --- Part 3: Demonstrate checksum verification
    println!("\n--- Checksum verification demo ---");

    let mut buf = Vec::new();
    let offset = serialize_value(&mut buf, &Value::from("ChecksumDemo"));
    println!("Serialized one value ({} bytes total)", buf.len() - offset);

    // Corrupt the serialized payload to trigger checksum failure
    if buf.len() > offset + RAW_HEADER_SIZE + 1 {
        buf[offset + RAW_HEADER_SIZE + 1] ^= 0xFF;
    }

    println!("Attempting to deserialize corrupted data...");
    let result = std::panic::catch_unwind(|| deserialize_borrowed(&buf[offset..]));
    match result {
        Ok(Some((_v, _))) => println!("Decoded value unexpectedly (checksum passed)"),
        Ok(None) => println!("Checksum verification failed (None returned)"),
        Err(_) => println!("Checksum mismatch caused panic as expected"),
    }

    println!("\nExample 3 complete.");
}
