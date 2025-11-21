use kvin::{Key, KvStore, Kvin, Value};

fn main() {
    println!("=== Example 4: Iterators over Entries, Keys, and Values ===\n");

    // --- Part 1: Create and populate the store
    let mut store = Kvin::new();
    store.put(Key::from("a"), Value::from(1));
    store.put(Key::from("b"), Value::from(true));
    store.put(Key::from(3), Value::from("x"));

    println!("Initial store contents inserted.\n");

    // --- Part 2: Iterate over entries (key + value)
    println!("--- Iterating over entries with `iter()` ---");
    for entry in store.iter() {
        match entry {
            Ok(e) => println!("  OK: {:?} => {:?}", e.key, e.value),
            Err(err) => println!("  ERROR: {}", err),
        }
    }

    // --- Part 3: Iterate over keys
    println!("\n--- Iterating over keys with `keys()` ---");
    for key in store.keys() {
        println!("  {:?}", key);
    }

    // --- Part 4: Iterate over values
    println!("\n--- Iterating over values with `values()` ---");
    for value in store.values() {
        println!("  {:?}", value);
    }

    // --- Part 5: Collect and summarize
    println!("\n--- Collecting results ---");
    let keys: Vec<String> = store.keys().map(|k| format!("{:?}", k)).collect();
    let values: Vec<String> = store.values().map(|v| format!("{:?}", v)).collect();

    println!("All keys   : {:?}", keys);
    println!("All values : {:?}", values);

    println!("\nExample 4 complete.\n");
}
