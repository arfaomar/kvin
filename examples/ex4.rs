use kvin::{Kvin, KvStore, Key, Value, Result};

fn main() -> Result<()> {
    println!("=== Example 4: Iterators over Entries, Keys, and Values ===");

    let mut store = Kvin::new();
    store.put(Key::from("a"), Value::from(1_i64))?;
    store.put(Key::from("b"), Value::from(true))?;
    store.put(Key::from(3), Value::from("x"))?;

    println!("--- Entries via iter() ---");
    for entry_res in store.iter() {
        match entry_res {
            Ok(e) => println!("  {:?} => {:?}", e.key, e.value),
            Err(e) => println!("  error decoding entry: {}", e),
        }
    }

    println!("\n--- Keys ---");
    for key_res in store.keys() {
        match key_res {
            Ok(k) => println!("  {:?}", k),
            Err(e) => println!("  error: {}", e),
        }
    }

    println!("\n--- Values ---");
    for val_res in store.values() {
        match val_res {
            Ok(v) => println!("  {:?}", v),
            Err(e) => println!("  error: {}", e),
        }
    }

    println!("\nExample 4 complete.\n");
    Ok(())
}
