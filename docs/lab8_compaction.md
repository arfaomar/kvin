# Compaction Complexity

## Time Complexity of the Compaction Step

Since kvin uses an append-only log. When values are overwritten or
deleted, old entries remain in the log until compaction is performed.

During compaction:

- all live keys are iterated once
- the latest value for each key is copied into a new buffer

If:

- **U** is the number of live keys
- **S** is the total size of all live values

then the compaction step runs in **O(U + S)** time.

In practice, the runtime is linear in the size of the live data.


## When to Run the Compaction Step

Compaction is an expensive operation and should not be performed on every
insert or update.

In this implementation, compaction is triggered automatically based on the
configurable compaction ratio. This ensures that compaction only runs when a
significant amount of obsolete data has accumulated, keeping normal operations
fast while preventing unbounded memory growth.

Trade-offs:

- Lower ratio (for example 1.5):

    - compaction runs more often
    - lower memory usage
    - higher runtime overhead
- Higher ratio (for example 3 or more):

    - compaction runs less often
    - higher memory usage
    - lower compaction overhead

A ratio of **2** provides a good balance:

- at least 50 % of the stored data is obsolete when compaction runs
- memory usage is bounded to at most twice the live data size
- compaction overhead remains manageable
