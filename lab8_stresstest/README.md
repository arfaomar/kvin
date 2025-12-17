# Lab 8 – Exercise 1.3 Stress Test

This directory contains the code used for the stress test in Exercise 1.3.
The implementation processes large Wikipedia pagecounts-ez files and records
timing information during execution.


## Timing and Output

The program measures elapsed time internally using `std::time::Instant`.

During execution, it tracks:
- total wall-clock time
- total number of processed rows
- average time per row (in nanoseconds)
- parsing time per row
- key–value store operation time per row
- throughput (rows per second)

All of these values are printed at the end of the run.

**On macOS**, additional timing and memory information is collected using the system
tool `/usr/bin/time -l`.

The recommended commands are:

### macOS
```zsh
/usr/bin/time -l cargo run --release --bin lab8_stresstest \
  file1-path \
  file2-path \
  > lab8_stresstest/stresstest_output.txt 2>&1
````

### Linux

```bash
/usr/bin/time -v cargo run --release --bin lab8_stresstest \
  file1-path \
  file2-path \
  > lab8_stresstest/stresstest_output.txt 2>&1
```

### Windows (PowerShell)

```powershell
Measure-Command {
  cargo run --release --bin lab8_stresstest `
    file1-path `
    file2-path
} | Out-File -Append lab8_stresstest\stresstest_output.txt
```

These commands capture both the program output and the system-level timing
information in a single file.

 

## Autosave and Compaction Settings

The `Kvin` store supports configurable autosave and compaction behavior, which is
used during the stress test.

- Autosave is enabled, but the autosave threshold is set to a relatively high
  value to avoid frequent disk writes during the test.
- Compaction is also enabled using an adjustable compaction ratio.
- Compaction only runs when the log buffer grows large compared to the number of
  live keys.

Both values can be changed easily to experiment with different settings without
modifying the rest of the code.


## Processing Logic

* Input files are read line by line using `BufReader`.
* Each line is parsed into a domain code, page name, and view count.
* Keys are constructed as `"<domain>/<page>"`.

The files are processed in order:

* The first file inserts new entries into the store.
* The second file updates existing entries by accumulating view counts or inserts
  new ones if a key was not present.

All operations go through the `KvStore` trait.
