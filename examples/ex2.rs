use kvin::{Key, KvStore, Kvin, Value};

fn main() {
    let mut store = Kvin::new();

    // Insert values
    store.put(Key::from("language"), Value::from("Rust"));
    store.put(Key::from(1), Value::from(true));

    // Attempt to overwrite existing key should fail
    assert!(!store.put(Key::from("language"), Value::from("Go")));

    // get() now returns BorrowedValue
    println!("{:?}", store.get(&Key::from("language"))); // Some(BorrowedValue::Text("Rust"))
    println!("{:?}", store.get(&Key::from(1))); // Some(BorrowedValue::Bool(true))

    // get_owned() returns Value
    println!("{:?}", store.get_owned(&Key::from("language"))); // Some(Value::Text("Rust"))
    println!("{:?}", store.get_owned(&Key::from(1))); // Some(Value::Bool(true))

    // get_borrowed() is same as get()
    println!("{:?}", store.get_borrowed(&Key::from("language"))); // Some(BorrowedValue::Text("Rust"))

    // Update existing key
    store.update(&Key::from("language"), Value::from("C++"));
    println!("{:?}", store.get(&Key::from("language"))); // Some(BorrowedValue::Text("C++"))

    // Delete a key
    let deleted = store.delete(&Key::from(1));
    println!("{:?}", deleted); // Some(Value::Bool(true))

    // After delete
    println!("{:?}", store.get(&Key::from(1))); // None
}
