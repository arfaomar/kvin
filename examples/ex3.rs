use kvin::{
    Key, KvStore, Kvin, RAW_HEADER_SIZE, RawHeader, Result, Value, deserialize_borrowed,
    deserialize_unsafe, serialize_unsafe, serialize_value,
};

fn main() -> Result<()> {
    println!("=== Example 3: Unsafe header demo + checksum ===");

    // Part 1: Normal usage
    let mut store = Kvin::new();
    store.put(Key::from("safe"), Value::from(true))?;
    store.put(Key::from("msg"), Value::from("Hello"))?;

    println!("safe -> {:?}", store.get("safe")?);

    // Part 2: RawHeader unsafe roundtrip
    let header = RawHeader {
        total_len: 42,
        checksum: 0xDEADBEEF,
        tag: 2,
    };

    let mut buf = Vec::new();
    unsafe {
        serialize_unsafe(&mut buf, &header);
    }

    println!("Serialized RawHeader bytes: {:?}", buf);

    let parsed = unsafe { deserialize_unsafe(&buf)? };

    // Copy packed fields BEFORE printing
    let h_len = header.total_len;
    let h_crc = header.checksum;
    let h_tag = header.tag;

    let p_len = parsed.total_len;
    let p_crc = parsed.checksum;
    let p_tag = parsed.tag;

    println!(
        "Roundtrip RawHeader: len={}, crc=0x{:X}, tag={}",
        p_len, p_crc, p_tag
    );

    assert_eq!(h_len, p_len);
    assert_eq!(h_crc, p_crc);
    assert_eq!(h_tag, p_tag);

    // Part 3: Checksum detection
    let mut buf = Vec::new();
    let off = serialize_value(&mut buf, &Value::from("Checksum"));

    // Corrupt 1 byte
    buf[off + RAW_HEADER_SIZE + 1] ^= 0xFF;

    println!("Attempting to decode corrupted entry...");

    match deserialize_borrowed(&buf[off..]) {
        Ok((_, _)) => println!("ERROR: checksum passed unexpectedly"),
        Err(e) => println!("Checksum error caught: {}", e),
    }

    println!("Example 3 complete.\n");
    Ok(())
}
