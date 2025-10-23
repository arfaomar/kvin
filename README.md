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
fn get(&self, key: &Key) -> Option<&Value>;
fn delete(&mut self, key: &Key) -> Option<Value>;
fn update(&mut self, key: &Key, new_value: Value) -> bool;
```

---

## 🧪 Example Usage

```rust
use kvin::{Kvin, Key, Value};

fn main() {
    let mut store = Kvin::new();

    store.put(Key::from("language"), Value::from("Rust"));
    println!("{:?}", store.get(&Key::from("language"))); // Some(Text("Rust"))

    store.update(&Key::from("language"), Value::from("Kvin v2"));
    println!("{:?}", store.get(&Key::from("language"))); // Some(Text("Kvin v2"))

    store.delete(&Key::from("language"));
    println!("{:?}", store.get(&Key::from("language"))); // None
}
```

Run it with:

```bash
cargo run --example ex1
```
