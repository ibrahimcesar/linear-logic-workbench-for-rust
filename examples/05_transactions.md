# Database Transactions and ACID Properties

Linear logic naturally models database transactions where resources (data) must be handled atomically, consistently, and durably.

## The Problem

Database transactions have critical invariants:

- Atomicity: All operations succeed or all fail
- Consistency: Data integrity is maintained
- Isolation: Concurrent transactions don't interfere
- Durability: Committed changes persist

Traditional code can violate these through:
- Forgetting to commit or rollback
- Using data after rollback
- Partial commits
- Lost updates

## Linear Logic Model

### Basic Transaction Lifecycle

```
# Begin transaction: connection becomes transaction handle
Connection ⊸ Transaction

# Operations consume and produce transaction handle
Transaction ⊗ ReadQuery ⊸ Transaction ⊗ Data
Transaction ⊗ WriteQuery ⊗ Data ⊸ Transaction

# Must end with exactly one of:
Transaction ⊸ Connection ⊗ CommitProof    # Commit
Transaction ⊸ Connection ⊗ RollbackProof  # Rollback
```

## Proving Transaction Safety

### Valid Transaction Flow
```bash
# Start, do work, commit
$ cargo run -q -- prove "Connection, ReadQuery, WriteQuery, Data |- Connection * CommitProof * Data"
✓ PROVABLE
```

### Must Complete Transaction
```bash
# Cannot abandon a transaction (connection would leak)
$ cargo run -q -- prove "Connection |- CommitProof"
✗ NOT PROVABLE (Connection must be returned)

# Cannot get both commit AND rollback
$ cargo run -q -- prove "Connection |- Connection * CommitProof * RollbackProof"
✗ NOT PROVABLE (one transaction = one outcome)
```

### Atomicity: All or Nothing
```bash
# Either all operations complete with commit...
$ cargo run -q -- prove "Transaction, Op1, Op2, Op3 |- Connection * CommitProof"
✓ PROVABLE

# ...or rollback (no partial state)
$ cargo run -q -- prove "Transaction |- Connection * RollbackProof"
✓ PROVABLE
```

## Two-Phase Commit Protocol

For distributed transactions across multiple databases:

```
# Phase 1: Prepare
Coordinator ⊗ Participant ⊸ Coordinator ⊗ Prepared

# Phase 2a: Commit (all prepared)
Coordinator ⊗ Prepared ⊗ Prepared ⊸ Coordinator ⊗ Committed ⊗ Committed

# Phase 2b: Abort (any failed)
Coordinator ⊗ Prepared ⊸ Coordinator ⊗ Aborted
```

```bash
# Two-phase commit with 2 participants
$ cargo run -q -- prove "Coordinator, Participant, Participant |- Coordinator * Committed * Committed"
✓ PROVABLE

# Cannot commit without all participants prepared
$ cargo run -q -- prove "Coordinator, Participant |- Coordinator * Committed * Committed"
✗ NOT PROVABLE (need 2 participants for 2 commits)
```

## Saga Pattern for Long Transactions

Sagas use compensating transactions instead of rollback:

```
# Forward action with compensation capability
Action ⊸ Done ⊗ Compensate

# Execute compensation to undo
Compensate ⊸ Undone

# Saga: sequence of actions
Saga ⊗ Action1 ⊗ Action2 ⊗ Action3 ⊸
    (Done1 ⊗ Done2 ⊗ Done3)   # Success
  ⊕ (Undone1 ⊗ Undone2)       # Partial failure, compensated
```

```bash
# Saga either fully completes or compensates
$ cargo run -q -- prove "Saga, Action1, Action2 |- (Done1 * Done2) + (Undone1 * Undone2)"
✓ PROVABLE
```

## Optimistic Locking

```
# Read with version
Data ⊗ Version ⊸ ReadData ⊗ Version

# Write only if version matches
WriteData ⊗ Version ⊗ ExpectedVersion ⊸
    (Data ⊗ NewVersion)      # Success
  ⊕ VersionConflict          # Conflict
```

```bash
# Optimistic update: success or conflict
$ cargo run -q -- prove "WriteData, Version, ExpectedVersion |- (Data * NewVersion) + VersionConflict"
✓ PROVABLE
```

## Pessimistic Locking

