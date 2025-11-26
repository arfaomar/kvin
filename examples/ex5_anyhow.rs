use kvin::{Kvin, KvStore, Key, Value, Result, Context};
fn main() -> Result<()> {
    println!("=== Example 5: High-level error handling with anyhow ===");

    let mut store = Kvin::new();

    store
        .put(Key::from("user"), Value::from("Omar"))
        .context("failed to insert key 'user'")?;

    store
        .put(Key::from("age"), Value::from(21_i64))
        .context("failed to insert key 'age'")?;

    // Demonstrate normal lookup
    let user = store.get("user")?.context("missing key 'user'")?;
    println!("user => {:?}", user);

    // Show enriched error handling behavior
    println!("\nTrying to update non-existing key…");

    if let Err(e) = store.update(&Key::from("missing"), Value::from("oops")) {
        println!("update error: {}", e);
    }

    // Delete with context
    let deleted = store
        .delete("age")
        .context("delete should never fail structurally")?;

    println!("deleted => {:?}", deleted);

    println!("\nExample 5 complete.\n");
    Ok(())
}
