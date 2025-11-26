use anyhow::{Context, Result};
use kvin::{Key, KvStore, Kvin, Value};

fn main() -> Result<()> {
    println!("=== Example 5: Error Handling with anyhow (User Perspective) ===\n");

    // --- Part 1: Basic usage (no errors expected)
    let mut store = Kvin::new();
    store.put(Key::from("lang"), Value::from("Rust"));
    store.put(Key::from("year"), Value::from(2025_i64));
    store.put(Key::from("safe"), Value::from(true));

    println!("Stored a few values safely.");
    println!("  get(lang) -> {:?}", store.get("lang"));
    println!("  get(year) -> {:?}", store.get("year"));
    println!("  get(safe) -> {:?}", store.get("safe"));

    // --- Part 2: Demonstrate that normal usage never returns Result
    println!("\n--- Normal Kvin usage does NOT expose Result ---");
    println!("Users only see `Option<T>`:");
    println!("  get(nonexistent) -> {:?}", store.get("does_not_exist"));

    // --- Part 3: Show anyhow-style error handling by manually corrupting data
    println!("\n--- Introducing corruption to demonstrate anyhow error ---");

    // We serialize a clean value manually (users normally do not do this)
    let mut buf = Vec::new();
    let off = kvin::serialize_value(&mut buf, &Value::from("CorruptMe"));

    // Corrupt one byte after the header
    let header_size = kvin::RAW_HEADER_SIZE;
    buf[off + header_size + 1] ^= 0xFF;

    println!("Corrupted a serialized entry intentionally.");

    println!("\nAttempting to decode corrupted data...");

    match kvin::deserialize_borrowed(&buf[off..]) {
        Ok((val, _len)) => {
            println!("Decoded value: {:?}", val);
        }
        Err(e) => {
            println!("Error decoding value: {}", e);
        }
    }

    // --- Part 4: Normal user-facing error handling with context()
    println!("\n--- Normal usage is still clean and ergonomic ---");

    store.put(Key::from("demo"), Value::from("success"));
    let val = store
        .get_owned("demo")
        .context("expected key 'demo' in store")?;

    println!("Successfully retrieved owned value: {:?}", val);

    println!("\nExample 5 complete.");
    Ok(())
}
