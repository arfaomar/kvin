use anyhow::Result;
use kvin::{RAW_HEADER_SIZE, Value, deserialize_borrowed, serialize_value};

fn main() -> Result<()> {
    println!("=== Example 5: Proper Error Handling with anyhow ===\n");

    // --- Serialize a valid value ---
    let mut buf = Vec::new();
    let offset = serialize_value(&mut buf, &Value::from("hello world"));

    println!("Serialized value at offset {}", offset);

    // --- Introduce corruption intentionally ---
    println!("Corrupting encoded buffer...");
    buf[offset + RAW_HEADER_SIZE + 2] ^= 0xAA;

    // --- Pattern-match on Result<Option<T>> ---
    println!("Attempting deserialization:");

    match deserialize_borrowed(&buf[offset..]) {
        Ok(Some((bv, _len))) => {
            println!("[OK] Decoded value: {:?}", bv);
        }
        Ok(None) => {
            println!("[INFO] Not enough data to decode yet");
        }
        Err(e) => {
            println!("[ERROR] Deserialization failed: {}", e);
        }
    }

    println!("\nNow trying with a valid buffer...");

    let mut good_buf = Vec::new();
    let off2 = serialize_value(&mut good_buf, &Value::from("Rustacean"));

    let (bv2, _len2) = deserialize_borrowed(&good_buf[off2..])?.expect("valid entry missing");

    println!("[OK] Successfully decoded value: {:?}", bv2);

    println!("\nExample 5 complete.");
    Ok(())
}
