# Linear Logic Workbench

## Project Overview

A toolkit for working with linear logic — parsing formulas, searching for proofs, extracting computational content, and compiling to Rust. Linear logic is the type-theoretic foundation of Rust's ownership system; this tool makes that connection explicit and exploitable.

**Core thesis**: Linear logic isn't just theory — it's a practical tool for reasoning about resources. This workbench lets you write specifications in linear logic and extract verified Rust code that enforces those resource invariants at compile time.

### What This Tool Does

```
┌─────────────────────────────────────────────────────────────────┐
│                    Linear Logic Workbench                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   Formula ──▶ Parser ──▶ AST                                   │
│                           │                                     │
│                           ▼                                     │
│                      Proof Search                               │
│                           │                                     │
│                           ▼                                     │
│                    Proof / Proof Net                            │
│                      │         │                                │
│           ┌──────────┘         └──────────┐                     │
│           ▼                               ▼                     │
│      Term Extraction                 Visualization              │
│           │                               │                     │
│           ▼                               ▼                     │
│    Rust Code Gen                    SVG / Graphviz              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Goals

1. **Parse**: Linear logic formulas with standard notation
2. **Prove**: Automated proof search for provable sequents
3. **Extract**: Get λ-terms (programs) from proofs via Curry-Howard
4. **Compile**: Generate Rust code that enforces linearity
5. **Visualize**: Render proofs as trees or proof nets
6. **Educate**: Make linear logic accessible to Rust programmers

### Non-Goals (for v1)

- Full first-order linear logic (quantifiers)
- Proof assistant features (tactics, interactive proving)
- Non-commutative linear logic
- Differential linear logic

---

## Linear Logic Foundations

### Why Linear Logic?

Classical and intuitionistic logic have **structural rules** that let you:
- **Weaken**: Ignore hypotheses you don't need
- **Contract**: Use hypotheses multiple times

These are fine for truth, but terrible for resources. If A means "I have $10", then A ⊢ A ⊗ A would mean "If I have $10, I have $20". That's counterfeiting.

**Linear logic** (Girard, 1987) removes these rules. Every hypothesis must be used **exactly once**. This makes it a logic of resources, actions, and state change.

### Connectives

Linear logic has a rich set of connectives, split into **multiplicative** and **additive** families:

#### Multiplicative (about parallel/sequential composition)

| Connective | Symbol | Name | Meaning |
|------------|--------|------|---------|
| Tensor | A ⊗ B | "times" | Both A and B (independently) |
| Par | A ⅋ B | "par" | A or B (opponent chooses) |
| Linear implication | A ⊸ B | "lollipop" | Consume A, produce B |
| One | 1 | "one" | Unit for ⊗ (no resource) |
| Bottom | ⊥ | "bottom" | Unit for ⅋ |

#### Additive (about choice)

| Connective | Symbol | Name | Meaning |
|------------|--------|------|---------|
| With | A & B | "with" | Both available, choose one |
| Plus | A ⊕ B | "plus" | One of them (I choose which) |
| Top | ⊤ | "top" | Unit for & (trivially satisfiable) |
| Zero | 0 | "zero" | Unit for ⊕ (impossible) |

#### Exponentials (controlled structural rules)

| Connective | Symbol | Name | Meaning |
|------------|--------|------|---------|
| Of course | !A | "bang" | Unlimited supply of A (can copy/discard) |
| Why not | ?A | "whynot" | Demand for A |

### Negation and Duality

Linear logic has an involutive negation: (A⊥)⊥ = A

De Morgan dualities:
```
(A ⊗ B)⊥ = A⊥ ⅋ B⊥
(A ⅋ B)⊥ = A⊥ ⊗ B⊥
(A & B)⊥ = A⊥ ⊕ B⊥
(A ⊕ B)⊥ = A⊥ & B⊥
1⊥ = ⊥
⊥⊥ = 1
(!A)⊥ = ?(A⊥)
(?A)⊥ = !(A⊥)
```

### Fragments We Support

| Fragment | Connectives | Notes |
|----------|-------------|-------|
| MLL | ⊗, ⅋, 1, ⊥, ⊸, (-)⊥ | Multiplicative only, decidable |
| MALL | MLL + &, ⊕, ⊤, 0 | + Additives, decidable |
| MELL | MLL + !, ? | + Exponentials, undecidable |
| Full LL | All | The whole system |

**v1 focuses on MALL** — it's decidable, has good proof search, and captures most of the interesting resource reasoning.

### Sequent Calculus

We use one-sided sequent calculus: ⊢ Γ where Γ is a multiset of formulas.

The idea: proving ⊢ A, B, C means "from nothing, I can produce A and B and C (in the par sense)".

The two-sided sequent A₁, ..., Aₙ ⊢ B translates to ⊢ A₁⊥, ..., Aₙ⊥, B.

#### Identity Rules

```
────────── (ax)
⊢ A⊥, A

⊢ Γ, A    ⊢ Δ, A⊥
──────────────────── (cut)
⊢ Γ, Δ
```

#### Multiplicative Rules

```
─────────── (1)
⊢ 1

⊢ Γ
─────────── (⊥)
⊢ Γ, ⊥

⊢ Γ, A    ⊢ Δ, B
──────────────────── (⊗)
⊢ Γ, Δ, A ⊗ B

⊢ Γ, A, B
────────────── (⅋)
⊢ Γ, A ⅋ B
```

#### Additive Rules

```
─────────── (⊤)
⊢ Γ, ⊤

(no rule for 0)

⊢ Γ, A    ⊢ Γ, B
──────────────────── (&)
⊢ Γ, A & B

⊢ Γ, A
────────────── (⊕₁)
⊢ Γ, A ⊕ B

⊢ Γ, B
────────────── (⊕₂)
⊢ Γ, A ⊕ B
```

#### Exponential Rules

```
⊢ ?Γ, A
──────────── (!)
⊢ ?Γ, !A

⊢ Γ, A
──────────── (?)
⊢ Γ, ?A

⊢ Γ
──────────── (?w)     [weakening]
⊢ Γ, ?A

