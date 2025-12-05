//! LaTeX proof rendering using bussproofs package.
//!
//! Generates LaTeX code for typesetting proofs.

use lolli_core::Proof;

/// LaTeX proof renderer using bussproofs package.
pub struct LatexRenderer {
    /// Include package imports
    pub include_preamble: bool,
    /// Use shorthand rule labels
    pub short_labels: bool,
}

impl Default for LatexRenderer {
    fn default() -> Self {
        Self {
            include_preamble: false,
            short_labels: false,
        }
    }
}

impl LatexRenderer {
    /// Create a new LaTeX renderer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Render a proof as LaTeX.
    pub fn render(&self, proof: &Proof) -> String {
        let mut lines = Vec::new();

        if self.include_preamble {
            lines.push(r"\usepackage{bussproofs}".to_string());
            lines.push(r"\usepackage{amsmath}".to_string());
            lines.push(r"\usepackage{amssymb}".to_string());
            lines.push(String::new());
        }

        lines.push(r"\begin{prooftree}".to_string());
        self.render_proof(proof, &mut lines);
        lines.push(r"\end{prooftree}".to_string());

        lines.join("\n")
    }

    /// Render a complete LaTeX document.
    pub fn render_document(&self, proof: &Proof) -> String {
        let mut lines = Vec::new();

        lines.push(r"\documentclass{article}".to_string());
        lines.push(r"\usepackage{bussproofs}".to_string());
        lines.push(r"\usepackage{amsmath}".to_string());
        lines.push(r"\usepackage{amssymb}".to_string());
        lines.push(String::new());
        lines.push(r"\begin{document}".to_string());
        lines.push(String::new());

        lines.push(r"\begin{prooftree}".to_string());
        self.render_proof(proof, &mut lines);
        lines.push(r"\end{prooftree}".to_string());

        lines.push(String::new());
        lines.push(r"\end{document}".to_string());

        lines.join("\n")
    }

    /// Render a proof recursively.
    fn render_proof(&self, proof: &Proof, lines: &mut Vec<String>) {
        // Render premises first
        for premise in &proof.premises {
            self.render_proof(premise, lines);
        }

        // Format the conclusion
        let conclusion = self.format_sequent(proof);
        let rule_label = self.format_rule(&proof.rule);

        // Generate the appropriate inference command
        match proof.premises.len() {
            0 => {
                lines.push(format!(r"  \AxiomC{{$\vdash {}$}}", conclusion));
            }
            1 => {
                lines.push(format!(
                    r"  \RightLabel{{\scriptsize {}}}",
                    rule_label
                ));
                lines.push(format!(r"  \UnaryInfC{{$\vdash {}$}}", conclusion));
            }
            2 => {
                lines.push(format!(
                    r"  \RightLabel{{\scriptsize {}}}",
                    rule_label
                ));
                lines.push(format!(r"  \BinaryInfC{{$\vdash {}$}}", conclusion));
            }
            3 => {
                lines.push(format!(
                    r"  \RightLabel{{\scriptsize {}}}",
                    rule_label
                ));
                lines.push(format!(r"  \TrinaryInfC{{$\vdash {}$}}", conclusion));
            }
            _ => {
                // For more than 3 premises, we'd need a different approach
                lines.push(format!(
                    r"  \RightLabel{{\scriptsize {}}}",
                    rule_label
                ));
                lines.push(format!(r"  \QuaternaryInfC{{$\vdash {}$}}", conclusion));
            }
        }
    }

