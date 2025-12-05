# State Machines with Linear Logic

Linear logic excels at modeling state machines where each state transition consumes the current state and produces the next. This prevents impossible state transitions and ensures the machine follows valid paths.

## The Problem

State machines in traditional code often suffer from:
- Invalid state transitions
- States being "forgotten" (stuck in intermediate state)
- Race conditions from concurrent state access

## Example: Order Processing

An e-commerce order goes through specific states:

```
States:
- Pending
- Paid
- Shipped
- Delivered
- Cancelled

Transitions:
- Pending + Payment → Paid
- Paid → Shipped
- Shipped → Delivered
- Pending → Cancelled (can only cancel before payment)
```

## Linear Logic Model

```
# State transitions as linear implications
Pending ⊗ Payment ⊸ Paid
Paid ⊸ Shipped
Shipped ⊸ Delivered
Pending ⊸ Cancelled
```

## Proving Valid Paths

### Complete Order Flow
```bash
# Order placed, paid, shipped, delivered
$ cargo run -q -- prove "Pending, Payment |- Delivered"
✓ PROVABLE
```

The proof shows the path: Pending → Paid → Shipped → Delivered

### Cancellation
```bash
# Can cancel a pending order
$ cargo run -q -- prove "Pending |- Cancelled"
✓ PROVABLE
```

## Proving INVALID Paths (Safety)

```bash
# Cannot cancel after payment
$ cargo run -q -- prove "Paid |- Cancelled"
✗ NOT PROVABLE

# Cannot skip from Pending to Shipped
$ cargo run -q -- prove "Pending |- Shipped"
✗ NOT PROVABLE

# Cannot go backwards: Delivered to Shipped
$ cargo run -q -- prove "Delivered |- Shipped"
✗ NOT PROVABLE

# Cannot be in two states at once
$ cargo run -q -- prove "Pending |- Paid * Pending"
✗ NOT PROVABLE
```

## Branching State Machines

Using `⊕` (plus) for choices the system makes, `&` (with) for external choices:

```
# After payment processing, either succeeds or fails
Pending ⊗ Payment ⊸ (Paid ⊕ PaymentFailed)

# Customer can choose: proceed or request refund
Paid ⊸ (Shipped & RefundRequested)
```

```bash
# Payment might fail
$ cargo run -q -- prove "Pending, Payment |- Paid + PaymentFailed"
✓ PROVABLE

# From Paid, customer chooses ship or refund
$ cargo run -q -- prove "Paid |- Shipped"
✓ PROVABLE

$ cargo run -q -- prove "Paid |- RefundRequested"
✓ PROVABLE
```

## Parallel State Machines

Multiple items in an order, each with their own state:

```bash
# Two items: both must reach Delivered
$ cargo run -q -- prove "Pending, Pending, Payment, Payment |- Delivered * Delivered"
✓ PROVABLE

# Cannot deliver one item with payment for both
$ cargo run -q -- prove "Pending, Pending, Payment |- Delivered * Delivered"
✗ NOT PROVABLE (need 2 payments for 2 items)
```

## State Machine with Retries (Exponentials)

```
# Unlimited retry attempts
!RetryToken

# Each attempt consumes a token
RetryToken ⊗ Failed ⊸ (Success ⊕ Failed)
```

```bash
# With unlimited retries, can keep trying
$ cargo run -q -- prove "!RetryToken, Failed |- Success"
✓ PROVABLE (will eventually find Success path)
```

## Generated Rust: Type-State Pattern

```bash
$ cargo run -q -- codegen "Pending, Payment |- Paid"
```

```rust
struct Pending;
struct Payment;
struct Paid;

// Only way to get Paid is from Pending + Payment
fn process_payment(order: Pending, payment: Payment) -> Paid {
    // order and payment are consumed
    Paid
}

// Compiler prevents:
// - Calling process_payment twice on same order
// - Creating Paid without payment
// - Using Pending after it's been processed
```

## Complex Example: Document Workflow

```
Draft → (Submitted ⊕ Deleted)           # Author submits or deletes
Submitted → (Approved ⊕ Rejected)       # Reviewer decides
Rejected → Draft                         # Back to author
Approved → Published                     # Final state
```

```bash
# Valid: draft → submit → approve → publish
$ cargo run -q -- prove "Draft |- Published"
✓ PROVABLE (via Submitted, Approved)

# Valid: draft → submit → reject → (back to draft) → submit → approve → publish
$ cargo run -q -- prove "Draft |- Published"
✓ PROVABLE (includes rejection path)

# Invalid: cannot publish without approval
$ cargo run -q -- prove "Submitted |- Published"
✗ NOT PROVABLE
```

## Visualization

```bash
$ cargo run -q -- viz "Pending, Payment |- Delivered" --format tree
```

Shows the proof tree (state transition path):
```
      ⊢ Pending⊥, Paid  (Pending ⊗ Payment ⊸ Paid)
          ⊢ Paid⊥, Shipped  (Paid ⊸ Shipped)
              ⊢ Shipped⊥, Delivered  (Shipped ⊸ Delivered)
```

## Key Insights

| Linear Logic | State Machine Meaning |
|--------------|----------------------|
| State type | Current state token |
| `A ⊸ B` | Valid transition A → B |
| `A ⊗ B ⊸ C` | Transition needs both A and B |
| `A ⊸ B ⊕ C` | Transition to B or C (system chooses) |
| `A ⊸ B & C` | From A, can go to B or C (external choice) |
| No contraction | Can't clone state (no race conditions) |
| No weakening | Can't discard state (no stuck states) |
