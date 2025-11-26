use kvin::{Key, KvStore, Kvin, Result, Value};

fn main() -> Result<()> {
    println!("=== Example 1: Basic usage ===");

    let mut store = Kvin::new();

    store.put(Key::from("lang"), Value::from("Rust"))?;
    store.put(Key::from(1), Value::from(true))?;

    println!("lang -> {:?}", store.get("lang")?);
    println!("1 -> {:?}", store.get(1)?);

    if let Some(v) = store.delete("lang")? {
        println!("deleted lang -> {:?}", v);
    }

    println!("Example 1 complete.\n");
    Ok(())
}