```
# Acquire exclusive lock
Resource ⊸ LockedResource ⊗ LockHandle

# Release lock
LockedResource ⊗ LockHandle ⊸ Resource

# Cannot use resource without lock
Resource ⊸ Data  # NOT directly accessible
```

```bash
# Must acquire lock, use, and release
$ cargo run -q -- prove "Resource |- Resource"
✓ PROVABLE (identity: resource returned)

# Lock ensures exclusive access
$ cargo run -q -- prove "Resource |- LockedResource * LockedResource"
✗ NOT PROVABLE (one resource = one lock)
```

## Savepoints

```
# Create savepoint within transaction
Transaction ⊸ Transaction ⊗ Savepoint

# Rollback to savepoint (keeps transaction open)
Transaction ⊗ Savepoint ⊸ Transaction

# Release savepoint
Transaction ⊗ Savepoint ⊸ Transaction
```

```bash
# Create savepoint, do work, rollback to savepoint, continue
$ cargo run -q -- prove "Transaction |- Transaction"
✓ PROVABLE

# Savepoint must be consumed (released or rolled back to)
$ cargo run -q -- prove "Transaction |- Connection * Savepoint"
✗ NOT PROVABLE (savepoint must be handled)
```

## Generated Rust Code

```bash
$ cargo run -q -- codegen "Connection |- Connection * CommitProof"
```

```rust
struct Connection;
struct Transaction;
struct CommitProof;
struct RollbackProof;

impl Connection {
    fn begin(self) -> Transaction {
        Transaction
    }
}

impl Transaction {
    fn commit(self) -> (Connection, CommitProof) {
        (Connection, CommitProof)
    }

    fn rollback(self) -> (Connection, RollbackProof) {
        (Connection, RollbackProof)
    }
}

// Usage: compiler enforces transaction completion
fn safe_transaction(conn: Connection) -> (Connection, CommitProof) {
    let tx = conn.begin();
    // do work...
    tx.commit()  // tx consumed, cannot be used again
}

// This would NOT compile:
// fn bad_transaction(conn: Connection) {
//     let tx = conn.begin();
//     // forgot to commit or rollback
//     // ERROR: tx not consumed
// }
```

## Nested Transactions

```
# Outer transaction
OuterTx

# Begin nested (savepoint semantics)
OuterTx ⊸ OuterTx ⊗ InnerTx

# Inner commit (merge into outer)
OuterTx ⊗ InnerTx ⊸ OuterTx

# Inner rollback (revert inner only)
OuterTx ⊗ InnerTx ⊸ OuterTx

# Outer commit (commits all)
OuterTx ⊸ Connection ⊗ CommitProof
```

```bash
# Nested transaction that commits
$ cargo run -q -- prove "OuterTx |- Connection * CommitProof"
✓ PROVABLE

# Inner transaction must be handled before outer commits
$ cargo run -q -- prove "OuterTx, InnerTx |- Connection * InnerTx"
✗ NOT PROVABLE (inner must be committed/rolled back)
```

## Key Insights

| Linear Logic | Transaction Semantics |
|--------------|----------------------|
| `Connection ⊸ Transaction` | Begin transaction |
| `Transaction ⊸ Connection ⊗ Proof` | End transaction |
| No weakening | Cannot abandon transaction |
| No contraction | Cannot double-commit |
| `A ⊕ B` | Commit OR rollback (exactly one) |
| `!A` | Read-only snapshot (can share) |
| `A ⊗ B` | Both operations must complete |

## Real-World Mapping

```
Linear Logic           →  SQL / Database
──────────────────────────────────────────
Transaction            →  BEGIN
Connection ⊗ Commit    →  COMMIT (returns connection)
Connection ⊗ Rollback  →  ROLLBACK (returns connection)
Savepoint              →  SAVEPOINT name
InnerTx                →  Nested transaction
Prepared               →  2PC PREPARE
!Data                  →  Read-only snapshot
```

## Anti-Patterns Prevented

Linear logic makes these bugs **impossible**:

1. **Forgotten commit/rollback**: Transaction must be consumed
2. **Double commit**: Transaction consumed after first commit
3. **Use after rollback**: Transaction consumed by rollback
4. **Connection leak**: Connection must be returned
5. **Partial transaction**: All operations are part of atomic unit
