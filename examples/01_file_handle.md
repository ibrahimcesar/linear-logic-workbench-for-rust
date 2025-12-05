# File Handle Protocol

This example demonstrates how linear logic models file handle lifecycle management, ensuring files are properly closed and preventing use-after-close bugs.

## The Problem

In traditional programming, file handles can be:
- Used after being closed (use-after-free)
- Forgotten without closing (resource leak)
- Closed multiple times (double-free)

Linear logic prevents all of these by tracking resource consumption.

## Linear Logic Model

```
# A file handle that must be used exactly once
FileHandle

# Reading consumes the handle, produces contents AND a closed handle
FileHandle ⊸ (Contents ⊗ ClosedHandle)

# Alternative: read without closing (returns handle for reuse)
FileHandle ⊸ (Contents ⊗ FileHandle)

# Writing also consumes and returns handle
FileHandle ⊗ Data ⊸ FileHandle

# Closing consumes handle, produces proof of closure
FileHandle ⊸ ClosedHandle
```

## Proving Safe File Operations

### Basic Read and Close
```bash
$ cargo run -q -- prove "FileHandle |- Contents * ClosedHandle"
```

This proves: "Given a FileHandle, we can produce Contents AND a ClosedHandle."

### Sequential Read then Write
```bash
$ cargo run -q -- prove "FileHandle, Data |- FileHandle"
```

Using the write operation that returns the handle.

### What CANNOT be proven (safety guarantees)

```bash
# Cannot use handle twice (no contraction without !)
$ cargo run -q -- prove "FileHandle |- Contents * Contents"
✗ NOT PROVABLE

# Cannot discard handle without closing (no weakening)
$ cargo run -q -- prove "FileHandle, OtherData |- OtherData"
✗ NOT PROVABLE

# Cannot produce handle from nothing
$ cargo run -q -- prove "|- FileHandle"
✗ NOT PROVABLE
```

## Generated Rust Code

```bash
$ cargo run -q -- codegen "FileHandle |- Contents * ClosedHandle"
```

Produces:
```rust
fn read_and_close(handle: FileHandle) -> (Contents, ClosedHandle) {
    // handle is moved (consumed) - cannot be used again
    let contents = handle.read();
    let closed = handle.close();
    (contents, closed)
}
```

The Rust ownership system enforces what linear logic proves:
- `handle` is moved into the function
- After `handle.close()`, the handle cannot be used
- The function MUST return both `Contents` and `ClosedHandle`

## With Exponentials: Reusable Readers

If we want to read multiple times without closing:

```bash
# !FileHandle means "unlimited access to file handle"
$ cargo run -q -- prove "!FileHandle |- Contents * Contents"
✓ PROVABLE
```

This models a scenario like memory-mapped files or shared read access.

## Complete Session

```bash
# Open, read twice, write, close
$ cargo run -q -- prove "!FileHandle, Data |- Contents * Contents * ClosedHandle"
```

## Key Insights

| Linear Logic Concept | File Handle Meaning |
|---------------------|---------------------|
| `A` (linear) | Handle must be used exactly once |
| `A ⊗ B` | Produces both A and B |
| `A ⊸ B` | Consumes A to produce B |
| `!A` | Shared/reusable access |
| No weakening | Cannot forget to close |
| No contraction | Cannot use closed handle |
