//! Graphviz DOT format rendering.
//!
//! Generates DOT format for visualizing proofs as graphs.

use lolli_core::Proof;

/// Graphviz DOT renderer for proofs.
pub struct DotRenderer {
    /// Graph direction (TB = top-to-bottom, BT = bottom-to-top)
    pub direction: Direction,
    /// Node shape
    pub node_shape: NodeShape,
    /// Font name
    pub font: String,
    /// Show rule names in nodes
    pub show_rules: bool,
}

/// Graph direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Top to bottom
    TopToBottom,
    /// Bottom to top (natural for proofs)
    BottomToTop,
    /// Left to right
    LeftToRight,
    /// Right to left
    RightToLeft,
}

impl Direction {
    fn as_str(&self) -> &'static str {
        match self {
            Direction::TopToBottom => "TB",
            Direction::BottomToTop => "BT",
            Direction::LeftToRight => "LR",
            Direction::RightToLeft => "RL",
        }
    }
}

/// Node shape in the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeShape {
    /// Rectangle
    Box,
    /// Rounded rectangle
    RoundedBox,
    /// Ellipse
    Ellipse,
    /// No border
    Plain,
}

impl NodeShape {
    fn as_str(&self) -> &'static str {
        match self {
            NodeShape::Box => "box",
            NodeShape::RoundedBox => "box, style=rounded",
            NodeShape::Ellipse => "ellipse",
            NodeShape::Plain => "plain",
        }
    }
}

impl Default for DotRenderer {
    fn default() -> Self {
        Self {
            direction: Direction::BottomToTop,
            node_shape: NodeShape::Box,
            font: "Helvetica".to_string(),
            show_rules: true,
        }
    }
}

impl DotRenderer {
    /// Create a new DOT renderer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Render a proof as DOT format.
    pub fn render(&self, proof: &Proof) -> String {
        let mut lines = Vec::new();
        let mut counter = 0;

        lines.push("digraph proof {".to_string());
        lines.push(format!("  rankdir={};", self.direction.as_str()));
        lines.push(format!(
            "  node [shape={}, fontname=\"{}\"];",
            self.node_shape.as_str(),
            self.font
        ));
        lines.push("  edge [arrowhead=none];".to_string());
        lines.push(String::new());

        self.render_proof(proof, &mut lines, &mut counter);

        lines.push("}".to_string());
        lines.join("\n")
    }

    /// Render a proof recursively, returning the node ID.
    fn render_proof(&self, proof: &Proof, lines: &mut Vec<String>, counter: &mut usize) -> usize {
        let my_id = *counter;
        *counter += 1;

        // Format the node label
        let conclusion = self.format_sequent(proof);
        let rule_name = format!("{:?}", proof.rule);

        let label = if self.show_rules {
            format!("⊢ {}\\n({})", conclusion, rule_name)
        } else {
            format!("⊢ {}", conclusion)
        };

        // Escape for DOT
        let label = label.replace('"', "\\\"");

        lines.push(format!("  n{} [label=\"{}\"];", my_id, label));

        // Render premises and add edges
        for premise in &proof.premises {
            let child_id = self.render_proof(premise, lines, counter);
            lines.push(format!("  n{} -> n{};", child_id, my_id));
        }

        my_id
    }

    /// Format a sequent for display.
    fn format_sequent(&self, proof: &Proof) -> String {
        proof
            .conclusion
            .linear
            .iter()
            .map(|f| f.pretty())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Render as a proof net (for multiplicative fragment).
    pub fn render_proof_net(&self, proof: &Proof) -> String {
        let mut lines = Vec::new();
        let mut counter = 0;

        lines.push("digraph proof_net {".to_string());
        lines.push("  rankdir=TB;".to_string());
        lines.push("  node [shape=circle, width=0.3];".to_string());
        lines.push("  edge [dir=none];".to_string());
        lines.push(String::new());

        self.render_net_nodes(proof, &mut lines, &mut counter);

        lines.push("}".to_string());
        lines.join("\n")
    }

    /// Render proof net nodes.
    fn render_net_nodes(
        &self,
        proof: &Proof,
        lines: &mut Vec<String>,
        counter: &mut usize,
    ) -> usize {
        let my_id = *counter;
        *counter += 1;

        let rule_name = format!("{:?}", proof.rule);
        lines.push(format!("  n{} [label=\"{}\"];", my_id, rule_name));

        for premise in &proof.premises {
            let child_id = self.render_net_nodes(premise, lines, counter);
            lines.push(format!("  n{} -> n{};", child_id, my_id));
        }

        my_id
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

        let renderer = DotRenderer::new();
        let output = renderer.render(&proof);

        assert!(output.contains("digraph proof"));
        assert!(output.contains("rankdir=BT"));
        assert!(output.contains("n0"));
    }

    #[test]
    fn test_render_edges() {
        let left = Proof {
            conclusion: Sequent::new(vec![Formula::atom("A")]),
            rule: Rule::Axiom,
            premises: vec![],
        };
        let right = Proof {
            conclusion: Sequent::new(vec![Formula::atom("B")]),
            rule: Rule::Axiom,
            premises: vec![],
        };
        let proof = Proof {
            conclusion: Sequent::new(vec![Formula::tensor(Formula::atom("A"), Formula::atom("B"))]),
            rule: Rule::TensorIntro,
            premises: vec![left, right],
        };

        let renderer = DotRenderer::new();
        let output = renderer.render(&proof);

        // Should have edges from children to parent
        assert!(output.contains("->"));
    }

    #[test]
    fn test_direction() {
        let proof = Proof {
            conclusion: Sequent::new(vec![Formula::atom("A")]),
            rule: Rule::Axiom,
            premises: vec![],
        };

        let mut renderer = DotRenderer::new();
        renderer.direction = Direction::TopToBottom;

        let output = renderer.render(&proof);
        assert!(output.contains("rankdir=TB"));
    }

    #[test]
    fn test_no_rules() {
        let proof = Proof {
            conclusion: Sequent::new(vec![Formula::atom("A")]),
            rule: Rule::Axiom,
            premises: vec![],
        };

        let mut renderer = DotRenderer::new();
        renderer.show_rules = false;

        let output = renderer.render(&proof);
        assert!(!output.contains("Axiom"));
    }

    #[test]
    fn test_proof_net() {
        let proof = Proof {
            conclusion: Sequent::new(vec![Formula::atom("A")]),
            rule: Rule::Axiom,
            premises: vec![],
        };

        let renderer = DotRenderer::new();
        let output = renderer.render_proof_net(&proof);

        assert!(output.contains("proof_net"));
        assert!(output.contains("circle"));
    }
}
