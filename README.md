# 🦀 kvin — where memory safety meets questionable decisions

This repository contains the **Rust key–value store implementation** for the
*Advanced Systems Programming* lab (WS25/26).

It provides:

* A safe, byte-buffer–backed key–value store
* Zero-copy reads (`BorrowedValue`)
* Append-only serialization format with CRC32 validation
* Persistence to disk (`kvin.values` + `kvin.index`)
* Optional **automatic persistence (autosave)** with crash-recovery
* A CI pipeline that builds all examples and runs all tests

---

## Initialization

### In-memory store (no persistence):

```rust
let mut store = Kvin::new();
```

### Persistent store with autosave:

```rust
let mut store = Kvin::open("data")?;
```

This:

* Loads existing data from `"data/kvin.values"` and `"data/kvin.index"`
* Enables autosave by default (every 10 mutating operations)
* Persists updates automatically and again when the store is dropped

---

## ⚙️ CRUD API

```rust
fn put(&mut self, key: Key, value: Value) -> Result<()>;
fn update(&mut self, key: &Key, new_value: Value) -> Result<()>;

fn get<K>(&self, key: K) -> Result<Option<BorrowedValue<'_>>>
where
    K: Into<Key>;

fn delete<K>(&mut self, key: K) -> Result<Option<Value>>
where
    K: Into<Key>;

fn get_owned<K>(&self, key: K) -> Result<Option<Value>>
where
    K: Into<Key>;
```

The `get` method returns **zero-copy borrowed values** directly referencing the internal buffer, while `get_owned` clones the data when needed.

---

## Persistence

Kvin persists its internal state using two files:

```
kvin.values   // raw append-only value log
kvin.index    // key → offset mapping
```

You may save manually:

```rust
store.save_to_dir("data")?;
```

You may load manually:

```rust
let store = Kvin::load_from_dir("data")?;
```

Both operations provide full CRC32 validation of all entries.

---

## Autosave (Automatic Persistence)

Opening a store via:

```rust
let mut store = Kvin::open("data")?;
```

enables automatic persistence:

* Every *N* mutating operations (default: `N = 10`)
* On `Drop` (best-effort)
* Using atomic temp-file writes (`*.tmp`) for crash safety

You can change or disable the autosave threshold:

```rust
store.set_autosave_threshold(Some(5));  // autosave every 5 ops
store.set_autosave_threshold(None);     // disable autosave
```

Autosave failures surface via:

```
KvinError::AutosaveFailed(...)
```

so that callers can react to durability issues explicitly.

---

## Example Usage

The `examples/` folder contains runnable code for demonstrating different features of the key–value store.

Run an example with:

```bash
cargo run --example ex<number>
```

For instance:

```bash
cargo run --example ex6
```

---

## Testing

Run the full test suite:

```bash
cargo test
```

---

## Kvin File Format

```plantuml
@startuml
title Kvin On-Disk File Format (values + index)

skinparam rectangleStyle rounded
skinparam shadowing false
skinparam defaultTextAlignment center

rectangle "kvin.values" as VALUES {
    rectangle "Entry 0" as E0 {
        rectangle "Header\n(total_len: u64\nchecksum: u32\ntag: u8)" as H0
        rectangle "Payload\n(variable length\nencoded per type)" as P0
    }
    rectangle "Entry 1" as E1 {
        rectangle "Header" as H1
        rectangle "Payload" as P1
    }
    rectangle "..." as EM
}

rectangle "kvin.index" as INDEX {
    rectangle "count: u64\n(# of entries)" as CNT
    rectangle "For each entry:" as FE {
        rectangle "Serialized Key\n(tag + payload)" as KEY
        rectangle "Offset: u64\n(start position in values file)" as OFF
    }
}

note right of VALUES
Each entry in `kvin.values`
is self-contained:
• Header includes:
  – total payload length
  – CRC32 checksum
  – type tag
• Payload encoding depends
  on Value variant:
  Int, Bool, Text, Blob
end note

note right of INDEX
`kvin.index` maps
   Key → offset
into the values file.
Keys use the same
serialization format
as Values but without
entry headers.
end note

@enduml
``