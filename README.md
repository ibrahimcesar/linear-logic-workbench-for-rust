<div align="center">

# Lolli ⊸

**Prove it linear, ship it safe.**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

</div>

Lolli is a linear logic toolkit that turns resource specifications into verified Rust code. Describe how resources flow — what gets consumed, what gets produced, which operations are exclusive — and Lolli proves your specification is valid, extracts a program from the proof, and generates Rust where ownership enforces the invariants at compile time. No runtime checks, no manual discipline: if it compiles, the resources are handled correctly.

> [!NOTE]
> **Why "Lolli"?**
>
> In linear logic, A ⊸ B is called the "lollipop" — consume A, produce B, exactly once. It's the connective that makes resource reasoning precise: no accidental copies, no forgotten cleanups, no use-after-free. Girard introduced it in 1987; Rust's ownership system operationalizes it today. Lolli is named for this symbol because the tool embodies what it represents: proving that resources flow correctly, then generating code that enforces it.

## Features

- **Parse** linear logic formulas with standard notation (Unicode and ASCII)
- **Prove** sequents automatically using focused proof search
- **Extract** λ-terms from proofs via Curry-Howard correspondence
- **Generate** Rust code that enforces linearity through ownership
- **Visualize** proofs as ASCII trees, LaTeX, or Graphviz DOT

## Installation

```bash
git clone https://github.com/ibrahimcesar/lolli.git
cd lolli
cargo build --release
```

## Quick Start

```bash
# Parse a formula
cargo run -- parse "A * B -o B * A"

# Prove a sequent
cargo run -- prove "A, B |- A * B"

# Extract a term from a proof
cargo run -- extract "A -o B, B -o C |- A -o C"

# Generate Rust code
cargo run -- codegen "A, B |- A * B"

# Visualize a proof
cargo run -- viz "A |- A" --format latex

# Interactive REPL
cargo run -- repl
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `parse <formula>` | Parse and analyze a formula |
| `prove <sequent>` | Check if a sequent is provable |
| `extract <sequent>` | Extract a λ-term from a proof |
| `codegen <sequent>` | Generate Rust code from a proof |
| `viz <sequent>` | Visualize a proof (tree, latex, dot) |
| `repl` | Interactive REPL mode |

### Command Options

```bash
# Prove with custom depth limit
cargo run -- prove "A |- A" --depth 50

# Visualize in different formats
cargo run -- viz "A, B |- A * B" --format tree    # ASCII tree (default)
cargo run -- viz "A, B |- A * B" --format latex   # LaTeX (bussproofs)
cargo run -- viz "A, B |- A * B" --format dot     # Graphviz DOT

# Parse with different output modes
cargo run -- parse "A * B" --latex   # LaTeX output
cargo run -- parse "A * B" --ascii   # ASCII-only output
```

## Syntax

### Connectives

| Symbol | ASCII | Name | Meaning |
|--------|-------|------|---------|
| `⊗` | `*` | Tensor | Both A and B (simultaneously) |
| `⅋` | `\|` | Par | A or B (opponent chooses) |
| `⊸` | `-o` | Lollipop | Consume A, produce B |
| `&` | `&` | With | Both available, choose one |
| `⊕` | `+` | Plus | One of them (I choose) |
| `!` | `!` | Bang | Unlimited supply (can copy) |
| `?` | `?` | Why-not | Demand for resource |
| `1` | `1` | One | Unit for tensor |
| `⊥` | `bot` | Bottom | Unit for par |
| `⊤` | `top` | Top | Always satisfiable |
| `0` | `zero` | Zero | Never satisfiable |
| `⊥` (suffix) | `^` | Negation | Linear negation |

### Sequent Notation

```
A, B |- C          # A and B prove C
A |- B * C         # A proves B tensor C
!A |- A * A        # Bang A proves A used twice
```

## Examples

### Identity
```bash
$ cargo run -q -- prove "A |- A"
✓ PROVABLE

Proof:
⊢ A⊥, A  (Axiom)
```

### Tensor Introduction
```bash
$ cargo run -q -- prove "A, B |- A * B"
✓ PROVABLE

Proof:
    ⊢ A⊥, A  (Axiom)
    ⊢ B⊥, B  (Axiom)
──────────────────────  TensorIntro
⊢ A⊥, B⊥, (A ⊗ B)
```

### Contraction with Bang
```bash
$ cargo run -q -- prove "!A |- A * A"
✓ PROVABLE

# Uses A twice via the ! modality
```

### Code Generation
```bash
$ cargo run -q -- codegen "A, B |- A * B"

fn f(arg0: A, arg1: B) -> (A, B) {
    (arg0, arg1)
}
```

## Linear Logic to Rust Mapping

| Linear Logic | Rust Type |
|--------------|-----------|
| `A ⊸ B` | `impl FnOnce(A) -> B` |
| `A ⊗ B` | `(A, B)` |
| `A & B` | `With<A, B>` (lazy pair) |
| `A ⊕ B` | `Either<A, B>` |
| `!A` | `Rc<A>` (shareable) |
| `1` | `()` |
| `⊤` | `Top` |
| `0` | `Void` (empty type) |

## Architecture

```
lolli/
├── lolli-core      # Formula, Sequent, Proof, Term types
├── lolli-parse     # Pest grammar and parser
├── lolli-prove     # Focused proof search (MALL + MELL)
├── lolli-extract   # Curry-Howard term extraction
├── lolli-codegen   # Rust code generation
├── lolli-viz       # ASCII, LaTeX, Graphviz rendering
└── lolli-cli       # Command-line interface
```

## Supported Logic Fragments

| Fragment | Connectives | Status |
|----------|-------------|--------|
| MLL | ⊗, ⅋, 1, ⊥, ⊸ | ✓ Complete |
| MALL | MLL + &, ⊕, ⊤, 0 | ✓ Complete |
| MELL | MLL + !, ? | ✓ Complete |

## Status & Roadmap

### Current Status (v0.2.0)

Lolli is functional and suitable for:
- **Educational use** — Learning linear logic and the Curry-Howard correspondence
- **Prototyping** — Modeling resource protocols before implementation
- **Small specifications** — Verifying sequents with ~10-20 connectives
- **Code scaffolding** — Generating type-safe Rust API skeletons

### Known Limitations

| Area | Limitation | Impact |
|------|------------|--------|
| **Performance** | Proof search is exponential | Large formulas may timeout |
| **Scalability** | Single-threaded, in-memory | Not suitable for batch processing |
| **Error handling** | Parser fails on first error | No error recovery or suggestions |
| **Code generation** | Produces skeletons only | Manual implementation still needed |
| **Verification** | Prover not formally verified | Suitable for prototyping, not certification |

### Path to 1.0.0

| Version | Focus | Key Deliverables |
|---------|-------|------------------|
| **v0.3.0** | Robustness | Better errors, input validation, edge cases |
| **v0.4.0** | Performance | Proof caching, pruning, benchmarks |
| **v0.5.0** | Usability | LSP support, better REPL, documentation |
| **v1.0.0** | Production | Stability guarantees, full test coverage |

See [GitHub Milestones](https://github.com/ibrahimcesar/lolli/milestones) for detailed tracking.

## Development

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test --package lolli-prove

# Build documentation
cargo doc --workspace --open
```

**118 tests** across all crates.

## References

- Girard, "Linear Logic" (1987)
- Andreoli, "Logic Programming with Focusing Proofs" (1992)
- Wadler, "A Taste of Linear Logic" (1993)

## License

MIT License. See [LICENSE](LICENSE) for details.

---

<div align="center">
<strong>⊸</strong> <i>Consume once, produce safely.</i>
</div>