⊢ Γ, ?A, ?A
──────────── (?c)     [contraction]
⊢ Γ, ?A
```

---

## Connection to Rust

### The Core Insight

Rust's type system is affine (use at most once), not linear (use exactly once). But the connection is tight:

| Linear Logic | Rust |
|--------------|------|
| A ⊗ B | `(A, B)` tuple |
| A ⊸ B | `fn(A) -> B` (consuming) |
| A & B | `enum { Left(A), Right(B) }` with pattern match required |
| A ⊕ B | `enum { Left(A), Right(B) }` you construct |
| !A | `A: Copy` or `&A` |
| 1 | `()` |
| 0 | `!` (never type) |

### Example: File Handle Protocol

Linear logic:
```
FileHandle ⊸ (Contents ⊗ ClosedHandle)
```

Meaning: consuming a FileHandle produces Contents and a ClosedHandle. You can't:
- Use the handle after reading (it's consumed)
- Forget to close (you must produce ClosedHandle)
- Read twice (handle is linear)

Rust translation:
```rust
fn read_and_close(handle: FileHandle) -> (Contents, ClosedHandle) {
    let contents = handle.read();
    let closed = handle.close();
    (contents, closed)
}
```

The Rust compiler enforces this via ownership.

### Example: Session Type

Linear logic:
```
!Request ⊸ Response
```

A server that can handle unlimited requests (!) and produces responses.

```rust
fn server(requests: impl Iterator<Item = Request>) -> impl Iterator<Item = Response> {
    requests.map(|req| handle(req))
}
```

---

## Architecture

```
linear-logic-workbench/
│
├── CONTEXT.md
├── README.md
├── LICENSE
│
├── crates/
│   ├── llw-core/                    # Core data structures
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── formula.rs           # Formula AST
│   │   │   ├── sequent.rs           # Sequents
│   │   │   ├── proof.rs             # Proof trees
│   │   │   ├── proof_net.rs         # Proof nets
│   │   │   └── term.rs              # λ-terms (extracted)
│   │   └── Cargo.toml
│   │
│   ├── llw-parse/                   # Parser
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── lexer.rs
│   │   │   ├── parser.rs
│   │   │   └── pretty.rs            # Pretty printing
│   │   ├── src/grammar.pest         # Pest grammar
│   │   └── Cargo.toml
│   │
│   ├── llw-prove/                   # Proof search
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── search.rs            # Main proof search
│   │   │   ├── focused.rs           # Focused sequent calculus
│   │   │   ├── inverse.rs           # Inverse method
│   │   │   └── decide.rs            # Decision procedures
│   │   └── Cargo.toml
│   │
│   ├── llw-extract/                 # Term extraction
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── extract.rs           # Proof → Term
│   │   │   ├── reduce.rs            # Term reduction
│   │   │   └── normalize.rs         # Normalization
│   │   └── Cargo.toml
│   │
│   ├── llw-codegen/                 # Code generation
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── rust.rs              # Rust code generation
│   │   │   ├── typescript.rs        # TypeScript generation
│   │   │   └── templates/           # Code templates
│   │   └── Cargo.toml
│   │
│   ├── llw-viz/                     # Visualization
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── tree.rs              # Proof tree rendering
│   │   │   ├── net.rs               # Proof net rendering
│   │   │   └── graphviz.rs          # Graphviz output
│   │   └── Cargo.toml
│   │
│   └── llw-cli/                     # Command-line interface
│       ├── src/
│       │   └── main.rs
│       └── Cargo.toml
│
├── examples/
│   ├── basic/
│   │   ├── identity.ll              # A ⊸ A
│   │   ├── tensor_intro.ll          # A, B ⊢ A ⊗ B
│   │   └── currying.ll              # A ⊗ B ⊸ C ⊣⊢ A ⊸ B ⊸ C
│   ├── protocols/
│   │   ├── file_handle.ll
│   │   ├── channel.ll
│   │   └── state_machine.ll
│   └── generated/                   # Generated Rust code
│
├── tests/
│   ├── parsing_tests.rs
│   ├── proving_tests.rs
│   ├── extraction_tests.rs
│   └── codegen_tests.rs
│
└── docs/
    ├── tutorial.md
    ├── logic_reference.md
    └── rust_correspondence.md
```

---

## Core Data Structures

### Formula Representation

```rust
// llw-core/src/formula.rs

/// A linear logic formula
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Formula {
    // Atoms
    Atom(String),
    NegAtom(String),              // A⊥
    
    // Multiplicatives
    Tensor(Box<Formula>, Box<Formula>),   // A ⊗ B
    Par(Box<Formula>, Box<Formula>),       // A ⅋ B
    One,                                   // 1
    Bottom,                                // ⊥
    
    // Additives
    With(Box<Formula>, Box<Formula>),      // A & B
    Plus(Box<Formula>, Box<Formula>),      // A ⊕ B
    Top,                                   // ⊤
    Zero,                                  // 0
    
    // Exponentials
    OfCourse(Box<Formula>),                // !A
    WhyNot(Box<Formula>),                  // ?A
    
    // Derived (sugar, but useful)
    Lolli(Box<Formula>, Box<Formula>),     // A ⊸ B = A⊥ ⅋ B
}

impl Formula {
    /// Compute the linear negation
    pub fn negate(&self) -> Formula {
        match self {
            Formula::Atom(a) => Formula::NegAtom(a.clone()),
            Formula::NegAtom(a) => Formula::Atom(a.clone()),
            
            Formula::Tensor(a, b) => Formula::Par(
                Box::new(a.negate()),
                Box::new(b.negate())
            ),
            Formula::Par(a, b) => Formula::Tensor(
                Box::new(a.negate()),
                Box::new(b.negate())
            ),
            Formula::One => Formula::Bottom,
            Formula::Bottom => Formula::One,
            
            Formula::With(a, b) => Formula::Plus(
                Box::new(a.negate()),
                Box::new(b.negate())
            ),
            Formula::Plus(a, b) => Formula::With(
                Box::new(a.negate()),
                Box::new(b.negate())
            ),
            Formula::Top => Formula::Zero,
            Formula::Zero => Formula::Top,
            
            Formula::OfCourse(a) => Formula::WhyNot(Box::new(a.negate())),
            Formula::WhyNot(a) => Formula::OfCourse(Box::new(a.negate())),
            
            Formula::Lolli(a, b) => Formula::Tensor(
                a.clone(),
                Box::new(b.negate())
            ),
        }
    }
    
    /// Desugar A ⊸ B to A⊥ ⅋ B
    pub fn desugar(&self) -> Formula {
        match self {
            Formula::Lolli(a, b) => Formula::Par(
                Box::new(a.negate().desugar()),
                Box::new(b.desugar())
            ),
            Formula::Tensor(a, b) => Formula::Tensor(
                Box::new(a.desugar()),
                Box::new(b.desugar())
            ),
            // ... recursively desugar all constructors
            _ => self.clone()
        }
    }
    
    /// Is this formula positive (async/eager)?
    pub fn is_positive(&self) -> bool {
        matches!(self, 
            Formula::Atom(_) |
            Formula::Tensor(_, _) |
            Formula::One |
            Formula::Plus(_, _) |
            Formula::Zero |
            Formula::OfCourse(_)
        )
    }
    
    /// Is this formula negative (sync/lazy)?
    pub fn is_negative(&self) -> bool {
        !self.is_positive()
    }
}
```

### Sequent Representation

```rust
// llw-core/src/sequent.rs

use std::collections::HashMap;

/// A one-sided sequent ⊢ Γ
/// 
/// We use a zone-based representation for focused proof search:
/// - Linear zone: formulas that must be used exactly once
/// - Unrestricted zone: formulas under ? that can be reused
#[derive(Clone, Debug)]
pub struct Sequent {
    /// Linear hypotheses (multiset, represented as Vec)
    pub linear: Vec<Formula>,
    
    /// Unrestricted hypotheses (under ?)
    pub unrestricted: Vec<Formula>,
    
    /// Currently focused formula (if any)
    pub focus: Option<Formula>,
}

impl Sequent {
    pub fn new(formulas: Vec<Formula>) -> Self {
        Sequent {
            linear: formulas,
            unrestricted: vec![],
            focus: None,
        }
    }
    
    /// Check if sequent is empty (proven)
    pub fn is_empty(&self) -> bool {
        self.linear.is_empty() && self.focus.is_none()
    }
    
    /// Focus on a formula
    pub fn focus_on(&self, idx: usize) -> Option<Sequent> {
        if idx >= self.linear.len() {
            return None;
        }
        
        let mut new_linear = self.linear.clone();
        let focused = new_linear.remove(idx);
        
        Some(Sequent {
            linear: new_linear,
            unrestricted: self.unrestricted.clone(),
            focus: Some(focused),
        })
    }
    
    /// Unfocus, returning formula to linear zone
    pub fn unfocus(&self) -> Sequent {
        let mut new_linear = self.linear.clone();
        if let Some(f) = &self.focus {
            new_linear.push(f.clone());
        }
        
        Sequent {
            linear: new_linear,
            unrestricted: self.unrestricted.clone(),
            focus: None,
        }
    }
}

/// A two-sided sequent Γ ⊢ Δ (for user-facing API)
#[derive(Clone, Debug)]
pub struct TwoSidedSequent {
    pub antecedent: Vec<Formula>,
    pub succedent: Vec<Formula>,
}

impl TwoSidedSequent {
    /// Convert to one-sided: Γ ⊢ Δ becomes ⊢ Γ⊥, Δ
    pub fn to_one_sided(&self) -> Sequent {
        let mut formulas: Vec<Formula> = self.antecedent
            .iter()
            .map(|f| f.negate())
            .collect();
        formulas.extend(self.succedent.clone());
        
        Sequent::new(formulas)
    }
}
```

### Proof Trees

```rust
// llw-core/src/proof.rs

/// A proof in the sequent calculus
#[derive(Clone, Debug)]
pub struct Proof {
    pub conclusion: Sequent,
    pub rule: Rule,
    pub premises: Vec<Proof>,
}

/// Inference rules
#[derive(Clone, Debug)]
pub enum Rule {
    // Identity
    Axiom,                          // ⊢ A⊥, A
    Cut(Formula),                   // cut on formula A
    
