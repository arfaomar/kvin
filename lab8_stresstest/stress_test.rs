//! Usage:
//!   cargo run --release --bin lab8_stresstest <file1> <file2>
//!
//! Notes:
//! - Input files must be plain-text pagecounts-ez files
//! - Files must be decompressed beforehand


use kvin::{BorrowedValue, Key, KvStore, Kvin, Result, Value};

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

/// Process one pagecounts-ez file.
/// Returns:
///   (rows_processed, kv_operations, parse_time_ns, kv_time_ns)
fn process_file(store: &mut Kvin, path: &str, first_file: bool) -> Result<(u64, u64, u128, u128)> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut line = String::new();

    let mut rows: u64 = 0;
    let mut kv_ops: u64 = 0;
    let mut parse_time: u128 = 0;
    let mut kv_time: u128 = 0;

    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }

        rows += 1;

        // ---------------- Parse timing ----------------
        let t_parse = Instant::now();

        let mut tokens = line.split_ascii_whitespace();
        let domain_code = match tokens.next() {
            Some(v) => v,
            None => continue,
        };
        let page_name = match tokens.next() {
            Some(v) => v,
            None => continue,
        };
        let view_count_str = match tokens.next() {
            Some(v) => v,
            None => continue,
        };

        let view_count: i64 = match view_count_str.parse() {
            Ok(v) => v,
            Err(_) => continue,
        };

        let key = format!("{domain_code}/{page_name}");
        let key = Key::from(key.as_str());

        parse_time += t_parse.elapsed().as_nanos();

        // ---------------- KV timing ----------------
        let t_kv = Instant::now();

        if first_file {
            if store.put(key, Value::from(view_count)).is_ok() {
                kv_ops += 1;
            }
        } else {
            match store.get(&key)? {
                Some(BorrowedValue::Int(prev)) => {
                    store.update(&key, Value::from(prev + view_count))?;
                    kv_ops += 1;
                }
                _ => {
                    store.put(key, Value::from(view_count))?;
                    kv_ops += 1;
                }
            }
        }

        kv_time += t_kv.elapsed().as_nanos();
    }

    Ok((rows, kv_ops, parse_time, kv_time))
}

fn main() -> Result<()> {
    println!("=== Lab 8 – Exercise 1.3: Wikipedia Pageviews Stress Test ===");

    // ------------------------------------------------------------
    // CLI arguments
    // ------------------------------------------------------------
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage:");
        eprintln!("  cargo run --release --bin lab8_stresstest <file1> <file2>");
        return Ok(());
    }

    let file1 = &args[1];
    let file2 = &args[2];

    println!("Input files:");
    println!("  1) {file1}");
    println!("  2) {file2}");

    // ------------------------------------------------------------
    // Create persistent Kvin store
    // ------------------------------------------------------------
    let dir = tempfile::tempdir()?;
    let path = dir.path();

    let mut store = Kvin::open(path)?;
    store.set_autosave_threshold(Some(50_000)); // reduce I/O noise
    store.set_compaction_ratio(Some(8));

    println!("Kvin store directory: {}", path.display());

    // ------------------------------------------------------------
    // Process first file
    // ------------------------------------------------------------
    println!("\n--- Processing file 1 (insert phase) ---");
    let t1_start = Instant::now();
    let (rows1, ops1, parse1, kv1) = process_file(&mut store, file1, true)?;
    let t1 = t1_start.elapsed();

    // ------------------------------------------------------------
    // Process second file
    // ------------------------------------------------------------
    println!("\n--- Processing file 2 (accumulation phase) ---");
    let t2_start = Instant::now();
    let (rows2, ops2, parse2, kv2) = process_file(&mut store, file2, false)?;
    let t2 = t2_start.elapsed();

    // ------------------------------------------------------------
    // Aggregate statistics
    // ------------------------------------------------------------
    let total_rows = rows1 + rows2;
    let total_ops = ops1 + ops2;
    let total_time = t1 + t2;
    let total_parse = parse1 + parse2;
    let total_kv = kv1 + kv2;

    println!("\n=== Results ===");
    println!("Total rows processed:     {total_rows}");
    println!("Total KV operations:     {total_ops}");
    println!("Unique keys stored:      {}", store.index.len());
    println!("Final buffer size:       {} bytes", store.buf.len());
    println!("Total wall-clock time:   {:.2?}", total_time);

    let avg_ns_per_row = total_time.as_nanos() as f64 / total_rows as f64;
    let avg_parse_ns = total_parse as f64 / total_rows as f64;
    let avg_kv_ns = total_kv as f64 / total_rows as f64;

    println!("\n--- Per-row averages ---");
    println!("Avg total time / row:    {:.1} ns", avg_ns_per_row);
    println!("Avg parse time / row:    {:.1} ns", avg_parse_ns);
    println!("Avg KV time / row:       {:.1} ns", avg_kv_ns);

    let throughput = total_rows as f64 / total_time.as_secs_f64();
    println!("Throughput:              {:.2} rows/sec", throughput);


    println!("\nStress test complete.");
    println!("Temporary directory cleaned up automatically.");

    Ok(())
}
