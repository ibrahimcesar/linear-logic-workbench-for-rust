//! # lolli-extract
//!
//! Term extraction for the Lolli linear logic workbench.
//!
//! This crate implements the Curry-Howard correspondence, extracting
//! computational content (lambda terms) from linear logic proofs.
//!
//! ## Curry-Howard Correspondence
//!
//! The correspondence between linear logic and linear lambda calculus:
//!
//! | Linear Logic | Lambda Calculus |
//! |--------------|-----------------|
//! | A ⊸ B | A → B (linear function) |
//! | A ⊗ B | (A, B) (tensor pair) |
//! | A & B | A × B (with pair, lazy) |
//! | A ⊕ B | A + B (sum type) |
//! | 1 | () (unit) |
//! | ⊤ | ⟨⟩ (trivial) |
//! | 0 | absurd (empty) |
//! | !A | replicable A |
//!
//! ## Example
//!
//! ```
//! use lolli_extract::{Extractor, Proof, Term};
//! use lolli_core::{Formula, Rule, Sequent};
//!
//! // Build a proof of A ⊢ A (identity)
//! let proof = Proof {
//!     conclusion: Sequent::new(vec![Formula::neg_atom("A"), Formula::atom("A")]),
//!     rule: Rule::Axiom,
//!     premises: vec![],
//! };
//!
//! let mut extractor = Extractor::new();
//! let term = extractor.extract(&proof);
//! // The extracted term is λa. a (identity function)
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub use lolli_core::{Formula, Proof, Rule, Sequent, Term};

mod extract;
mod normalize;

pub use extract::Extractor;
pub use normalize::{is_normal, normalize, normalize_bounded, step};

/// Extract a term from a proof (convenience function).
///
/// # Example
///
/// ```
/// use lolli_extract::{extract_term, Proof};
/// use lolli_core::{Formula, Rule, Sequent};
///
/// let proof = Proof {
///     conclusion: Sequent::new(vec![Formula::neg_atom("A"), Formula::atom("A")]),
///     rule: Rule::Axiom,
///     premises: vec![],
/// };
///
/// let term = extract_term(&proof);
/// println!("{}", term.pretty());
/// ```
pub fn extract_term(proof: &Proof) -> Term {
    let mut extractor = Extractor::new();
    extractor.extract(proof)
}
