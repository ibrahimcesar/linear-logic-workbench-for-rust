//! # lolli-prove
//!
//! Proof search for the Lolli linear logic workbench.
//!
//! This crate implements focused proof search for linear logic,
//! supporting MALL (Multiplicative-Additive) and MELL (with exponentials).
//!
//! ## Focused Proof Search
//!
//! Focused proof search (Andreoli, 1992) reduces non-determinism by:
//! 1. Classifying formulas as **positive** (async) or **negative** (sync)
//! 2. Applying invertible (negative) rules eagerly
//! 3. Choosing a positive formula to **focus** on
//! 4. Decomposing the focused formula completely
//!
//! ## Example
//!
//! ```
//! use lolli_prove::Prover;
//! use lolli_core::{Formula, TwoSidedSequent};
//!
//! let mut prover = Prover::new(100);
//!
//! // Prove A ⊢ A (identity)
//! let seq = TwoSidedSequent::new(
//!     vec![Formula::atom("A")],
//!     vec![Formula::atom("A")],
//! );
//! let result = prover.prove_two_sided(&seq);
//! assert!(result.is_some());
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

mod search;
mod verify;

pub use lolli_core::{Formula, Proof, Rule, Sequent, TwoSidedSequent};
pub use search::Prover;
pub use verify::{verify_proof, ProofError};
