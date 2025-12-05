//! ASCII/Unicode proof tree rendering.
//!
//! Renders proofs as text trees suitable for terminal display.

use lolli_core::Proof;

/// Proof tree renderer for ASCII/Unicode output.
pub struct TreeRenderer {
    /// Use Unicode box-drawing characters
    pub unicode: bool,
    /// Show rule names
    pub show_rules: bool,
    /// Indent width
    pub indent_width: usize,
}

impl Default for TreeRenderer {
    fn default() -> Self {
        Self {
            unicode: true,
            show_rules: true,
            indent_width: 2,
        }
    }
}

impl TreeRenderer {
    /// Create a new renderer with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Render a proof as a text tree.
    pub fn render(&self, proof: &Proof) -> String {
        let mut lines = Vec::new();
        self.render_proof(proof, 0, &mut lines);
        lines.join("\n")
    }

    /// Render a proof recursively, building up lines.
    fn render_proof(&self, proof: &Proof, indent: usize, lines: &mut Vec<String>) {
        let prefix = " ".repeat(indent * self.indent_width);

        // Render premises first (they appear above the conclusion)
        for premise in &proof.premises {
            self.render_proof(premise, indent + 1, lines);
        }

        // Format the conclusion
        let conclusion = self.format_sequent(proof);
        let rule_name = format!("{:?}", proof.rule);

        // Draw the inference line if there are premises
        if !proof.premises.is_empty() {
            let line_char = if self.unicode { '─' } else { '-' };
            let line_width = conclusion.len().max(20);
            let line = line_char.to_string().repeat(line_width);

            if self.show_rules {
                lines.push(format!("{}{}  {}", prefix, line, rule_name));
            } else {
                lines.push(format!("{}{}", prefix, line));
            }
        } else if self.show_rules {
            // Leaf node - show rule name inline
            lines.push(format!("{}⊢ {}  ({})", prefix, conclusion, rule_name));
            return;
        }

        // Add the conclusion
        lines.push(format!("{}⊢ {}", prefix, conclusion));
    }

    /// Format a sequent for display.
    fn format_sequent(&self, proof: &Proof) -> String {
        proof
            .conclusion
            .linear
            .iter()
            .map(|f| {
                if self.unicode {
                    f.pretty()
                } else {
                    f.pretty_ascii()
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Render a proof with custom options.
pub struct TreeRenderOptions {
    /// Use Unicode characters
    pub unicode: bool,
    /// Show rule names
    pub show_rules: bool,
    /// Maximum width before wrapping
    pub max_width: Option<usize>,
    /// Color theme (for terminal output)
    pub colors: bool,
}

impl Default for TreeRenderOptions {
    fn default() -> Self {
        Self {
            unicode: true,
            show_rules: true,
            max_width: None,
            colors: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lolli_core::{Formula, Rule, Sequent};

    #[test]
    fn test_render_axiom() {
        let proof = Proof {
            conclusion: Sequent::new(vec![Formula::neg_atom("A"), Formula::atom("A")]),
            rule: Rule::Axiom,
            premises: vec![],
        };

        let renderer = TreeRenderer::new();
        let output = renderer.render(&proof);

        assert!(output.contains("A⊥"));
        assert!(output.contains("Axiom"));
    }

    #[test]
    fn test_render_tensor() {
        let left = Proof {
            conclusion: Sequent::new(vec![Formula::neg_atom("A"), Formula::atom("A")]),
            rule: Rule::Axiom,
            premises: vec![],
        };
        let right = Proof {
            conclusion: Sequent::new(vec![Formula::neg_atom("B"), Formula::atom("B")]),
            rule: Rule::Axiom,
            premises: vec![],
        };
        let proof = Proof {
            conclusion: Sequent::new(vec![
                Formula::neg_atom("A"),
                Formula::neg_atom("B"),
                Formula::tensor(Formula::atom("A"), Formula::atom("B")),
            ]),
            rule: Rule::TensorIntro,
            premises: vec![left, right],
        };

        let renderer = TreeRenderer::new();
        let output = renderer.render(&proof);

        assert!(output.contains("TensorIntro"));
        assert!(output.contains("─")); // Inference line
    }

    #[test]
    fn test_ascii_mode() {
        let proof = Proof {
            conclusion: Sequent::new(vec![Formula::neg_atom("A"), Formula::atom("A")]),
            rule: Rule::Axiom,
            premises: vec![],
        };

        let mut renderer = TreeRenderer::new();
        renderer.unicode = false;

        let output = renderer.render(&proof);
        assert!(output.contains("A^")); // ASCII negation
    }

    #[test]
    fn test_no_rules() {
        let proof = Proof {
            conclusion: Sequent::new(vec![Formula::atom("A")]),
            rule: Rule::Axiom,
            premises: vec![],
        };

        let mut renderer = TreeRenderer::new();
        renderer.show_rules = false;

        let output = renderer.render(&proof);
        assert!(!output.contains("Axiom"));
    }
}
