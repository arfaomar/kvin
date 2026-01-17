# Lab 7 – Persistence Design Document

## 1. What persistence strategy did we choose?
We implemented a directory-based persistence model where a store is backed by two files:

- `kvin.values` – the serialized value log  
- `kvin.index` – the mapping of keys to offsets  

Kvin supports two persistence modes:


### 1) Manual Persistence

A store created with `Kvin::new()` is in-memory only until the user explicitly calls:

```rust
store.save_to_dir("path")?;
````

Likewise, an in-memory store can be restored later using:

```rust
let store = Kvin::load_from_dir("path")?;
```

This mode is useful when the caller wants full control over when and how persistence happens.



### 2) Automatic Persistence (Autosave)

A store opened through:

```rust
let mut store = Kvin::open("path")?;
```

automatically enables persistence.
`open()` performs:

- Loading existing data from disk (if available)
- Activating autosave with a default threshold of 10 write operations
- Storing the persistence directory path inside the store

Once autosave is active:

- Every `put`, `update`, or `delete` increments an internal counter (`dirty_ops`)
- When `dirty_ops` reaches the threshold, the store automatically writes itself to disk

This provides a middle ground between performance and durability: data is not flushed after every operation, but regular persistence still happens without developer intervention.


## 2. How do we make autosave configurable?

The autosave threshold can be changed or disabled entirely:

```rust
store.set_autosave_threshold(Some(5)); // autosave every 5 writes
store.set_autosave_threshold(None);    // disable autosave
```

This tunability allows users to balance performance and durability depending on their needs.


## 3. How does saving to disk work?

Every save (manual or autosave) uses a crash-safe write protocol:

1. Write new data to temporary files:

    - `kvin.values.tmp`
    - `kvin.index.tmp`
2. Flush the data to disk with `sync_all`
3. Use atomic rename to replace the original files

This ensures that a save either completes fully or not at all.
There is never a half-written or corrupted file left behind.

### Platform considerations

- On Unix-like systems, `rename` is atomic and reliable.
- On Windows, rename behavior is not fully POSIX-compliant but is still best-effort and works well for our use case.


## 4. How does loading work (including crash recovery)?

`Kvin::load_from_dir` uses a safe and predictable algorithm:

1. If both main files exist, load them normally.
2. If main files are missing but `.tmp` files exist, assume a crash occurred:

    - Try loading the `.tmp` versions
    - If successful, promote them to the main files
3. If neither exists, return an empty store.

This design guarantees:

- No corrupted state is ever silently accepted
- Recovery is automatic, without requiring user intervention
- Partially written files from crashes are safely handled


## 5. How does error handling work?

### Autosave errors

Autosave failures return:

```
KvinError::AutosaveFailed(...)
```

This prevents silent data-loss scenarios:
operations like `put`, `update`, and `delete` fail if the autosave mechanism cannot persist the new state.

The in-memory changes still occur, but callers are explicitly informed that durability was not achieved.

### Manual save errors

Manual saves return:

```
KvinError::Io(...)
```

This typically indicates:

- Missing directory permissions
- Disk I/O issues
- Invalid or non-existent paths

### Errors during `Drop`

Rust destructors cannot return errors, so:

- A final autosave is attempted when the store is dropped
- Any failure is logged but not propagated
- This prevents unexpected panics during program shutdown

This best-effort approach ensures that Drop never violates Rust’s guarantees while still keeping persistence attempts transparent.