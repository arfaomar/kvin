use kvin::{
    Key, KvStore, Kvin, RAW_HEADER_SIZE, RawHeader, Value, deserialize_borrowed,
    deserialize_unsafe, serialize_unsafe, serialize_value,
};

fn main() {
    println!("=== Example 3: Unsafe Serialization & Checksum Demo ===\n");

    let mut store = Kvin::new();
    store.put(Key::from("language"), Value::from("Rust"));
    store.put(Key::from(1), Value::from(true));

    println!("Stored values:");
    println!("  get(language)  -> {:?}", store.get("language").unwrap());
    println!("  get_owned(1)   -> {:?}", store.get_owned(1).unwrap());

    println!("\n--- RawHeader serialization with unsafe pointers ---");

    let header = RawHeader {
        total_len: 42,
        checksum: 0xDEADBEEF,
        tag: 2,
    };

    let mut safe_bytes = Vec::new();
    safe_bytes.extend_from_slice(&header.total_len.to_le_bytes());
    safe_bytes.extend_from_slice(&header.checksum.to_le_bytes());
    safe_bytes.push(header.tag);
    println!("Safe header bytes:   {:?}", safe_bytes);

    let mut unsafe_buf = Vec::new();
    unsafe {
        serialize_unsafe(&mut unsafe_buf, &header);
    }
    println!("Unsafe header bytes: {:?}", unsafe_buf);

    let parsed = unsafe { deserialize_unsafe(&unsafe_buf).unwrap() };

    let out_total_len = parsed.total_len;
    let out_checksum = parsed.checksum;
    let out_tag = parsed.tag;

    println!(
        "Deserialized RawHeader -> total_len={}, checksum=0x{:X}, tag={}",
        out_total_len, out_checksum, out_tag
    );

    println!("\n--- Checksum verification demo ---");

    let mut buf = Vec::new();
    let offset = serialize_value(&mut buf, &Value::from("ChecksumDemo"));
    println!("Serialized one value ({} bytes total)", buf.len() - offset);

    if buf.len() > offset + RAW_HEADER_SIZE + 1 {
        buf[offset + RAW_HEADER_SIZE + 1] ^= 0xFF;
    }

    println!("Attempting to deserialize corrupted data...");

    match deserialize_borrowed(&buf[offset..]) {
        Ok((v, _)) => println!("Decoded value unexpectedly: {:?}", v),
        Err(e) => println!("Checksum mismatch detected: {}", e),
    }

    println!("\nExample 3 complete.");
}