    // Multiplicatives
    OneIntro,                       // ⊢ 1
    BottomIntro,                    // ⊢ Γ, ⊥ from ⊢ Γ
    TensorIntro,                    // ⊢ Γ, Δ, A ⊗ B from ⊢ Γ, A and ⊢ Δ, B
    ParIntro,                       // ⊢ Γ, A ⅋ B from ⊢ Γ, A, B
    
    // Additives
    TopIntro,                       // ⊢ Γ, ⊤
    WithIntro,                      // ⊢ Γ, A & B from ⊢ Γ, A and ⊢ Γ, B
    PlusIntroLeft,                  // ⊢ Γ, A ⊕ B from ⊢ Γ, A
    PlusIntroRight,                 // ⊢ Γ, A ⊕ B from ⊢ Γ, B
    
    // Exponentials
    OfCourseIntro,                  // ⊢ ?Γ, !A from ⊢ ?Γ, A
    WhyNotIntro,                    // ⊢ Γ, ?A from ⊢ Γ, A
    Weakening,                      // ⊢ Γ, ?A from ⊢ Γ
    Contraction,                    // ⊢ Γ, ?A from ⊢ Γ, ?A, ?A
    Dereliction,                    // ⊢ Γ, A from ⊢ Γ, ?A (implicit in some presentations)
    
    // Focused rules
    FocusPositive(usize),           // Focus on positive formula at index
    FocusNegative(usize),           // Focus on negative formula at index
    Blur,                           // Unfocus
}

impl Proof {
    /// Verify that this proof is valid
    pub fn verify(&self) -> Result<(), ProofError> {
        // Check that the rule application is valid
        // Check that premises have correct conclusions
        // Recursively verify premises
        todo!()
    }
    
    /// Count the number of cut rules (for cut-elimination)
    pub fn cut_count(&self) -> usize {
        let self_cuts = if matches!(self.rule, Rule::Cut(_)) { 1 } else { 0 };
        let premise_cuts: usize = self.premises.iter()
            .map(|p| p.cut_count())
            .sum();
        self_cuts + premise_cuts
    }
    
    /// Is this proof cut-free?
    pub fn is_cut_free(&self) -> bool {
        self.cut_count() == 0
    }
}
```

### Proof Nets (Alternative Representation)

```rust
// llw-core/src/proof_net.rs

use petgraph::Graph;

/// A proof net - a graph representation of proofs
/// 
/// Proof nets are canonical: different proofs of the same sequent
/// that differ only by inessential rule orderings give the same net.
#[derive(Clone, Debug)]
pub struct ProofNet {
    /// The underlying graph
    pub graph: Graph<NetNode, NetEdge>,
    
    /// Conclusion ports (boundary of the net)
    pub conclusions: Vec<NodeIndex>,
}

#[derive(Clone, Debug)]
pub enum NetNode {
    // Axiom link
    Axiom,
    
    // Multiplicative
    Tensor,
    Par,
    One,
    Bottom,
    
    // Additive (boxes for additives)
    WithBox { left: Box<ProofNet>, right: Box<ProofNet> },
    PlusLeft,
    PlusRight,
    
    // Exponential (boxes for !)
    OfCourseBox { inner: Box<ProofNet> },
    WhyNot,
    Weakening,
    Contraction,
}

#[derive(Clone, Debug)]
pub struct NetEdge {
    pub formula: Formula,
}

impl ProofNet {
    /// Check if this is a valid proof net (correctness criterion)
    /// 
    /// For MLL: Danos-Regnier criterion (switching/contractibility)
    pub fn is_valid(&self) -> bool {
        self.check_danos_regnier()
    }
    
    fn check_danos_regnier(&self) -> bool {
        // For each ⅋ node, we can choose to "switch" it left or right
        // The net is valid iff all switchings produce acyclic, connected graphs
        todo!()
    }
    
    /// Convert from proof tree to proof net
    pub fn from_proof(proof: &Proof) -> Self {
        todo!()
    }
    
    /// Normalize the proof net (cut-elimination at the net level)
    pub fn normalize(&self) -> Self {
        todo!()
    }
}
```

### Extracted Terms

```rust
// llw-core/src/term.rs

/// Linear λ-terms extracted from proofs
#[derive(Clone, Debug)]
pub enum Term {
    // Variables
    Var(String),
    
    // Multiplicatives
    Unit,                                       // ()
    Pair(Box<Term>, Box<Term>),                // (a, b)
    LetPair(String, String, Box<Term>, Box<Term>), // let (x, y) = e in e'
    
    // Linear functions
    Abs(String, Box<Term>),                    // λx. e
    App(Box<Term>, Box<Term>),                 // e e'
    
    // Additives
    Inl(Box<Term>),                            // inl e
    Inr(Box<Term>),                            // inr e
    Case(Box<Term>, String, Box<Term>, String, Box<Term>), // case e of inl x => e1 | inr y => e2
    Trivial,                                   // ⟨⟩ (unit for &)
    Fst(Box<Term>),                            // fst e
    Snd(Box<Term>),                            // snd e
    Abort(Box<Term>),                          // absurd e (for 0)
    
    // Exponentials
    Promote(Box<Term>),                        // !e
    Derelict(Box<Term>),                       // let !x = e in e'
    Discard(Box<Term>, Box<Term>),             // discard e in e'
    Copy(Box<Term>, String, String, Box<Term>), // copy e as (x, y) in e'
}

impl Term {
    /// Perform one step of β-reduction
    pub fn reduce_step(&self) -> Option<Term> {
        match self {
            // β for functions
            Term::App(f, a) => {
                if let Term::Abs(x, body) = f.as_ref() {
                    Some(body.substitute(x, a))
                } else {
                    None
                }
            }
            // β for pairs
            Term::LetPair(x, y, pair, body) => {
                if let Term::Pair(a, b) = pair.as_ref() {
                    let body = body.substitute(x, a);
                    Some(body.substitute(y, b))
                } else {
                    None
                }
            }
            // β for sums
            Term::Case(scrutinee, x, left, y, right) => {
                match scrutinee.as_ref() {
                    Term::Inl(a) => Some(left.substitute(x, a)),
                    Term::Inr(b) => Some(right.substitute(y, b)),
                    _ => None,
                }
            }
            // ... more reduction rules
            _ => None,
        }
    }
    
    /// Substitute term for variable
    pub fn substitute(&self, var: &str, replacement: &Term) -> Term {
        match self {
            Term::Var(v) if v == var => replacement.clone(),
            Term::Var(v) => Term::Var(v.clone()),
            Term::Abs(x, body) if x != var => {
                Term::Abs(x.clone(), Box::new(body.substitute(var, replacement)))
            }
            Term::App(f, a) => Term::App(
                Box::new(f.substitute(var, replacement)),
                Box::new(a.substitute(var, replacement))
            ),
            // ... all other cases
            _ => self.clone()
        }
    }
    
    /// Normalize to normal form
    pub fn normalize(&self) -> Term {
        let mut current = self.clone();
        while let Some(next) = current.reduce_step() {
            current = next;
        }
        // Also reduce under binders
        current.normalize_under_binders()
    }
    
    fn normalize_under_binders(&self) -> Term {
        todo!()
    }
}
```

---

## Parser

### Grammar (Pest)

```pest
// llw-parse/src/grammar.pest

WHITESPACE = _{ " " | "\t" | "\n" | "\r" }
COMMENT = _{ "//" ~ (!"\n" ~ ANY)* }

// Identifiers
ident = @{ ASCII_ALPHA ~ (ASCII_ALPHANUMERIC | "_")* }

// Formulas (precedence from low to high)
formula = { lolli_formula }

lolli_formula = { par_formula ~ ("-o" ~ par_formula)* }

par_formula = { tensor_formula ~ ("⅋" | "|" | "par")? ~ tensor_formula? }

tensor_formula = { plus_formula ~ (("⊗" | "*" | "⊗") ~ plus_formula)* }

plus_formula = { with_formula ~ (("⊕" | "+" ) ~ with_formula)* }

