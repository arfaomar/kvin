use kvin::{Key, KvStore, Kvin, Value};

fn main() {
    let mut store = Kvin::new();

    println!(
        "→ put name: {}",
        store.put(Key::from("name"), Value::from("Kvin"))
    );
    println!("→ put int: {}", store.put(Key::from(1), Value::from(404)));
    println!(
        "→ put bool: {}",
        store.put(Key::from(true), Value::from("mem safe"))
    );

    println!(
        "→ put name again: {}",
        store.put(Key::from("name"), Value::from("Kvin v3"))
    );

    // Use get() (returns owned Value)
    println!(
        "name after failed put (get): {:?}",
        store.get(&Key::from("name"))
    );

    // Use get_owned() (same data, explicit helper)
    println!(
        "name after failed put (get_owned): {:?}",
        store.get_owned(&Key::from("name"))
    );

    // Use get_borrowed() (zero-copy borrowed access)
    println!(
        "name after failed put (get_borrowed): {:?}",
        store.get_borrowed(&Key::from("name"))
    );

    // Demonstrate mixed key types
    println!("int key (get): {:?}", store.get(&Key::from(1)));
    println!(
        "bool key (get_borrowed): {:?}",
        store.get_borrowed(&Key::from(true))
    );

    // Update and verify via all getters
    store.update(&Key::from("name"), Value::from("Kvin v2"));
    println!("updated name (get): {:?}", store.get(&Key::from("name")));
    println!(
        "updated name (get_owned): {:?}",
        store.get_owned(&Key::from("name"))
    );

    // Delete and confirm removal
    store.delete(&Key::from(1));
    println!("after delete (get): {:?}", store.get(&Key::from(1)));
    println!(
        "after delete (get_owned): {:?}",
        store.get_owned(&Key::from(1))
    );

    println!("\n Final store (debug): {:#?}", store);
}
