use kvin::{Key, KvStore, Kvin, Value};

fn main() {
    println!("=== Example 4: Iterators over Entries, Keys, and Values ===\n");

    let mut store = Kvin::new();
    store.put(Key::from("a"), Value::from(1));
    store.put(Key::from("b"), Value::from(true));
    store.put(Key::from(3), Value::from("x"));

    println!("Initial store contents inserted.\n");

    println!("--- Iterating over entries with `iter()` ---");
    for entry_res in store.iter() {
        match entry_res {
            Ok(entry) => println!("  {:?} => {:?}", entry.key, entry.value),
            Err(e) => println!("  [error decoding entry: {}]", e),
        }
    }

    println!("\n--- Iterating over keys with `keys()` ---");
    for key_res in store.keys() {
        match key_res {
            Ok(key) => println!("  {:?}", key),
            Err(e) => println!("  [error: {}]", e),
        }
    }

    println!("\n--- Iterating over values with `values()` ---");
    for val_res in store.values() {
        match val_res {
            Ok(v) => println!("  {:?}", v),
            Err(e) => println!("  [error: {}]", e),
        }
    }

    println!("\nExample 4 complete.\n");
}