    /// Format a sequent in LaTeX.
    fn format_sequent(&self, proof: &Proof) -> String {
        proof
            .conclusion
            .linear
            .iter()
            .map(|f| f.pretty_latex())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Format a rule name for LaTeX.
    fn format_rule(&self, rule: &lolli_core::Rule) -> String {
        use lolli_core::Rule;

        if self.short_labels {
            match rule {
                Rule::Axiom => "ax".to_string(),
                Rule::Cut(_) => "cut".to_string(),
                Rule::OneIntro => "$1$".to_string(),
                Rule::BottomIntro => "$\\bot$".to_string(),
                Rule::TopIntro => "$\\top$".to_string(),
                Rule::TensorIntro => "$\\otimes$".to_string(),
                Rule::ParIntro => "$\\parr$".to_string(),
                Rule::WithIntro => "$\\with$".to_string(),
                Rule::PlusIntroLeft => "$\\oplus_L$".to_string(),
                Rule::PlusIntroRight => "$\\oplus_R$".to_string(),
                Rule::OfCourseIntro => "$!$".to_string(),
                Rule::WhyNotIntro => "$?$".to_string(),
                Rule::Weakening => "W".to_string(),
                Rule::Contraction => "C".to_string(),
                Rule::Dereliction => "D".to_string(),
                Rule::FocusPositive(_) => "F+".to_string(),
                Rule::FocusNegative(_) => "F-".to_string(),
                Rule::Blur => "B".to_string(),
            }
        } else {
            match rule {
                Rule::Axiom => "axiom".to_string(),
                Rule::Cut(_) => "cut".to_string(),
                Rule::OneIntro => "$1$-intro".to_string(),
                Rule::BottomIntro => "$\\bot$-intro".to_string(),
                Rule::TopIntro => "$\\top$-intro".to_string(),
                Rule::TensorIntro => "$\\otimes$-intro".to_string(),
                Rule::ParIntro => "$\\parr$-intro".to_string(),
                Rule::WithIntro => "$\\with$-intro".to_string(),
                Rule::PlusIntroLeft => "$\\oplus$-intro$_L$".to_string(),
                Rule::PlusIntroRight => "$\\oplus$-intro$_R$".to_string(),
                Rule::OfCourseIntro => "$!$-intro".to_string(),
                Rule::WhyNotIntro => "$?$-intro".to_string(),
                Rule::Weakening => "weakening".to_string(),
                Rule::Contraction => "contraction".to_string(),
                Rule::Dereliction => "dereliction".to_string(),
                Rule::FocusPositive(_) => "focus+".to_string(),
                Rule::FocusNegative(_) => "focus-".to_string(),
                Rule::Blur => "blur".to_string(),
            }
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

        let renderer = LatexRenderer::new();
        let output = renderer.render(&proof);

        assert!(output.contains(r"\begin{prooftree}"));
        assert!(output.contains(r"\AxiomC"));
        assert!(output.contains(r"\end{prooftree}"));
    }

    #[test]
    fn test_render_binary() {
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

        let renderer = LatexRenderer::new();
        let output = renderer.render(&proof);

        assert!(output.contains(r"\BinaryInfC"));
        assert!(output.contains(r"\otimes"));
    }

    #[test]
    fn test_with_preamble() {
        let proof = Proof {
            conclusion: Sequent::new(vec![Formula::atom("A")]),
            rule: Rule::Axiom,
            premises: vec![],
        };

        let mut renderer = LatexRenderer::new();
        renderer.include_preamble = true;

        let output = renderer.render(&proof);
        assert!(output.contains(r"\usepackage{bussproofs}"));
    }

    #[test]
    fn test_short_labels() {
        let proof = Proof {
            conclusion: Sequent::new(vec![Formula::One]),
            rule: Rule::OneIntro,
            premises: vec![],
        };

        let mut renderer = LatexRenderer::new();
        renderer.short_labels = true;

        let output = renderer.render(&proof);
        // Short labels should not include "-intro"
        assert!(!output.contains("-intro"));
    }

    #[test]
    fn test_document() {
        let proof = Proof {
            conclusion: Sequent::new(vec![Formula::atom("A")]),
            rule: Rule::Axiom,
            premises: vec![],
        };

        let renderer = LatexRenderer::new();
        let output = renderer.render_document(&proof);

        assert!(output.contains(r"\documentclass"));
        assert!(output.contains(r"\begin{document}"));
        assert!(output.contains(r"\end{document}"));
    }
}
