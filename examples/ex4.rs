use kvin::{Key, KvStore, Kvin, Result, Value};

fn main() -> Result<()> {
    println!("=== Example 4: Iterating keys/values/entries ===");

    let mut s = Kvin::new();
    s.put(Key::from("a"), Value::from(1))?;
    s.put(Key::from("b"), Value::from(true))?;
    s.put(Key::from(3), Value::from("x"))?;

    println!("-- entries --");
    for entry_res in s.iter() {
        let e = entry_res?;
        println!("{:?} => {:?}", e.key, e.value);
    }

    println!("\n-- keys --");
    for k in s.keys() {
        println!("{:?}", k?);
    }

    println!("\n-- values --");
    for v in s.values() {
        println!("{:?}", v?);
    }

    println!("Example 4 complete.\n");
    Ok(())
}