with_formula = { unary_formula ~ (("&" | "with") ~ unary_formula)* }

unary_formula = {
    "!" ~ unary_formula |
    "?" ~ unary_formula |
    primary_formula ~ "^"? // optional negation suffix
}

primary_formula = {
    "(" ~ formula ~ ")" |
    "1" | "one" |
    "⊥" | "bot" | "bottom" |
    "⊤" | "top" |
    "0" | "zero" |
    ident
}

// Sequents
sequent = {
    formula_list? ~ ("⊢" | "|-" | "=>") ~ formula_list? |
    "⊢" ~ formula_list
}

formula_list = { formula ~ ("," ~ formula)* }

// Top-level
file = { SOI ~ (declaration | sequent)* ~ EOI }

declaration = {
    "atom" ~ ident |
    "def" ~ ident ~ "=" ~ formula
}
```

### Parser Implementation

```rust
// llw-parse/src/parser.rs

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct LLParser;

pub fn parse_formula(input: &str) -> Result<Formula, ParseError> {
    let pairs = LLParser::parse(Rule::formula, input)?;
    build_formula(pairs.into_iter().next().unwrap())
}

pub fn parse_sequent(input: &str) -> Result<TwoSidedSequent, ParseError> {
    let pairs = LLParser::parse(Rule::sequent, input)?;
    build_sequent(pairs.into_iter().next().unwrap())
}

fn build_formula(pair: Pair<Rule>) -> Result<Formula, ParseError> {
    match pair.as_rule() {
        Rule::ident => Ok(Formula::Atom(pair.as_str().to_string())),
        
        Rule::lolli_formula => {
            let mut inner: Vec<_> = pair.into_inner().collect();
            // Right-associative: A -o B -o C = A -o (B -o C)
            let mut result = build_formula(inner.pop().unwrap())?;
            while let Some(left) = inner.pop() {
                let left = build_formula(left)?;
                result = Formula::Lolli(Box::new(left), Box::new(result));
            }
            Ok(result)
        }
        
        Rule::tensor_formula => {
            let mut inner = pair.into_inner();
            let first = build_formula(inner.next().unwrap())?;
            inner.try_fold(first, |acc, p| {
                let right = build_formula(p)?;
                Ok(Formula::Tensor(Box::new(acc), Box::new(right)))
            })
        }
        
        Rule::par_formula => {
            let mut inner = pair.into_inner();
            let first = build_formula(inner.next().unwrap())?;
            inner.try_fold(first, |acc, p| {
                let right = build_formula(p)?;
                Ok(Formula::Par(Box::new(acc), Box::new(right)))
            })
        }
        
        Rule::unary_formula => {
            let mut inner = pair.into_inner();
            let first = inner.next().unwrap();
            
            match first.as_rule() {
                Rule::primary_formula => build_formula(first),
                _ => {
                    // It's a unary operator
                    let op = first.as_str();
                    let operand = build_formula(inner.next().unwrap())?;
                    match op {
                        "!" => Ok(Formula::OfCourse(Box::new(operand))),
                        "?" => Ok(Formula::WhyNot(Box::new(operand))),
                        _ => Err(ParseError::UnknownOperator(op.to_string())),
                    }
                }
            }
        }
        
        Rule::primary_formula => {
            let inner = pair.into_inner().next().unwrap();
            match inner.as_rule() {
                Rule::formula => build_formula(inner),
                Rule::ident => Ok(Formula::Atom(inner.as_str().to_string())),
                _ => {
                    match inner.as_str() {
                        "1" | "one" => Ok(Formula::One),
                        "⊥" | "bot" | "bottom" => Ok(Formula::Bottom),
                        "⊤" | "top" => Ok(Formula::Top),
                        "0" | "zero" => Ok(Formula::Zero),
                        s => Err(ParseError::UnexpectedToken(s.to_string())),
                    }
                }
            }
        }
        
        _ => Err(ParseError::UnexpectedRule(format!("{:?}", pair.as_rule()))),
    }
}

// Pretty printing
impl Formula {
    pub fn pretty(&self) -> String {
        match self {
            Formula::Atom(a) => a.clone(),
            Formula::NegAtom(a) => format!("{}⊥", a),
            Formula::Tensor(a, b) => format!("({} ⊗ {})", a.pretty(), b.pretty()),
            Formula::Par(a, b) => format!("({} ⅋ {})", a.pretty(), b.pretty()),
            Formula::Lolli(a, b) => format!("({} ⊸ {})", a.pretty(), b.pretty()),
            Formula::With(a, b) => format!("({} & {})", a.pretty(), b.pretty()),
            Formula::Plus(a, b) => format!("({} ⊕ {})", a.pretty(), b.pretty()),
            Formula::OfCourse(a) => format!("!{}", a.pretty()),
            Formula::WhyNot(a) => format!("?{}", a.pretty()),
            Formula::One => "1".to_string(),
            Formula::Bottom => "⊥".to_string(),
            Formula::Top => "⊤".to_string(),
            Formula::Zero => "0".to_string(),
        }
    }
}
```

---

## Proof Search

### Focused Sequent Calculus

Standard sequent calculus has too much non-determinism. **Focused** proof search (Andreoli, 1992) reduces this:

- **Positive formulas** (async): ⊗, 1, ⊕, 0, !, atoms
- **Negative formulas** (sync): ⅋, ⊥, &, ⊤, ?, negated atoms

Rules:
1. Apply all invertible (negative) rules eagerly
2. Then choose a positive formula to **focus** on
3. Decompose the focused formula completely (no choice)
4. Return to step 1

This dramatically prunes the search space.

```rust
// llw-prove/src/focused.rs

use crate::core::{Formula, Sequent, Proof, Rule};

pub struct FocusedProver {
    /// Maximum search depth
    pub max_depth: usize,
    
    /// Enable caching of failed branches
    pub use_cache: bool,
    
    /// Cache of unprovable sequents
    cache: HashSet<Sequent>,
}

impl FocusedProver {
    pub fn prove(&mut self, sequent: &Sequent) -> Option<Proof> {
        self.prove_async(sequent, 0)
    }
    
    /// Asynchronous phase: apply all invertible rules
    fn prove_async(&mut self, sequent: &Sequent, depth: usize) -> Option<Proof> {
        if depth > self.max_depth {
            return None;
        }
        
        // Check cache
        if self.use_cache && self.cache.contains(sequent) {
            return None;
        }
        
        // Look for invertible rules to apply
        for (i, formula) in sequent.linear.iter().enumerate() {
            if formula.is_negative() {
                return self.apply_negative_rule(sequent, i, depth);
            }
        }
        
        // No negative formulas - enter synchronous phase
        self.prove_sync(sequent, depth)
    }
    
