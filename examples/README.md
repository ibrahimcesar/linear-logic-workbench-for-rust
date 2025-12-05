# Lolli Examples

Practical examples showing how linear logic models real-world programming patterns. Each example demonstrates how Lolli can verify resource invariants and generate safe Rust code.

## Examples

| # | Example | Key Concepts |
|---|---------|--------------|
| 01 | [File Handle Protocol](01_file_handle.md) | Open/read/write/close lifecycle, use-after-close prevention |
| 02 | [Session Types](02_session_types.md) | Communication protocols, login flows, channels |
| 03 | [State Machines](03_state_machine.md) | Order processing, valid state transitions |
| 04 | [Resource Pools](04_resource_pool.md) | Connection pools, semaphores, thread pools |
| 05 | [Transactions](05_transactions.md) | ACID, two-phase commit, sagas, locking |

## Running Examples

Each example contains `cargo run` commands you can execute directly:

```bash
# Parse a formula
cargo run -q -- parse "File * Data -o ClosedFile"

# Prove a sequent is valid
cargo run -q -- prove "OpenFile, Data |- ClosedFile"

# Generate Rust code
cargo run -q -- codegen "A, B |- A * B"
```

The `-q` flag suppresses build output for cleaner results.

## Common Patterns

### Resource Lifecycle

```
Resource ⊸ ActiveResource           # Acquire
ActiveResource ⊗ Work ⊸ Result      # Use
ActiveResource ⊸ Done               # Release
```

### Exclusive vs Shared Access

```
WriteHandle                         # Linear: exclusive access
!ReadHandle                         # Exponential: can be shared/copied
```

### State Transitions

```
StateA ⊸ StateB                     # A transitions to B
StateB ⊸ StateC ⊕ StateD            # B goes to C or D (choice)
```

### Must-Use Resources

```
Token ⊗ Operation ⊸ Result ⊗ Token  # Token preserved through operation
```

## Linear Logic Quick Reference

| Symbol | ASCII | Meaning |
|--------|-------|---------|
| `⊸` | `-o` | Consume A, produce B |
| `⊗` | `*` | Both A and B (tensor) |
| `⊕` | `+` | A or B, I choose (plus) |
| `&` | `&` | A or B, you choose (with) |
| `!` | `!` | Unlimited copies (bang) |
| `?` | `?` | Demand (why-not) |

## Why Linear Logic?

Linear logic prevents:

- **Use-after-free**: Resource consumed after release
- **Double-free**: Resource released twice
- **Leaks**: Resource never released
- **Data races**: Shared mutable state
- **Protocol violations**: Wrong state transitions

If Lolli proves your specification valid, and you generate code from it, Rust's ownership system enforces these invariants at compile time.

## Creating Your Own Examples

1. **Identify resources**: What must be tracked? (files, connections, tokens)
2. **Model states**: What states can resources be in? (open, closed, locked)
3. **Define transitions**: What operations change state? (read, write, commit)
4. **Express in linear logic**: Use `⊸` for transitions, `⊗` for combinations
5. **Prove with Lolli**: Verify your invariants hold
6. **Generate code**: Get Rust that enforces the specification

Example workflow:

```bash
# 1. Start with your specification
echo "Connection -o Transaction" > my_spec.ll

# 2. Verify it makes sense
cargo run -q -- prove "Connection |- Transaction"

# 3. Check safety properties
cargo run -q -- prove "Transaction |- Transaction * Transaction"  # Should fail (no dup)

# 4. Generate implementation skeleton
cargo run -q -- codegen "Connection |- Connection * CommitProof"
```

## Further Reading

- [README](../README.md) - Full CLI reference and syntax guide
- Girard, "Linear Logic" (1987) - The original paper
- Wadler, "A Taste of Linear Logic" (1993) - Accessible introduction
- Caires & Pfenning, "Session Types as Intuitionistic Linear Propositions" (2010)
