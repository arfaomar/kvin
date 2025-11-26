use kvin::{Key, KvStore, Kvin, Result, Value};

fn main() -> Result<()> {
    println!("=== Example 5: Idiomatic error handling ===");

    let mut db = Kvin::new();

    db.put(Key::from("x"), Value::from(10))?;
    db.put(Key::from("y"), Value::from("hello"))?;

    // A failing operation:
    match db.put(Key::from("x"), Value::from(999)) {
        Ok(_) => println!("unexpected success"),
        Err(e) => println!("expected failure inserting duplicate key: {}", e),
    }

    // Normal usage:
    let y = db.get("y")?;
    println!("borrowed 'y' => {:?}", y);

    let y_owned = db.get_owned("y")?;
    println!("owned 'y' => {:?}", y_owned);

    println!("Example 5 complete.\n");
    Ok(())
}