    /// Apply a negative (invertible) rule
    fn apply_negative_rule(&mut self, sequent: &Sequent, idx: usize, depth: usize) -> Option<Proof> {
        let formula = &sequent.linear[idx];
        
        match formula {
            Formula::Par(a, b) => {
                // ⅋ is invertible: decompose into A, B
                let mut new_linear = sequent.linear.clone();
                new_linear.remove(idx);
                new_linear.push(a.as_ref().clone());
                new_linear.push(b.as_ref().clone());
                
                let premise_seq = Sequent {
                    linear: new_linear,
                    unrestricted: sequent.unrestricted.clone(),
                    focus: None,
                };
                
                let premise = self.prove_async(&premise_seq, depth + 1)?;
                
                Some(Proof {
                    conclusion: sequent.clone(),
                    rule: Rule::ParIntro,
                    premises: vec![premise],
                })
            }
            
            Formula::Bottom => {
                // ⊥ is invertible: just remove it
                let mut new_linear = sequent.linear.clone();
                new_linear.remove(idx);
                
                let premise_seq = Sequent {
                    linear: new_linear,
                    unrestricted: sequent.unrestricted.clone(),
                    focus: None,
                };
                
                let premise = self.prove_async(&premise_seq, depth + 1)?;
                
                Some(Proof {
                    conclusion: sequent.clone(),
                    rule: Rule::BottomIntro,
                    premises: vec![premise],
                })
            }
            
            Formula::With(a, b) => {
                // & requires proving both branches with same context
                let mut new_linear = sequent.linear.clone();
                new_linear.remove(idx);
                
                let left_seq = Sequent {
                    linear: {
                        let mut l = new_linear.clone();
                        l.push(a.as_ref().clone());
                        l
                    },
                    unrestricted: sequent.unrestricted.clone(),
                    focus: None,
                };
                
                let right_seq = Sequent {
                    linear: {
                        let mut l = new_linear;
                        l.push(b.as_ref().clone());
                        l
                    },
                    unrestricted: sequent.unrestricted.clone(),
                    focus: None,
                };
                
                let left_premise = self.prove_async(&left_seq, depth + 1)?;
                let right_premise = self.prove_async(&right_seq, depth + 1)?;
                
                Some(Proof {
                    conclusion: sequent.clone(),
                    rule: Rule::WithIntro,
                    premises: vec![left_premise, right_premise],
                })
            }
            
            Formula::Top => {
                // ⊤ is always provable
                Some(Proof {
                    conclusion: sequent.clone(),
                    rule: Rule::TopIntro,
                    premises: vec![],
                })
            }
            
            Formula::WhyNot(a) => {
                // ?A: move to unrestricted zone
                let mut new_linear = sequent.linear.clone();
                new_linear.remove(idx);
                
                let mut new_unrestricted = sequent.unrestricted.clone();
                new_unrestricted.push(a.as_ref().clone());
                
                let premise_seq = Sequent {
                    linear: new_linear,
                    unrestricted: new_unrestricted,
                    focus: None,
                };
                
                let premise = self.prove_async(&premise_seq, depth + 1)?;
                
                Some(Proof {
                    conclusion: sequent.clone(),
                    rule: Rule::WhyNotIntro,
                    premises: vec![premise],
                })
            }
            
            _ => None, // Not a negative formula
        }
    }
    
    /// Synchronous phase: choose a formula to focus on
    fn prove_sync(&mut self, sequent: &Sequent, depth: usize) -> Option<Proof> {
        // Base case: empty sequent (shouldn't happen in valid proof search)
        if sequent.linear.is_empty() && sequent.unrestricted.is_empty() {
            return None;
        }
        
        // Try focusing on each positive formula
        for i in 0..sequent.linear.len() {
            if sequent.linear[i].is_positive() {
                if let Some(proof) = self.prove_focused(sequent, i, depth) {
                    return Some(proof);
                }
            }
        }
        
        // Try using unrestricted formulas
        for i in 0..sequent.unrestricted.len() {
            // Copy into linear zone and focus
            let mut new_linear = sequent.linear.clone();
            new_linear.push(sequent.unrestricted[i].clone());
            
            let new_seq = Sequent {
                linear: new_linear,
                unrestricted: sequent.unrestricted.clone(),
                focus: None,
            };
            
            if let Some(proof) = self.prove_async(&new_seq, depth + 1) {
                return Some(Proof {
                    conclusion: sequent.clone(),
                    rule: Rule::Dereliction,
                    premises: vec![proof],
                });
            }
        }
        
        // Cache failure
        if self.use_cache {
            self.cache.insert(sequent.clone());
        }
        
        None
    }
    
    /// Focused phase: decompose positive formula
    fn prove_focused(&mut self, sequent: &Sequent, idx: usize, depth: usize) -> Option<Proof> {
        let formula = sequent.linear[idx].clone();
        let mut remaining = sequent.linear.clone();
        remaining.remove(idx);
        
        match formula {
            Formula::Atom(a) => {
                // Axiom: look for matching negative atom
                for (j, f) in remaining.iter().enumerate() {
                    if let Formula::NegAtom(b) = f {
                        if a == *b && remaining.len() == 1 {
                            return Some(Proof {
                                conclusion: sequent.clone(),
                                rule: Rule::Axiom,
                                premises: vec![],
                            });
                        }
                    }
                }
                None
            }
            
            Formula::One => {
                // 1 requires empty context
                if remaining.is_empty() && sequent.unrestricted.is_empty() {
                    Some(Proof {
                        conclusion: sequent.clone(),
                        rule: Rule::OneIntro,
                        premises: vec![],
                    })
                } else {
                    None
                }
            }
            
            Formula::Tensor(a, b) => {
                // ⊗ requires splitting the context
                // Try all possible splits
                for split in all_splits(&remaining) {
                    let (left_ctx, right_ctx) = split;
                    
                    let left_seq = Sequent {
                        linear: {
                            let mut l = left_ctx;
                            l.push(a.as_ref().clone());
                            l
                        },
                        unrestricted: sequent.unrestricted.clone(),
                        focus: None,
                    };
                    
                    let right_seq = Sequent {
                        linear: {
                            let mut r = right_ctx;
                            r.push(b.as_ref().clone());
                            r
                        },
                        unrestricted: sequent.unrestricted.clone(),
                        focus: None,
                    };
                    
                    if let (Some(left_proof), Some(right_proof)) = (
                        self.prove_async(&left_seq, depth + 1),
                        self.prove_async(&right_seq, depth + 1)
                    ) {
                        return Some(Proof {
                            conclusion: sequent.clone(),
                            rule: Rule::TensorIntro,
                            premises: vec![left_proof, right_proof],
                        });
                    }
                }
                None
            }
            
            Formula::Plus(a, b) => {
                // ⊕ requires choosing a side
                // Try left first
                let left_seq = Sequent {
                    linear: {
                        let mut l = remaining.clone();
                        l.push(a.as_ref().clone());
                        l
                    },
                    unrestricted: sequent.unrestricted.clone(),
                    focus: None,
                };
                
                if let Some(proof) = self.prove_async(&left_seq, depth + 1) {
                    return Some(Proof {
                        conclusion: sequent.clone(),
                        rule: Rule::PlusIntroLeft,
                        premises: vec![proof],
                    });
                }
                
                // Try right
                let right_seq = Sequent {
                    linear: {
                        let mut r = remaining;
                        r.push(b.as_ref().clone());
                        r
                    },
                    unrestricted: sequent.unrestricted.clone(),
                    focus: None,
                };
                
                if let Some(proof) = self.prove_async(&right_seq, depth + 1) {
                    return Some(Proof {
                        conclusion: sequent.clone(),
                        rule: Rule::PlusIntroRight,
                        premises: vec![proof],
                    });
                }
                
                None
            }
            
            Formula::OfCourse(a) => {
                // !A requires all context to be unrestricted
                if remaining.iter().all(|f| matches!(f, Formula::WhyNot(_))) {
                    let premise_seq = Sequent {
                        linear: vec![a.as_ref().clone()],
                        unrestricted: sequent.unrestricted.clone(),
                        focus: None,
                    };
                    
                    if let Some(proof) = self.prove_async(&premise_seq, depth + 1) {
                        return Some(Proof {
                            conclusion: sequent.clone(),
                            rule: Rule::OfCourseIntro,
                            premises: vec![proof],
                        });
                    }
                }
                None
            }
            
            _ => None,
        }
    }
}

/// Generate all ways to split a multiset into two parts
fn all_splits<T: Clone>(items: &[T]) -> Vec<(Vec<T>, Vec<T>)> {
    if items.is_empty() {
        return vec![(vec![], vec![])];
    }
    
    let first = items[0].clone();
    let rest_splits = all_splits(&items[1..]);
    
    let mut result = vec![];
    for (left, right) in rest_splits {
        // Put first in left
        let mut new_left = vec![first.clone()];
        new_left.extend(left.clone());
        result.push((new_left, right.clone()));
        
        // Put first in right
        let mut new_right = vec![first.clone()];
        new_right.extend(right);
        result.push((left, new_right));
    }
    
    result
}
```

---

## Term Extraction

The Curry-Howard correspondence gives us programs from proofs:

```rust
// llw-extract/src/extract.rs

use crate::core::{Proof, Rule, Term, Formula};

pub struct Extractor {
    /// Counter for fresh variable names
    var_counter: usize,
}

impl Extractor {
    pub fn new() -> Self {
        Self { var_counter: 0 }
    }
    
    fn fresh_var(&mut self) -> String {
        let v = format!("x{}", self.var_counter);
        self.var_counter += 1;
        v
    }
    
