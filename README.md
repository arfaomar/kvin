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
fn put(&mut self, key: Key, value: Value);
fn get(&self, key: &Key) -> Option<BorrowedValue<'_>>;
fn delete(&mut self, key: &Key) -> Option<Value>;
fn update(&mut self, key: &Key, new_value: Value) -> bool;
fn get_owned(&self, key: &Key) -> Option<Value>;
```

---

## 🧪 Example Usage

See the [examples/ex2.rs](examples/ex2.rs) file for a simple demonstration of how to use the Kvin key–value store.

Run it with:

```bash
cargo run --example ex1
```
