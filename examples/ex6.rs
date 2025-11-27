use kvin::{BorrowedValue, Key, KvStore, Kvin, Result, Value};
use tempfile::tempdir;

fn main() -> Result<()> {
    println!("=== Example 6: Persistence with tempfile sandbox ===");

    // ----------------------------------------------------------------------
    // NOTE FOR USERS:
    //
    // This example uses a *temporary* directory created by `tempfile`.
    // The directory is deleted automatically as soon as the program exits.
    //
    // If you want the data to actually persist on your own machine across runs,
    // replace these two lines with, for example:
    //
    //     let path = std::path::Path::new("data");
    //     // Optionally create the directory manually:
    //     std::fs::create_dir_all(path)?;
    //
    // And then use `path` below.
    // ----------------------------------------------------------------------
    let dir = tempdir()?; // Creates a new empty temp directory
    let path = dir.path(); // Path to that directory

    println!("Using temporary directory: {}", path.display());

    // ----------------------------------------------------------------------
    // Create and populate the store (in-memory)
    // ----------------------------------------------------------------------
    let mut store = Kvin::new();
    store.put(Key::from("foo"), Value::from("bar"))?;
    store.put(Key::from("n"), Value::from(42_i64))?;

    // ----------------------------------------------------------------------
    // Persist the in-memory data to disk (inside the temp directory)
    //
    // Writes:
    //     <tempdir>/kvin.values
    //     <tempdir>/kvin.index
    //
    // If you use your own directory instead (e.g. `data/`), the files will
    // remain available after this program exits.
    // ----------------------------------------------------------------------
    store.save_to_dir(path)?;
    println!("Store written.");

    // ----------------------------------------------------------------------
    // Simulate a restart by loading a *fresh* Kvin from the same directory.
    // This reads the two files back into memory.
    // ----------------------------------------------------------------------
    let store2 = Kvin::load_from_dir(path)?;
    println!("Store reloaded.");

    // Read a value to show that persistence worked
    if let Some(BorrowedValue::Text(s)) = store2.get("foo")? {
        println!("foo = {s}");
    }

    // ----------------------------------------------------------------------
    // Because we used `tempfile`, the directory and the files inside it
    // will be deleted automatically when `dir` goes out of scope.
    //
    // If you switch to a real directory such as "./data", this will of course
    // NOT be deleted, and the data will persist across program runs.
    // ----------------------------------------------------------------------
    println!("Temporary directory will be deleted automatically.");
    println!("Example 6 complete.");

    Ok(())
}
