use kvin::{Kvin, Key, Value, KvStore};

fn main() {
    let mut store = Kvin::new();

    store.put(Key::from("name"), Value::from("Kvin"));
    store.put(Key::from(1), Value::from(404));
    store.put(Key::from(true), Value::from("mem safe"));

    println!("name -> {:?}", store.get(&Key::from("name")));
    println!("int key -> {:?}", store.get(&Key::from(1)));

    store.update(&Key::from("name"), Value::from("Kvin v2"));
    println!("updated name -> {:?}", store.get(&Key::from("name")));

    store.delete(&Key::from(1));
    println!("after delete -> {:?}", store.get(&Key::from(1)));

    println!("\nFinal store: {:#?}", store);
}
