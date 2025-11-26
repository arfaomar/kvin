use kvin::{
    Key, KvStore, Kvin, Value, Result, deserialize_borrowed,
    deserialize_unsafe, serialize_unsafe, serialize_value,
    RAW_HEADER_SIZE, RawHeader,
};

fn main() -> Result<()> {
    println!("=== Example 3: Unsafe Serialization & Checksum Demo ===");

    let mut store = Kvin::new();
    store.put(Key::from("safe"), Value::from(true))?;
    store.put(Key::from("msg"), Value::from("Hello"))?;

    println!("get(msg) => {:?}", store.get("msg")?);

    println!("\n--- RawHeader unsafe roundtrip ---");

    let header = RawHeader {
        total_len: 42,
        checksum: 0xDEADBEEF,
        tag: 2,
    };

    let mut buf = Vec::new();
    unsafe {
        serialize_unsafe(&mut buf, &header);
    }

    println!("serialized header bytes: {:?}", buf);

    // Unsafe deserialize
    let parsed = unsafe { deserialize_unsafe(&buf)? };

    // FIX: Copy packed fields safely into locals
    let total_len = parsed.total_len;
    let checksum = parsed.checksum;
    let tag = parsed.tag;

    println!(
        "deserialized => total_len={}, checksum=0x{:X}, tag={}",
        total_len, checksum, tag
    );

    println!("\n--- Checksum verification ---");

    let mut buf = Vec::new();
    let off = serialize_value(&mut buf, &Value::from("ChecksumDemo"));

    buf[off + RAW_HEADER_SIZE + 1] ^= 0xFF; // corrupt payload

    match deserialize_borrowed(&buf[off..]) {
        Ok((_v, _)) => println!("ERROR: checksum unexpectedly passed"),
        Err(e) => println!("checksum error detected => {}", e),
    }

    println!("Example 3 complete.\n");
    Ok(())
}
