# 🦀 kvin — where memory safety meets questionable decisions

This repository contains the **Rust key–value store implementation** for the  
*Advanced Systems Programming* lab (WS25/26).

It provides:
- A basic `lib` + `examples` project setup
- A CI pipeline that checks for compile errors, runs all tests, and executes all examples

---

## 🧩 Initialization

```rust
let mut store = Kvin::new();
````

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

---

## 🧪 Example Usage

See the [examples](examples) file for a simple demonstration of how to use the Kvin key–value store.

Run it with:

```bash
cargo run --example ex<number>
```
