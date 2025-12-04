use kvin::{BorrowedValue, Key, KvStore, Kvin, Result, Value};
use tempfile::tempdir;

fn main() -> Result<()> {
    println!("=== Example 6: Persistence + Autosave (Tempfile Sandbox) ===");

    // ----------------------------------------------------------------------
    // NOTE FOR USERS:
    //
    // This example uses a temporary directory created by `tempfile`.
    // It is automatically deleted at program exit.
    //
    // To persist data on your own machine across runs, replace:
    //
    //     let dir = tempdir()?;
    //     let path = dir.path();
    //
    // with:
    //
    //     let path = std::path::Path::new("data");
    //     std::fs::create_dir_all(path)?;
    //
    // and call `Kvin::open(path)` to enable autosave.
    // ----------------------------------------------------------------------
    let dir = tempdir()?;
    let path = dir.path();

    println!("Using temporary directory: {}", path.display());

    // ----------------------------------------------------------------------
    // Create a *persistent* Kvin store using `Kvin::open()`.
    //
    // This differs from `Kvin::new()`:
    //   - It automatically loads existing data (if present).
    //   - It enables autosave by default (threshold = 10).
    // ----------------------------------------------------------------------
    let mut store = Kvin::open(path)?;
    println!("Persistent store opened (autosave enabled).");

    // ----------------------------------------------------------------------
    // OPTIONAL: Adjust autosave threshold.
    //
    // Setting it lower lets us see autosave happen during this example.
    //
    // For demonstration, we set autosave threshold to 2,
    // meaning: every 2 mutating operations -> store is automatically saved.
    // ----------------------------------------------------------------------
    store.set_autosave_threshold(Some(2));
    println!("Autosave threshold set to 2 operations.");

    // ----------------------------------------------------------------------
    // Perform several operations to trigger autosave.
    //
    // Operation count:
    //   1) put foo
    //   2) put n
    // Autosave happens here.
    //
    //   3) update foo
    //   4) delete n
    // Autosave happens again.
    // ----------------------------------------------------------------------
    store.put(Key::from("foo"), Value::from("bar"))?;
    store.put(Key::from("n"), Value::from(42_i64))?;
    println!("Performed 2 operations -> autosave should have occurred.");

    store.update(&Key::from("foo"), Value::from("BAR"))?;
    store.delete("n")?;
    println!("Performed 2 more operations -> autosave should have occurred again.");

    // ----------------------------------------------------------------------
    // Simulate a restart by loading a fresh store.
    //
    // Since autosave was active, these writes should already be persisted.
    // ----------------------------------------------------------------------
    let store2 = Kvin::open(path)?;
    println!("Store reloaded from disk after autosaves.");

    if let Some(BorrowedValue::Text(s)) = store2.get("foo")? {
        println!("foo after reload = {s}");
    }

    // ----------------------------------------------------------------------
    // Reminder: because this example uses a temporary directory created
    // via `tempfile`, the directory and all files inside it vanish after exit.
    // ----------------------------------------------------------------------
    println!("Temporary directory will be deleted automatically.");
    println!("Example 6 complete.");

    Ok(())
}
