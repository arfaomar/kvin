use kvin::{Kvin, KvStore, Key, Value, Result};

fn main() -> Result<()> {
    println!("=== Example 1: Basic Put/Get/Update/Delete ===");

    let mut store = Kvin::new();

    // Insert values
    store.put(Key::from("language"), Value::from("Rust"))?;
    store.put(Key::from("year"), Value::from(2025_i64))?;

    // Read borrowed values
    if let Some(v) = store.get("language")? {
        println!("get(language) => {:?}", v);
    }

    // Update existing key
    store.update(&Key::from("language"), Value::from("Rustacean"))?;

    if let Some(v) = store.get("language")? {
        println!("after update => {:?}", v);
    }

    // Delete key
    if let Some(v) = store.delete("year")? {
        println!("deleted(year) => {:?}", v);
    }

    // Attempt to read deleted key
    println!("get(year) => {:?}", store.get("year")?);

    println!("Example 1 complete.\n");
    Ok(())
}
