use kvin::{Kvin, KvStore, Key, Value, Result};

fn main() -> Result<()> {
    println!("=== Example 2: Borrowed vs Owned Values ===");

    let mut store = Kvin::new();

    store.put(Key::from("name"), Value::from("Kvin"))?;
    store.put(Key::from("count"), Value::from(123_i64))?;

    // Borrowed access
    if let Some(b) = store.get("name")? {
        println!("borrowed name: {:?}", b);
    }

    // Owned access
    if let Some(v) = store.get_owned("name")? {
        println!("owned name: {:?}", v);
    }

    println!("\nUpdating the name key…");
    store.update(&Key::from("name"), Value::from("Updated"))?;

    if let Some(b) = store.get("name")? {
        println!("borrowed after update: {:?}", b);
    }

    if let Some(v) = store.get_owned("name")? {
        println!("owned after update: {:?}", v);
    }

    println!("Example 2 complete.\n");
    Ok(())
}
