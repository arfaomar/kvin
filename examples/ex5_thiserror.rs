use kvin::{Key, KvStore, Kvin, Value, KvinError};

fn main() {
    println!("=== Example 5: Error Handling with thiserror (User & Developer Perspective) ===\n");

    // --- Part 1: Normal store usage (safe)
    let mut store = Kvin::new();
    store.put(Key::from("lang"), Value::from("Rust"));
    store.put(Key::from("year"), Value::from(2025_i64));
    store.put(Key::from("safe"), Value::from(true));

    println!("Stored a few values safely.");
    println!("  get(lang) -> {:?}", store.get("lang"));
    println!("  get(year) -> {:?}", store.get("year"));
    println!("  get(safe) -> {:?}", store.get("safe"));

    // --- Part 2: Demonstrate that normal usage does not expose Result
    println!("\n--- Normal Kvin usage does NOT expose Result ---");
    println!("Users only see `Option<T>` – exactly as before:");
    println!("  get(nonexistent) -> {:?}", store.get("does_not_exist"));
    println!("  No breaking change – API remains 100% compatible!\n");

    // --- Part 3:
    println!("--- Introducing corruption to demonstrate thiserror power ---\n");

    // Serialize a value
    let mut buf = Vec::new();
    let off = kvin::serialize_value(&mut buf, &Value::from("This entry will be corrupted"));

    // Corrupt one byte in the payload to trigger checksum failure
    let header_size = kvin::RAW_HEADER_SIZE;
    if buf.len() > off + header_size + 5 {
        buf[off + header_size + 5] ^= 0xFF;
    }

    println!("Corrupted a serialized entry intentionally (flipped one byte in payload).");

    println!("\nAttempting to decode corrupted data...");
    match kvin::deserialize_borrowed(&buf[off..]) {
        Ok((val, _)) => {
            println!("Unexpectedly succeeded (checksum somehow passed?): {:?}", val);
        }
        Err(KvinError::ChecksumMismatch) => {
            println!("Perfect! Corruption detected cleanly – no panic!");
            println!("  -> KvinError::ChecksumMismatch");
            println!("     Precise, structured, debuggable error – exactly what we want.");
        }
        Err(err) => {
            println!("Other internal error occurred: {}", err);
        }
    }

    // --- Part 4: Load corrupted entry into a real store
    println!("\n--- Now loading the corrupted entry into a real store ---");

    // Create a new store and put a good entry
    let mut corrupted_store = Kvin::new();
    corrupted_store.put("good".into(), "I'm fine".into());

    // Append the corrupted entry directly into the store's buffer
    let slice = corrupted_store.buf.as_mut_slice();
    if slice.len() > 100 {
        let corrupt_start = off.min(slice.len().saturating_sub(buf.len() - off));
        if corrupt_start + (buf.len() - off) <= slice.len() {
            slice[corrupt_start..corrupt_start + (buf.len() - off)]
                .copy_from_slice(&buf[off..]);
        }
    }

    println!("Store now contains one corrupted entry.\n");

    println!("store.get(\"good\") -> {:?}", corrupted_store.get("good")); // could be none
    println!("But the iterator reveals the truth:\n");

    for entry in &corrupted_store {
        match entry {
            Ok(e) => println!("  Healthy Entry: {:?} => {:?}", e.key, e.value),
            Err(KvinError::ChecksumMismatch) => {
                println!("  Corrupted Entry detected! -> KvinError::ChecksumMismatch");
                println!("     The store stays usable – no crash, no hidden corruption!");
            }
            Err(err) => println!("  Other error: {}", err),
        }
    }
}