    /// Extract a term from a proof
    /// 
    /// The proof must be cut-free for this to work directly.
    pub fn extract(&mut self, proof: &Proof) -> Term {
        match &proof.rule {
            Rule::Axiom => {
                // Identity: x : A ⊢ x : A
                // Find the variable
                let var = self.fresh_var();
                Term::Var(var)
            }
            
            Rule::OneIntro => Term::Unit,
            
            Rule::TensorIntro => {
                // From proofs of Γ ⊢ A and Δ ⊢ B, build (a, b)
                let left_term = self.extract(&proof.premises[0]);
                let right_term = self.extract(&proof.premises[1]);
                Term::Pair(Box::new(left_term), Box::new(right_term))
            }
            
            Rule::ParIntro => {
                // ⅋ introduction: from Γ, A, B ⊢, build λx.λy.e
                let var_a = self.fresh_var();
                let var_b = self.fresh_var();
                let body = self.extract(&proof.premises[0]);
                Term::Abs(var_a, Box::new(Term::Abs(var_b, Box::new(body))))
            }
            
            Rule::WithIntro => {
                // & introduction: from Γ ⊢ A and Γ ⊢ B, build ⟨a, b⟩
                // (This is a lazy pair - both components computed on demand)
                let left_term = self.extract(&proof.premises[0]);
                let right_term = self.extract(&proof.premises[1]);
                // Represent as a special pair that's projected lazily
                Term::Pair(Box::new(left_term), Box::new(right_term))
            }
            
            Rule::PlusIntroLeft => {
                let inner = self.extract(&proof.premises[0]);
                Term::Inl(Box::new(inner))
            }
            
            Rule::PlusIntroRight => {
                let inner = self.extract(&proof.premises[0]);
                Term::Inr(Box::new(inner))
            }
            
            Rule::TopIntro => Term::Trivial,
            
            Rule::OfCourseIntro => {
                let inner = self.extract(&proof.premises[0]);
                Term::Promote(Box::new(inner))
            }
            
            Rule::WhyNotIntro => {
                // ?A is about demand - extract the underlying term
                self.extract(&proof.premises[0])
            }
            
            Rule::Weakening => {
                // Discard the ?A resource
                let var = self.fresh_var();
                let body = self.extract(&proof.premises[0]);
                Term::Discard(Box::new(Term::Var(var)), Box::new(body))
            }
            
            Rule::Contraction => {
                // Copy the ?A resource
                let var = self.fresh_var();
                let copy1 = self.fresh_var();
                let copy2 = self.fresh_var();
                let body = self.extract(&proof.premises[0]);
                Term::Copy(
                    Box::new(Term::Var(var)),
                    copy1,
                    copy2,
                    Box::new(body)
                )
            }
            
            Rule::Cut(formula) => {
                // Cut should be eliminated before extraction
                // But if present: let x = e1 in e2
                let left = self.extract(&proof.premises[0]);
                let right = self.extract(&proof.premises[1]);
                let var = self.fresh_var();
                Term::App(
                    Box::new(Term::Abs(var, Box::new(right))),
                    Box::new(left)
                )
            }
            
            _ => {
                // Focused rules are administrative
                self.extract(&proof.premises[0])
            }
        }
    }
}
```

---

## Rust Code Generation

```rust
// llw-codegen/src/rust.rs

use crate::core::{Formula, Term};

pub struct RustCodegen {
    /// Indentation level
    indent: usize,
}

impl RustCodegen {
    pub fn new() -> Self {
        Self { indent: 0 }
    }
    
    /// Generate Rust type from linear logic formula
    pub fn formula_to_type(&self, formula: &Formula) -> String {
        match formula {
            Formula::Atom(a) => a.clone(),
            Formula::NegAtom(a) => format!("{}Dual", a),
            
            Formula::Tensor(a, b) => {
                format!("({}, {})", 
                    self.formula_to_type(a),
                    self.formula_to_type(b)
                )
            }
            
            Formula::Par(a, b) => {
                // A ⅋ B ≈ A⊥ ⊸ B ≈ fn(A) -> B in negative position
                // This is tricky - par is "or" from opponent's view
                format!("Par<{}, {}>",
                    self.formula_to_type(a),
                    self.formula_to_type(b)
                )
            }
            
            Formula::Lolli(a, b) => {
                // A ⊸ B is a consuming function
                format!("impl FnOnce({}) -> {}",
                    self.formula_to_type(a),
                    self.formula_to_type(b)
                )
            }
            
            Formula::With(a, b) => {
                // A & B: both available, choose one
                // Could be enum or lazy pair
                format!("With<{}, {}>",
                    self.formula_to_type(a),
                    self.formula_to_type(b)
                )
            }
            
            Formula::Plus(a, b) => {
                // A ⊕ B: sum type
                format!("Either<{}, {}>",
                    self.formula_to_type(a),
                    self.formula_to_type(b)
                )
            }
            
            Formula::OfCourse(a) => {
                // !A: can copy
                // Use Rc or Clone
                format!("Rc<{}>", self.formula_to_type(a))
            }
            
            Formula::WhyNot(a) => {
                // ?A: demand/need for A
                format!("Demand<{}>", self.formula_to_type(a))
            }
            
            Formula::One => "()".to_string(),
            Formula::Bottom => "!".to_string(), // never type
            Formula::Top => "Top".to_string(),  // trivially satisfiable
            Formula::Zero => "Void".to_string(), // empty type
        }
    }
    
    /// Generate Rust code from extracted term
    pub fn term_to_code(&mut self, term: &Term) -> String {
        match term {
            Term::Var(v) => v.clone(),
            
            Term::Unit => "()".to_string(),
            
            Term::Pair(a, b) => {
                format!("({}, {})",
                    self.term_to_code(a),
                    self.term_to_code(b)
                )
            }
            
            Term::LetPair(x, y, pair, body) => {
                format!("{{\n{}let ({}, {}) = {};\n{}{}\n{}}}",
                    self.indent_str(),
                    x, y,
                    self.term_to_code(pair),
                    self.indent_str(),
                    self.term_to_code(body),
                    self.indent_str()
                )
            }
            
            Term::Abs(x, body) => {
                self.indent += 1;
                let body_code = self.term_to_code(body);
                self.indent -= 1;
                format!("|{}| {}", x, body_code)
            }
            
            Term::App(f, a) => {
                format!("{}({})",
                    self.term_to_code(f),
                    self.term_to_code(a)
                )
            }
            
            Term::Inl(a) => {
                format!("Either::Left({})", self.term_to_code(a))
            }
            
            Term::Inr(b) => {
                format!("Either::Right({})", self.term_to_code(b))
            }
            
            Term::Case(scrutinee, x, left, y, right) => {
                self.indent += 1;
                let left_code = self.term_to_code(left);
                let right_code = self.term_to_code(right);
                self.indent -= 1;
                
                format!(
                    "match {} {{\n{}Either::Left({}) => {},\n{}Either::Right({}) => {},\n{}}}",
                    self.term_to_code(scrutinee),
                    self.indent_str(), x, left_code,
                    self.indent_str(), y, right_code,
                    self.indent_str()
                )
            }
            
            Term::Trivial => "Top::new()".to_string(),
            
            Term::Fst(p) => format!("{}.0", self.term_to_code(p)),
            
            Term::Snd(p) => format!("{}.1", self.term_to_code(p)),
            
            Term::Abort(e) => format!("match {} {{}}", self.term_to_code(e)),
            
            Term::Promote(a) => {
                format!("Rc::new({})", self.term_to_code(a))
            }
            
            Term::Derelict(a) => {
                format!("(*{})", self.term_to_code(a))
            }
            
            Term::Discard(_, body) => {
                format!("{{ drop(_); {} }}", self.term_to_code(body))
            }
            
            Term::Copy(src, x, y, body) => {
                format!("{{\n{}let {} = {}.clone();\n{}let {} = {};\n{}{}\n{}}}",
                    self.indent_str(),
                    x, self.term_to_code(src),
                    self.indent_str(),
                    y, self.term_to_code(src),
                    self.indent_str(),
                    self.term_to_code(body),
                    self.indent_str()
                )
            }
        }
    }
    
