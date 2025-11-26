use kvin::{Key, KvStore, Kvin, Result, Value};

fn main() -> Result<()> {
    println!("=== Example 2: Borrowed vs Owned ===");

    let mut store = Kvin::new();
    store.put(Key::from("msg"), Value::from("Hello"))?;

    // Borrowed value (zero-copy)
    if let Some(b) = store.get("msg")? {
        println!("borrowed: {:?}", b);
    }

    // Owned copy
    if let Some(o) = store.get_owned("msg")? {
        println!("owned: {:?}", o);
    }

    println!("Example 2 complete.\n");
    Ok(())
}
