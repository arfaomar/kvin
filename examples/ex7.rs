use kvin::{BorrowedValue, Key, KvStore, Kvin, Result, Value};
use std::fs;
use tempfile::tempdir;

fn values_file_len(dir: &std::path::Path) -> u64 {
    fs::metadata(dir.join("kvin.values"))
        .map(|m| m.len())
        .unwrap_or(0)
}

fn main() -> Result<()> {
    println!("=== Example 7: Autosave + Automatic Compaction + Manual Compaction ===");

    let dir = tempdir()?;
    let path = dir.path();
    println!("Using temporary directory: {}", path.display());

    let mut store = Kvin::open(path)?;
    println!("Persistent store opened.");

    // For a demo, make persistence deterministic:
    // every write triggers autosave (and therefore also the compaction check).
    store.set_autosave_threshold(Some(1));
    store.set_compaction_ratio(Some(2));
    println!("Autosave threshold set to 1.");
    println!("Automatic compaction ratio set to 2.");

    println!("\n--- Producing log growth ---");

    store.put(Key::from("msg"), Value::from("small"))?;
    store.update(&Key::from("msg"), Value::from("this is larger"))?;
    store.update(
        &Key::from("msg"),
        Value::from("this is a very long message indeed"),
    )?;

    let buf_before = store.buf.len();
    let disk_before = values_file_len(path);
    println!("Buffer length (before): {buf_before}");
    println!("On-disk kvin.values (before): {disk_before}");

    // More overwrites => more garbage.
    store.update(&Key::from("msg"), Value::from("final"))?;

    let buf_after = store.buf.len();
    let disk_after = values_file_len(path);
    println!("Buffer length (after update): {buf_after}");
    println!("On-disk kvin.values (after update): {disk_after}");

    // Manual compaction (always possible)
    println!("\n--- Running manual compaction() ---");
    store.compact()?;
    println!("Buffer length after manual compaction: {}", store.buf.len());

    // Persist the compacted state explicitly for demonstration.
    store.save_to_dir(path)?;
    println!(
        "On-disk kvin.values after manual compaction save: {}",
        values_file_len(path)
    );

    // Verify correctness
    println!("In-memory value under 'msg' = {:?}", store.get("msg")?);

    println!("\n--- Reloading store from disk ---");
    let reloaded = Kvin::open(path)?;
    match reloaded.get("msg")? {
        Some(BorrowedValue::Text(s)) => println!("Reloaded value: {s}"),
        other => println!("Reloaded value: {:?}", other),
    }

    Ok(())
}
