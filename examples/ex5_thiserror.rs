use kvin::{Kvin, KvStore};

fn main() {
    let mut store = Kvin::new();
    store.put("hello".into(), "world".into());

    corrupt_store_internally(&mut store);


    println!("Initial store contents inserted.\n");

    match store.get("hello") {
        Some(val) => println!("Entry read (doesnt work if corrupted): {:?}", val),
        None => println!("get() None"),
    }

    println!("--- Iterating over entries with `iter()` ---");
    for entry in &store {
        match entry {
            Ok(e) => println!("Good Entry: {:?} => {:?}", e.key, e.value),
            Err(err) => println!("Corrupt Enty: {}", err),
        }
    }
}

fn corrupt_store_internally(store: &mut Kvin) {

    let buf = store.buf.as_mut_slice();

    // Flip some bits in the last value's payload to simulate corruption
    if buf.len() > kvin::RAW_HEADER_SIZE + 8 {
        let payload_start = buf.len() - 10;
        if payload_start + 5 < buf.len() {
            buf[payload_start + 5] ^= 0xFF;
        }
    }
}