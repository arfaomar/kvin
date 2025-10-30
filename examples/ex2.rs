use kvin::{Key, KvStore, Kvin, Value};

fn main() {
    let mut store = Kvin::new();

    println!("→ put name: {}", store.put(Key::from("name"), Value::from("Kvin")));
    println!("→ put int: {}", store.put(Key::from(1), Value::from(404)));
    println!("→ put bool: {}", store.put(Key::from(true), Value::from("mem safe")));

    println!("→ put name again: {}", store.put(Key::from("name"), Value::from("Kvin v3")));
    println!(
        "name after failed put (owned): {:?}",
        store.get_owned(&Key::from("name"))
    );
    println!(
        "name after failed put (borrowed): {:?}",
        store.get_borrowed(&Key::from("name"))
    );

    println!("int key (owned) -> {:?}", store.get_owned(&Key::from(1)));
    println!("bool key (borrowed) -> {:?}", store.get_borrowed(&Key::from(true)));

    store.update(&Key::from("name"), Value::from("Kvin v2"));
    println!("updated name (owned) -> {:?}", store.get_owned(&Key::from("name")));

    store.delete(&Key::from(1));
    println!("after delete (owned) -> {:?}", store.get_owned(&Key::from(1)));

    println!("\n🔍 Final store (debug): {:#?}", store);
}
