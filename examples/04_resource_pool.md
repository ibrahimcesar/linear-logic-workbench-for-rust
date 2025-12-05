# Resource Pools and Connection Management

Linear logic models resource pools where resources must be acquired, used, and returned. This prevents resource leaks, double-frees, and ensures pool invariants.

## The Problem

Connection pools (database, HTTP, thread pools) have subtle bugs:

- Forgetting to return a connection (leak)
- Returning a connection twice (corruption)
- Using a connection after returning it
- Exceeding pool capacity

## Linear Logic Model

### Basic Pool Operations

```
# Pool has available connections
Pool ⊗ Available ⊗ Available ⊗ Available    # Pool with 3 connections

# Acquire: take connection from pool
Pool ⊗ Available ⊸ Pool ⊗ InUse

# Release: return connection to pool
Pool ⊗ InUse ⊸ Pool ⊗ Available

# Use: do work with connection (keeps it borrowed)
InUse ⊗ Query ⊸ InUse ⊗ Result
```

## Proving Pool Safety

### Acquire and Release
```bash
# Acquire one, use it, release it
$ cargo run -q -- prove "Pool, Available, Query |- Pool * Available * Result"
✓ PROVABLE
```

The connection is returned to the same state it was acquired.

### Cannot Exceed Capacity
```bash
# Try to acquire 4 from a pool of 3
$ cargo run -q -- prove "Pool, Available, Available, Available |- Pool * InUse * InUse * InUse * InUse"
✗ NOT PROVABLE (only 3 Available tokens)
```

### Cannot Leak Connections
```bash
# Try to end with fewer Available than started
$ cargo run -q -- prove "Pool, Available, Available |- Pool * Available"
✗ NOT PROVABLE (one Available is missing!)
```

### Cannot Double-Release
```bash
# Try to return same connection twice
$ cargo run -q -- prove "Pool, InUse |- Pool * Available * Available"
✗ NOT PROVABLE (one InUse can only become one Available)
```

## With Exponentials: Shared Read Connections

```
# Read-only connection can be shared (copied)
!ReadOnlyConn

# Write connection is linear (exclusive)
WriteConn
```

```bash
# Can do multiple reads with shared connection
$ cargo run -q -- prove "!ReadOnlyConn |- Result * Result * Result"
✓ PROVABLE

# Write connection: exactly one write
$ cargo run -q -- prove "WriteConn |- Result * Result"
✗ NOT PROVABLE
```

## Scoped Resource Pattern

Ensure resources are always released, even on error:

```
# Scope: guarantees cleanup
Scope ⊗ Available ⊸ (Result ⊕ Error) ⊗ Available

# The Available token MUST be produced regardless of success/error
```

```bash
# Both success and error paths return the connection
$ cargo run -q -- prove "Scope, Available |- (Result + Error) * Available"
✓ PROVABLE
```

## Thread Pool Model

```
# Thread tokens
IdleThread
BusyThread

# Spawn task: takes idle thread, returns busy
IdleThread ⊗ Task ⊸ BusyThread

# Task completion: busy becomes idle
BusyThread ⊸ IdleThread ⊗ TaskResult
```

```bash
# Run task and thread returns to pool
$ cargo run -q -- prove "IdleThread, Task |- IdleThread * TaskResult"
✓ PROVABLE

# Cannot run two tasks on one thread simultaneously
$ cargo run -q -- prove "IdleThread, Task, Task |- IdleThread * TaskResult * TaskResult"
✗ NOT PROVABLE (need 2 threads for 2 concurrent tasks)
```

## Database Transaction Pool

```
# Connection from pool
Connection

# Begin transaction: connection → transaction handle
Connection ⊸ Transaction

# Commit: transaction → connection (returned to pool)
Transaction ⊸ Connection ⊗ CommitProof

# Rollback: transaction → connection (returned to pool)
Transaction ⊸ Connection ⊗ RollbackProof

# Connection must go back to pool (one of commit or rollback)
```

```bash
# Transaction with commit returns connection
$ cargo run -q -- prove "Connection |- Connection * CommitProof"
✓ PROVABLE

# Transaction with rollback returns connection
$ cargo run -q -- prove "Connection |- Connection * RollbackProof"
✓ PROVABLE

# Cannot just abandon transaction (connection would leak)
$ cargo run -q -- prove "Connection |- CommitProof"
✗ NOT PROVABLE (Connection must be returned)
```

## Generated Rust: RAII Pattern

```bash
$ cargo run -q -- codegen "Pool, Available |- Pool * InUse"
```

```rust
struct Pool;
struct Available;
struct InUse;

// Acquire takes ownership of Available, returns InUse
fn acquire(pool: Pool, available: Available) -> (Pool, InUse) {
    // available is consumed
    (pool, InUse)
}

fn release(pool: Pool, in_use: InUse) -> (Pool, Available) {
    // in_use is consumed, Available is returned
    (pool, Available)
}

// Rust's ownership ensures:
// - Can't use Available after acquiring
// - Can't use InUse after releasing
// - Compiler tracks resource state
```

## Semaphore Pattern

```
# Semaphore with N permits
Permit ⊗ Permit ⊗ Permit    # 3 permits

# Acquire permit
Permit ⊸ HeldPermit

# Release permit
HeldPermit ⊸ Permit
```

```bash
# Acquire 2 permits, use them, release both
$ cargo run -q -- prove "Permit, Permit, Permit |- Permit * HeldPermit * HeldPermit"
✓ PROVABLE (1 in pool, 2 held)

# Must release what we acquire
$ cargo run -q -- prove "Permit, Permit |- Permit * Permit"
✓ PROVABLE (trivial: no operations)
```

## Key Insights

| Linear Logic | Pool Semantics |
|--------------|----------------|
| Linear token | Must be used exactly once |
| `Available` | Connection ready for use |
| `InUse` | Connection currently borrowed |
| `A ⊸ B` | State transition |
| `A ⊗ B ⊸ C ⊗ D` | Acquire/release pattern |
| No weakening | Cannot leak resources |
| No contraction | Cannot duplicate resources |
| `!A` | Shared/read-only access |

## Real-World Mapping

```
Linear Logic          →  Rust
──────────────────────────────────────
Available             →  Pool::get() returns Handle
InUse                 →  Handle (owns connection)
InUse ⊸ Available     →  Drop impl returns to pool
!ReadConn             →  Arc<Connection>
```
