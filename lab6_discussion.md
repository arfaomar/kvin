# Lab 6 – Error Handling

## 1. Implementation Effort

### What is the effort for implementing each of the two approaches (`anyhow` vs `thiserror`)?

- **anyhow**  
  Implementing anyhow required very little effort. We did not need to design error types, and error propagation was simple using the `?` operator. This made it fast to integrate into the existing codebase.

- **thiserror**  
  Implementing thiserror required more work because we had to design a proper error enum, think about which errors should be public, and update all functions to return typed errors. This took more time but resulted in a cleaner and more explicit error model.


## 2. Behavioral Insights

### What did each approach teach you about the behavior of your key-value store?

- Adding proper error handling made it clear how many edge cases the store actually has.  
  For example: incomplete headers, invalid UTF-8, checksum mismatches, missing keys, and truncated serialized entries.

### Did you identify failure cases that you did not foresee before adding proper error handling?

- Yes. When defining typed errors, we realized we needed to handle cases like:
  - Reading entries with corrupted headers  
  - Deserializing values with missing bytes  
  - Invalid UTF-8 in text values  
  - Iteration encountering corrupted offsets  
  - Key lookups that silently failed before but now needed structured error handling

These failure cases became much more visible once the error model was explicit.


## 3. Usability

### How is the usability of each approach for developers who use your library?

- **anyhow**  
  Usability is simple and flexible. Developers can attach context easily, but they cannot reliably match specific error kinds because the error type is erased.

- **thiserror**  
  Usability is better for library consumers because they can inspect and match specific variants. This allows writing structured control-flow depending on the reason for failure.

### How does user-code look with anyhow?

- User code typically handles errors in a generic way, e.g. printing them or adding context.  
  It cannot easily distinguish different error causes without relying on string inspection.

### How does user-code look with thiserror?

- User code can match on variants like `KvinError::KeyExists` or `KvinError::ChecksumMismatch`.  
  This leads to clearer and safer error handling, especially when the application must react differently depending on what went wrong.


## 4. Power and Control

### Are both approaches equally powerful?

- Both can represent all error cases, but they are not equally useful.  
  anyhow is more convenient, while thiserror gives finer control and stronger type guarantees.

### Which approach gives better control over:

#### How errors propagate through multiple functions?

- **thiserror**  
  Propagation is explicit because error variants are stable and known. This makes debugging and tracing easier.

#### How errors propagate out of your library into user code?

- **thiserror**  
  Users see only your curated error enum, not internal error sources. This creates a clean API boundary.

#### How nested errors (errors caused by other errors) are handled?

- **anyhow**  
  Better ergonomics for attaching context and chaining sources.  
  thiserror can also handle nested errors, but requires manual annotation with `#[source]`.

#### How users can react to specific kinds of errors?

- **thiserror**  
  The only option that reliably supports pattern matching and structured handling.


## 5. Runtime Overhead

### What is the runtime overhead of both approaches?

- **anyhow**  
  May allocate on the heap when constructing error messages or adding context.  
  Overhead is small but non-zero.

- **thiserror**  
  Errors are simple enums with static strings.  
  Usually zero heap allocations and lower overhead.