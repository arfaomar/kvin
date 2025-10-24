use kvin::{Key, KvStore, Kvin, Value};

fn main() {
    let mut store = Kvin::new();

    println!(
        "put name: {}",
        store.put(Key::from("name"), Value::from("Kvin"))
    );
    println!("put int: {}", store.put(Key::from(1), Value::from(404)));
    println!(
        "put bool: {}",
        store.put(Key::from(true), Value::from("mem safe"))
    );

    println!(
        "put name again: {}",
        store.put(Key::from("name"), Value::from("Kvin v3"))
    );
    println!("name after failed put: {:?}", store.get(&Key::from("name")));

    println!("int key -> {:?}", store.get(&Key::from(1)));

    store.update(&Key::from("name"), Value::from("Kvin v2"));
    println!("updated name -> {:?}", store.get(&Key::from("name")));

    store.delete(&Key::from(1));
    println!("after delete -> {:?}", store.get(&Key::from(1)));

    println!("\nFinal store: {:#?}", store);
}
