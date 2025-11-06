use kvin::{Key, KvStore, Kvin, Value};

fn main() {
    let mut store = Kvin::new();

    // Insert values
    store.put(Key::from("language"), Value::from("Rust"));
    store.put(Key::from(1), Value::from(true));

    // Attempt to overwrite existing key should fail
    assert!(!store.put(Key::from("language"), Value::from("Go")));

    // get() now returns BorrowedValue
    println!("{:?}", store.get("language")); // Some(BorrowedValue::Text("Rust"))
    println!("{:?}", store.get(1)); // Some(BorrowedValue::Bool(true))

    // get_owned() returns Value
    println!("{:?}", store.get_owned("language")); // Some(Value::Text("Rust"))
    println!("{:?}", store.get_owned(1)); // Some(Value::Bool(true))

    // get_borrowed() is same as get()
    println!("{:?}", store.get_borrowed(&Key::from("language"))); // Some(BorrowedValue::Text("Rust"))

    // Update existing key
    store.update(&Key::from("language"), Value::from("C++"));
    println!("{:?}", store.get("language")); // Some(BorrowedValue::Text("C++"))

    // Delete a key
    let deleted = store.delete(1);
    println!("{:?}", deleted); // Some(Value::Bool(true))

    // After delete
    println!("{:?}", store.get(1)); // None
}
