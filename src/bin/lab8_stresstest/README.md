# Lab 8 – Exercise 1.3 Stress Test

This directory contains the code used for the stress test in Lab 8.

## Timing and Output

The program measures elapsed time internally using `std::time::Instant`.

During execution, it tracks:

- total wall-clock time
- total number of processed rows
- average time per row (in nanoseconds)
- parsing time per row
- key–value store operation time per row
- throughput (rows per second)

All values are printed at the end of the run.

Additionally, system-level timing and memory information is collected using
platform-specific tools:

- On macOS, `/usr/bin/time -l`
- On Linux, `/usr/bin/time -v`
- On Windows, PowerShell’s `Measure-Command`

The recommended commands are:

### macOS

```zsh
/usr/bin/time -l cargo run --release --bin lab8_stresstest \
  file1-path \
  file2-path \
  > src/bin/lab8_stresstest/stresstest_output.txt 2>&1
```

### Linux

```bash
/usr/bin/time -v cargo run --release --bin lab8_stresstest \
  file1-path \
  file2-path \
  > src/bin/lab8_stresstest/stresstest_output.txt 2>&1
```

### Windows (PowerShell)

```powershell
Measure-Command {
  cargo run --release --bin lab8_stresstest `
    file1-path `
    file2-path
} | Out-File -Append src/bin/lab8_stresstest/stresstest_output.txt
```

## Autosave and Compaction Settings

The `Kvin` store supports configurable autosave and compaction behavior, which
is used during the stress test.

Both the **autosave threshold** and the **compaction ratio** can be adjusted
easily to experiment with different performance and memory trade-offs, without
changing the rest of the code.

### Autosave Threshold

- Autosave controls how often the store is written to disk automatically.
- A higher autosave threshold means fewer disk writes.
- A lower autosave threshold means data is persisted more frequently.

Trade-offs:

- Higher threshold:

  - better performance during heavy insert workloads
  - higher risk of data loss if the program crashes
- Lower threshold:

  - safer persistence behavior
  - increased I/O overhead and slower inserts

For the stress test, the autosave threshold is set to a relatively high value to
avoid frequent disk writes and focus on in-memory performance.



### Compaction Ratio

The compaction ratio determines when obsolete data in the append-only log is
removed.

- Compaction is triggered when:

  ```
  total_log_size / live_data_size >= compaction_ratio
  ```

- The compaction ratio used during the stress test is **2**.

## Processing Logic

- Input files are read line by line using `BufReader`.
- Each line is parsed into a domain code, page name, and view count.
- Keys are constructed as `"<domain>/<page>"`.

The files are processed in order:

-  The first file inserts new entries into the store.
- The second file updates existing entries by accumulating view counts or
  inserts new entries if a key was not present.

All operations go through the `KvStore` trait.