    fn indent_str(&self) -> String {
        "    ".repeat(self.indent)
    }
    
    /// Generate a complete Rust module from a set of proofs
    pub fn generate_module(&mut self, proofs: &[(String, Proof)]) -> String {
        let mut code = String::new();
        
        code.push_str("// Auto-generated from linear logic proofs\n\n");
        code.push_str("use std::rc::Rc;\n\n");
        
        // Helper types
        code.push_str(HELPER_TYPES);
        
        // Generate each function
        for (name, proof) in proofs {
            let formula = &proof.conclusion; // Get the proven formula
            let mut extractor = Extractor::new();
            let term = extractor.extract(proof);
            let rust_code = self.term_to_code(&term);
            
            code.push_str(&format!("\npub fn {}() {{\n", name));
            code.push_str(&format!("    {}\n", rust_code));
            code.push_str("}\n");
        }
        
        code
    }
}

const HELPER_TYPES: &str = r#"
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

pub struct With<A, B> {
    left: Box<dyn FnOnce() -> A>,
    right: Box<dyn FnOnce() -> B>,
}

impl<A, B> With<A, B> {
    pub fn left(self) -> A { (self.left)() }
    pub fn right(self) -> B { (self.right)() }
}

pub struct Top;
impl Top {
    pub fn new() -> Self { Top }
}

pub enum Void {}

pub struct Par<A, B>(std::marker::PhantomData<(A, B)>);

pub struct Demand<A>(Box<dyn FnOnce() -> A>);
"#;
```

---

## Visualization

### Proof Tree Rendering

```rust
// llw-viz/src/tree.rs

use crate::core::Proof;

pub struct TreeRenderer {
    /// Use Unicode box-drawing characters
    pub unicode: bool,
}

impl TreeRenderer {
    /// Render proof as ASCII/Unicode tree
    pub fn render(&self, proof: &Proof) -> String {
        self.render_node(proof, 0)
    }
    
    fn render_node(&self, proof: &Proof, depth: usize) -> String {
        let indent = "  ".repeat(depth);
        let mut result = String::new();
        
        // Render premises first (above the line)
        for premise in &proof.premises {
            result.push_str(&self.render_node(premise, depth + 1));
        }
        
        // Render the inference line
        let conclusion = proof.conclusion.pretty();
        let rule_name = format!("{:?}", proof.rule);
        
        let line_char = if self.unicode { "─" } else { "-" };
        let line = line_char.repeat(conclusion.len().max(20));
        
        if !proof.premises.is_empty() {
            result.push_str(&format!("{}{} ({})\n", indent, line, rule_name));
        }
        result.push_str(&format!("{}{}\n", indent, conclusion));
        
        result
    }
    
    /// Render proof as LaTeX
    pub fn render_latex(&self, proof: &Proof) -> String {
        let mut result = String::from("\\begin{prooftree}\n");
        result.push_str(&self.render_latex_node(proof));
        result.push_str("\\end{prooftree}\n");
        result
    }
    
    fn render_latex_node(&self, proof: &Proof) -> String {
        let mut result = String::new();
        
        // Render premises
        for premise in &proof.premises {
            result.push_str(&self.render_latex_node(premise));
        }
        
        // Render inference
        let n_premises = proof.premises.len();
        let rule_name = format!("{:?}", proof.rule);
        let conclusion = proof.conclusion.pretty_latex();
        
        let infer_cmd = match n_premises {
            0 => "\\AxiomC",
            1 => "\\UnaryInfC",
            2 => "\\BinaryInfC",
            3 => "\\TrinaryInfC",
            _ => "\\QuaternaryInfC",
        };
        
        result.push_str(&format!("\\RightLabel{{\\scriptsize {}}}\n", rule_name));
        result.push_str(&format!("{}{{${}$}}\n", infer_cmd, conclusion));
        
        result
    }
}
```

### Proof Net Rendering

```rust
// llw-viz/src/net.rs

use crate::core::ProofNet;

pub struct NetRenderer;

impl NetRenderer {
    /// Render proof net as Graphviz DOT
    pub fn render_dot(&self, net: &ProofNet) -> String {
        let mut dot = String::from("digraph proof_net {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  node [shape=circle];\n");
        
        // Render nodes
        for (idx, node) in net.graph.node_indices().enumerate() {
            let weight = &net.graph[node];
            let label = match weight {
                NetNode::Axiom => "ax",
                NetNode::Tensor => "⊗",
                NetNode::Par => "⅋",
                NetNode::One => "1",
                NetNode::Bottom => "⊥",
                // ... etc
            };
            dot.push_str(&format!("  n{} [label=\"{}\"];\n", idx, label));
        }
        
        // Render edges
        for edge in net.graph.edge_indices() {
            let (src, tgt) = net.graph.edge_endpoints(edge).unwrap();
            let weight = &net.graph[edge];
            dot.push_str(&format!(
                "  n{} -> n{} [label=\"{}\"];\n",
                src.index(), tgt.index(), weight.formula.pretty()
            ));
        }
        
        dot.push_str("}\n");
        dot
    }
    
    /// Render as SVG (via graphviz or custom)
    pub fn render_svg(&self, net: &ProofNet) -> String {
        // Could call graphviz or generate SVG directly
        todo!()
    }
}
```

---

## CLI Interface

```rust
// llw-cli/src/main.rs

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "llw")]
#[command(about = "Linear Logic Workbench")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse and pretty-print a formula
    Parse {
        /// Formula to parse
        formula: String,
    },
    
    /// Check if a sequent is provable
    Prove {
        /// Sequent to prove (e.g., "A, B |- A ⊗ B")
        sequent: String,
        
        /// Maximum search depth
        #[arg(short, long, default_value = "100")]
        depth: usize,
        
        /// Output format: tree, latex, dot
        #[arg(short, long, default_value = "tree")]
        format: String,
    },
    
    /// Extract a term from a proof
    Extract {
        /// Sequent to prove
        sequent: String,
        
        /// Normalize the extracted term
        #[arg(short, long)]
        normalize: bool,
    },
    
    /// Generate Rust code from a proof
    Codegen {
        /// Sequent to prove
        sequent: String,
        
        /// Output file
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Visualize a proof
    Viz {
        /// Sequent to prove
        sequent: String,
        
        /// Output format: tree, latex, dot, svg
        #[arg(short, long, default_value = "tree")]
        format: String,
        
        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Run REPL
    Repl,
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Parse { formula } => {
            match parse_formula(&formula) {
                Ok(f) => {
                    println!("Parsed: {}", f.pretty());
                    println!("Desugared: {}", f.desugar().pretty());
                    println!("Negation: {}", f.negate().pretty());
                }
                Err(e) => eprintln!("Parse error: {}", e),
            }
        }
        
        Commands::Prove { sequent, depth, format } => {
            match parse_sequent(&sequent) {
                Ok(seq) => {
                    let mut prover = FocusedProver::new(depth);
                    match prover.prove(&seq.to_one_sided()) {
                        Some(proof) => {
                            println!("Provable!");
                            match format.as_str() {
                                "tree" => println!("{}", TreeRenderer::new().render(&proof)),
                                "latex" => println!("{}", TreeRenderer::new().render_latex(&proof)),
                                "dot" => {
                                    let net = ProofNet::from_proof(&proof);
                                    println!("{}", NetRenderer.render_dot(&net));
                                }
                                _ => eprintln!("Unknown format: {}", format),
                            }
                        }
                        None => println!("Not provable (within depth {})", depth),
                    }
                }
                Err(e) => eprintln!("Parse error: {}", e),
            }
        }
        
        Commands::Extract { sequent, normalize } => {
            match parse_sequent(&sequent) {
                Ok(seq) => {
                    let mut prover = FocusedProver::new(100);
                    match prover.prove(&seq.to_one_sided()) {
                        Some(proof) => {
                            let mut extractor = Extractor::new();
                            let mut term = extractor.extract(&proof);
                            if normalize {
                                term = term.normalize();
                            }
                            println!("{}", term.pretty());
                        }
                        None => println!("Not provable"),
                    }
                }
                Err(e) => eprintln!("Parse error: {}", e),
            }
        }
        
        Commands::Codegen { sequent, output } => {
            match parse_sequent(&sequent) {
                Ok(seq) => {
                    let mut prover = FocusedProver::new(100);
                    match prover.prove(&seq.to_one_sided()) {
                        Some(proof) => {
                            let mut extractor = Extractor::new();
                            let term = extractor.extract(&proof);
                            let mut codegen = RustCodegen::new();
                            let code = codegen.term_to_code(&term);
                            
                            match output {
                                Some(path) => {
                                    std::fs::write(&path, &code).unwrap();
                                    println!("Written to {}", path);
                                }
                                None => println!("{}", code),
                            }
                        }
                        None => println!("Not provable"),
                    }
                }
                Err(e) => eprintln!("Parse error: {}", e),
            }
        }
        
        Commands::Viz { sequent, format, output } => {
            // Similar to Prove but with more output options
            todo!()
        }
        
        Commands::Repl => {
            // Interactive mode
            run_repl();
        }
    }
}

