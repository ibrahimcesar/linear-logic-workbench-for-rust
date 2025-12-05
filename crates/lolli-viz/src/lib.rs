//! # lolli-viz
//!
//! Visualization for the Lolli linear logic workbench.
//!
//! This crate provides rendering of proofs as trees, LaTeX, and graphs.
//!
//! ## Output Formats
//!
//! - **ASCII/Unicode**: Terminal-friendly proof trees
//! - **LaTeX**: Using bussproofs package
//! - **DOT**: Graphviz format for graph visualization
//!
//! ## Example
//!
//! ```
//! use lolli_viz::{TreeRenderer, render_ascii};
//! use lolli_core::{Formula, Proof, Rule, Sequent};
//!
//! let proof = Proof {
//!     conclusion: Sequent::new(vec![Formula::neg_atom("A"), Formula::atom("A")]),
//!     rule: Rule::Axiom,
//!     premises: vec![],
//! };
//!
//! let ascii = render_ascii(&proof);
//! println!("{}", ascii);
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub use lolli_core::{Formula, Proof, Rule, Sequent};

mod ascii;
mod latex;
mod dot;

pub use ascii::TreeRenderer;
pub use latex::LatexRenderer;
pub use dot::DotRenderer;

/// Render a proof as ASCII text.
pub fn render_ascii(proof: &Proof) -> String {
    TreeRenderer::new().render(proof)
}

/// Render a proof as Unicode text (with box-drawing characters).
pub fn render_unicode(proof: &Proof) -> String {
    let mut renderer = TreeRenderer::new();
    renderer.unicode = true;
    renderer.render(proof)
}

/// Render a proof as LaTeX (bussproofs package).
pub fn render_latex(proof: &Proof) -> String {
    LatexRenderer::new().render(proof)
}

/// Render a proof as Graphviz DOT format.
pub fn render_dot(proof: &Proof) -> String {
    DotRenderer::new().render(proof)
}
