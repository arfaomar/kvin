# Iterator Design (Lab 5)

## Goal

In this lab, we added iterator support to our `Kvin` key-value store.
The goal was to make it possible to loop over all entries, keys, or values easily without copying data.

## How it works

Our store keeps all serialized values in a single `Vec<u8>` buffer and an index that maps each key to a byte offset in that buffer.

The iterator (`StoreIter<'a>`) holds:

* a reference to that byte buffer (`&'a [u8]`)
* an iterator over the internal `HashMap` (`Iter<'a, Key, usize>`)

When calling `next()`, it takes the next `(Key, offset)` pair and deserializes the value without cloning it.
This returns a `BorrowedEntry` that contains references into the store (`&Key` and `BorrowedValue<'a>`).

We also added helper methods:

* `iter()` → returns all entries
* `keys()` → returns only keys
* `values()` → returns only values

All three work without any extra allocation and use safe lifetime-based borrowing.

## Lifetime and safety

The lifetime `'a` ensures that the data borrowed by the iterator cannot outlive the store itself.
If the store is dropped, the compiler will prevent further iterator use.
This gives us zero-copy access with full safety guaranteed by Rust.

## Why we don’t allow mutation

We cannot modify the store while iterating because the iterator borrows the store immutably.
Changing it would break the validity of the borrowed data and is therefore not allowed.

## Extra improvements

We added some small niceties to improve usability:

```rust
impl<'a> IntoIterator for &'a Kvin {
    type Item = BorrowedEntry<'a>;
    type IntoIter = StoreIter<'a>;
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl<'a> std::iter::FusedIterator for StoreIter<'a> {}
impl<'a> ExactSizeIterator for StoreIter<'a> {
    fn len(&self) -> usize { self.inner.len() }
}
```

These additions allow syntax like `for entry in &store` and provide useful iterator traits such as `len()`.

## Result

Our iterator design is zero-copy, safe, and preserves DX.
It works well with Rust’s iterator and makes looking over the contents of the store clean and efficient.
