# Safe vs. Unsafe Serialization

In this lab we added an unsafe version of our serialization and deserialization functions for the `RawHeader` structure.
Both versions do the same thing, but they work in very different ways.

### The safe version

The safe version uses normal Rust methods to copy fields into a buffer one by one.
It is easy to read, hard to break, and checked by the compiler.
Even if we make small mistakes, the compiler will usually warn us.

The downside is that the safe version is a bit slower, because every field has to be copied and converted separately, and there are extra bounds checks which decrease speed as well.

### The unsafe version

The unsafe version uses raw pointers to write and read the `RawHeader` directly into memory. Writing this code takes more care and requires extra safety checks by the programmer (since the compiler cannot help us here).
We use `repr(C, packed)` to make sure the struct has a predictable layout, and then use `as_ptr()` and `ptr::copy_nonoverlapping` to move bytes quickly.

This version is faster and shows how low-level memory access works in Rust, but it also bypasses the safety checks of the compiler but is also more tedious to write and requires attention to detail.
If the buffer is too small, misaligned, or we make a pointer mistake, the program can crash or read wrong data.

### What we learned

* `unsafe` does not exactly mean bad but it means the programmer must make extra sure that the code is correct.
* Safe Rust is good for most cases, because it protects us from common memory errors.
* Unsafe Rust is useful when we need speed or must work directly with memory at a low level.

### Conclusion

Our unsafe parser works faster, but the safe one is easier to understand and less risky.
For normal use we would prefer to keep the safe version, but it was interesting to see how Rust allows low-level memory access while still keeping the unsafe part clearly marked and kind of isolated.