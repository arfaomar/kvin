use kvin::{BorrowedValue, Key, KvStore, Kvin, Result, Value};
use tempfile::tempdir;

fn main() -> Result<()> {
    println!("=== Example 7: Autosave + Automatic Compaction + Manual Compaction ===");

    // ----------------------------------------------------------------------
    // This example uses a temporary directory created by `tempfile`.
    // It is deleted automatically at program exit.
    //
    // To persist data across runs, replace:
    //
    //     let dir = tempdir()?;
    //     let path = dir.path();
    //
    // with a real directory, e.g.:
    //
    //     let path = std::path::Path::new("data");
    //     std::fs::create_dir_all(path)?;
    //
    // and use Kvin::open(path).
    // ----------------------------------------------------------------------
    let dir = tempdir()?;
    let path = dir.path();
    println!("Using temporary directory: {}", path.display());

    // ----------------------------------------------------------------------
    // Open a persistent store.
    // Kvin::open():
    //   - loads existing files if present
    //   - enables autosave (default threshold = 10)
    //   - we modify both autosave_threshold and compaction_ratio below
    // ----------------------------------------------------------------------
    let mut store = Kvin::open(path)?;
    println!("Persistent store opened.");

    // Trigger autosave every 3 mutating operations
    store.set_autosave_threshold(Some(3));
    println!("Autosave threshold set to 3.");

    // Trigger automatic compaction whenever:
    //     buf.len() > live_entries * compaction_ratio
    //
    // Setting ratio = 2 means: compaction will run *very* aggressively.
    store.set_compaction_ratio(Some(2));
    println!("Automatic compaction ratio set to 2.");

    // ----------------------------------------------------------------------
    // Produce fragmentation by repeatedly updating the same key with
    // increasingly larger values.
    //
    // These repeated updates cause the append-only log (buf)
    // to grow and keep old versions inside the buffer.
    // Compaction eliminates these obsolete entries.
    // ----------------------------------------------------------------------
    println!("\n--- Producing log growth ---");

    store.put(Key::from("msg"), Value::from("small"))?;
    println!("1) put 'small'");

    store.update(&Key::from("msg"), Value::from("this is larger"))?;
    println!("2) update to 'this is larger'");

    store.update(
        &Key::from("msg"),
        Value::from("this is a very long message indeed"),
    )?;
    println!("3) update to a very long message -> autosave + maybe compaction");

    let buf_after_updates = store.buf.len();
    println!("Buffer length after updates: {buf_after_updates}");

    // ----------------------------------------------------------------------
    // Add another update to trigger autosave + compaction again.
    // ----------------------------------------------------------------------
    store.update(&Key::from("msg"), Value::from("final"))?;
    println!("4) update to 'final' -> autosave + maybe compaction");

    let buf_after_compaction = store.buf.len();
    println!("Buffer length after compaction: {buf_after_compaction}");

    // We expect compaction to reduce space usage.
    println!(
        "Compaction {}reduce buffer size.",
        if buf_after_compaction < buf_after_updates {
            ""
        } else {
            "did NOT "
        }
    );

    // ----------------------------------------------------------------------
    // MANUAL COMPACTION
    // Even if automatic compaction did not trigger, the user can force one.
    // ----------------------------------------------------------------------
    println!("\n--- Running manual compaction() ---");
    store.compact()?; // simply calls do_compact()

    let buf_after_manual = store.buf.len();
    println!("Buffer length after manual compaction: {buf_after_manual}");

    // ----------------------------------------------------------------------
    // Verify correctness after all operations
    // ----------------------------------------------------------------------
    let val = store.get("msg")?;
    println!("Value stored under 'msg' = {:?}", val);

    // Simulate restart
    println!("\n--- Reloading store from disk ---");
    let reloaded = Kvin::open(path)?;

    match reloaded.get("msg")? {
        Some(BorrowedValue::Text(s)) => println!("Reloaded value: {s}"),
        other => println!("Reloaded value: {:?}", other),
    }

    println!("\nTemporary directory will be deleted automatically.");
    println!("Example 7 complete.");

    Ok(())
}
