# Session Types and Communication Protocols

Linear logic naturally models session types - protocols that specify the order and type of messages in a communication channel. Each message "consumes" the current protocol state and produces the next state.

## The Problem

Network protocols have specific rules:

- Send request before receiving response
- Authenticate before accessing resources
- Cannot send on a closed connection

Traditional type systems don't enforce these orderings.

## Linear Logic Model

### Simple Request-Response Protocol

```
# Client endpoint: can send Request, then must receive Response
ClientStart ⊸ (SendRequest ⊗ ReceiveResponse)

# Server endpoint: receives Request, must send Response
ServerStart ⊸ (ReceiveRequest ⊗ SendResponse)
```

### Login Protocol

```
# Initial state: must authenticate
Unauthenticated

# After login: get authenticated session
Unauthenticated ⊗ Credentials ⊸ Authenticated

# Authenticated session can access resources
Authenticated ⊸ (Resource ⊗ Authenticated)

# Must eventually logout
Authenticated ⊸ LoggedOut
```

## Proving Protocol Correctness

### Valid Login Flow
```bash
# Login, access resource, logout
$ cargo run -q -- prove "Unauthenticated, Credentials |- Resource * LoggedOut"
```

This requires:
1. Using credentials to authenticate
2. Accessing the resource
3. Logging out (cannot skip!)

### Invalid Flows (Rejected)

```bash
# Cannot access without authentication
$ cargo run -q -- prove "Unauthenticated |- Resource"
✗ NOT PROVABLE

# Cannot skip logout
$ cargo run -q -- prove "Authenticated |- Resource"
✗ NOT PROVABLE (Resource alone, no LoggedOut)

# Cannot login twice with same credentials
$ cargo run -q -- prove "Credentials |- Authenticated * Authenticated"
✗ NOT PROVABLE
```

## Bidirectional Channel

Model a channel where both ends must be used:

```
# Create a channel pair
1 ⊸ (SendEnd ⊗ ReceiveEnd)

# Using the channel
SendEnd ⊗ Message ⊸ SentConfirmation
ReceiveEnd ⊸ (Message ⊗ ReceivedConfirmation)
```

```bash
# Prove that creating and using a channel is valid
$ cargo run -q -- prove "Message |- SentConfirmation * ReceivedConfirmation"
```

## Choice in Protocols: Branching

The `&` (with) and `⊕` (plus) connectives model protocol choices:

```
# Server offers two options (client chooses)
ServerChoice = GetData & PostData

# Client chooses one option
ClientChoice = GetData ⊕ PostData
```

```bash
# Client can choose to get data
$ cargo run -q -- prove "ServerChoice |- GetData"
✓ PROVABLE

# Or post data
$ cargo run -q -- prove "ServerChoice |- PostData"
✓ PROVABLE

# But not both from same session!
$ cargo run -q -- prove "ServerChoice |- GetData * PostData"
✗ NOT PROVABLE
```

## Reusable Services with !

```bash
# A service that can handle unlimited requests
$ cargo run -q -- prove "!Service |- Response * Response * Response"
✓ PROVABLE

# But a linear service handles exactly one
$ cargo run -q -- prove "Service |- Response * Response"
✗ NOT PROVABLE
```

## Generated Rust: Type-State Pattern

```bash
$ cargo run -q -- codegen "Unauthenticated, Credentials |- Authenticated"
```

```rust
// Generated type-state pattern
struct Unauthenticated;
struct Authenticated;
struct Credentials;

fn login(session: Unauthenticated, creds: Credentials) -> Authenticated {
    // session is consumed - can't be used as Unauthenticated anymore
    // Returns Authenticated - only way to get this type
    Authenticated
}
```

The Rust compiler enforces:
- Can't call `login` twice on same session
- Can't access authenticated APIs without `Authenticated` token
- Must have `Credentials` to authenticate

## Real-World Protocol: HTTP Request

```
# HTTP connection lifecycle
Connection

# Send request consumes connection, gets response + maybe connection
Connection ⊗ Request ⊸ (Response ⊗ (Connection ⊕ Closed))

# Response indicates keep-alive OR connection closed
# ⊕ means exactly one of the two outcomes
```

```bash
# Valid: make request, handle either outcome
$ cargo run -q -- prove "Connection, Request |- Response * (Connection + Closed)"
✓ PROVABLE
```

## Key Insights

| Concept | Protocol Meaning |
|---------|------------------|
| `A ⊸ B` | State transition from A to B |
| `A ⊗ B` | Must handle both A and B |
| `A & B` | Choice offered (other party picks) |
| `A ⊕ B` | Choice made (you pick) |
| `!A` | Reusable/persistent service |
| Linear types | Each protocol step exactly once |