fn run_repl() {
    use std::io::{self, Write};
    
    println!("Linear Logic Workbench REPL");
    println!("Commands: :prove <sequent>, :parse <formula>, :quit");
    println!();
    
    loop {
        print!("llw> ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        if input == ":quit" || input == ":q" {
            break;
        }
        
        if let Some(sequent) = input.strip_prefix(":prove ") {
            // Prove the sequent
            match parse_sequent(sequent) {
                Ok(seq) => {
                    let mut prover = FocusedProver::new(100);
                    match prover.prove(&seq.to_one_sided()) {
                        Some(proof) => {
                            println!("✓ Provable");
                            println!("{}", TreeRenderer::new().render(&proof));
                        }
                        None => println!("✗ Not provable"),
                    }
                }
                Err(e) => eprintln!("Parse error: {}", e),
            }
        } else if let Some(formula) = input.strip_prefix(":parse ") {
            match parse_formula(formula) {
                Ok(f) => println!("{}", f.pretty()),
                Err(e) => eprintln!("Parse error: {}", e),
            }
        } else {
            // Default: try to prove as sequent
            match parse_sequent(input) {
                Ok(seq) => {
                    let mut prover = FocusedProver::new(100);
                    match prover.prove(&seq.to_one_sided()) {
                        Some(_) => println!("✓ Provable"),
                        None => println!("✗ Not provable"),
                    }
                }
                Err(_) => println!("Unknown command. Try :prove, :parse, or :quit"),
            }
        }
    }
}
```

---

## Development Roadmap

### Phase 1: Core & Parser (3-4 weeks)

- [ ] Formula data structures
- [ ] Sequent representation
- [ ] Pest grammar for MALL
- [ ] Parser implementation
- [ ] Pretty printing
- [ ] Basic tests

**Deliverable**: Can parse and print linear logic formulas

### Phase 2: Proof Search (4-5 weeks)

- [ ] Proof tree representation
- [ ] Focused sequent calculus
- [ ] MALL proof search
- [ ] Caching and optimizations
- [ ] Correctness tests against known theorems

**Deliverable**: Can prove MALL sequents

### Phase 3: Term Extraction (2-3 weeks)

- [ ] Term representation
- [ ] Proof-to-term extraction
- [ ] β-reduction
- [ ] Normalization
- [ ] Tests

**Deliverable**: Can extract λ-terms from proofs

### Phase 4: Rust Codegen (3-4 weeks)

- [ ] Type generation
- [ ] Term-to-Rust translation
- [ ] Helper types (Either, With, etc.)
- [ ] Generated code compiles
- [ ] Integration tests

**Deliverable**: Can generate working Rust from proofs

### Phase 5: Visualization & CLI (2-3 weeks)

- [ ] ASCII proof trees
- [ ] LaTeX export
- [ ] Graphviz export
- [ ] CLI with all commands
- [ ] REPL

**Deliverable**: Complete usable tool

### Phase 6: Exponentials (3-4 weeks)

- [ ] Extend parser for !, ?
- [ ] Proof search with exponentials (MELL)
- [ ] Term extraction with promotion/dereliction
- [ ] Codegen with Rc

**Deliverable**: Full linear logic support

### Future

- Proof nets (MLL)
- Interactive proof construction
- VS Code extension
- TypeScript codegen
- Protocol specifications DSL

---

## Examples

### Example 1: Currying

```
$ llw prove "A ⊗ B ⊸ C |- A ⊸ B ⊸ C"
✓ Provable

        ─────────────────── (ax)
        ⊢ A⊥, A
                    ─────────────────── (ax)
                    ⊢ B⊥, B
                            ─────────────────── (ax)
                            ⊢ C⊥, C
                    ───────────────────────────────────── (⊗)
                    ⊢ B⊥, C⊥, B ⊗ C
        ──────────────────────────────────────────────────────── (⅋)
        ⊢ A⊥, B⊥, C⊥, A ⊗ B
```

### Example 2: File Protocol

```
// file_protocol.ll
atom FileHandle
atom Contents
atom ClosedHandle

// Reading consumes handle, produces contents and closed handle
def read = FileHandle ⊸ (Contents ⊗ ClosedHandle)
```

```
$ llw codegen "FileHandle |- Contents ⊗ ClosedHandle"

// Auto-generated from linear logic proofs

pub fn read(handle: FileHandle) -> (Contents, ClosedHandle) {
    let (contents, closed) = handle.read_and_close();
    (contents, closed)
}
```

### Example 3: Choice

```
$ llw prove "A & B |- A"
✓ Provable

        ─────────────────── (ax)
        ⊢ A⊥, A
─────────────────────────────── (&₁)
⊢ A⊥, A & B

$ llw extract "A & B |- A"
fst(x)
```

---

## References

### Foundational

- Girard, "Linear Logic" (1987) — the original paper
- Girard, "Linear Logic: Its Syntax and Semantics" (1995) — comprehensive
- Wadler, "A Taste of Linear Logic" (1993) — accessible introduction

### Proof Search

- Andreoli, "Logic Programming with Focusing Proofs" (1992) — focused sequent calculus
- Chaudhuri, "Classical and Intuitionistic Subexponential Logics" (2010)
- Liang & Miller, "Focusing and Polarization in Linear, Intuitionistic, and Classical Logics" (2009)

### Proof Nets

- Danos & Regnier, "The Structure of Multiplicatives" (1989) — MLL proof nets
- Girard, "Proof-nets: The Parallel Syntax for Proof-theory" (1996)

### Implementation

- Pfenning, "Structural Cut Elimination" (2000) — algorithms
- Cervesato & Pfenning, "A Linear Logical Framework" (2002)

### Rust Connection

- Walker, "Substructural Type Systems" (2005) — in ATTAPL
- "Rust and Linear Types" — various blog posts

---

## Design Decisions

### Why one-sided sequents?

- Simpler rule set (no left/right rules)
- Proof nets work with one-sided
- Focused calculus is cleaner

### Why MALL first?

- Decidable (exponentials make it undecidable)
- Captures most resource reasoning
- Proof search is tractable

### Why Pest for parsing?

- PEG is natural for expression grammars
- Good error messages
- Fast compilation

### Why focused proof search?

- Dramatically reduces search space
- Principled (Andreoli's theorem)
- Maps well to term extraction

### Why generate Rust specifically?

- Rust's type system already enforces affine types
- Natural fit for resource reasoning
- Practical target for real systems

---

## Open Questions

1. **How to handle polymorphism?** System F linear? Type variables?

2. **What's the right API for protocols?** Session types DSL on top?

3. **Can we do bidirectional proof search?** Forward + backward?

4. **How to integrate with async Rust?** Linear futures?

5. **Can proof nets improve codegen?** More canonical than trees.

---

## Contributing

Areas where contributions help:

1. **More connectives**: Units, quantifiers
2. **Optimization**: Caching, indexing, parallel search
3. **Proof nets**: Full implementation
4. **TypeScript target**: Alternative codegen
5. **Documentation**: Tutorials, examples

See CONTRIBUTING.md (to be written).
