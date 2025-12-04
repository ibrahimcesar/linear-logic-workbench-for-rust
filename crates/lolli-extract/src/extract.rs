//! Term extraction from proofs.
//!
//! This module implements the Curry-Howard correspondence for linear logic,
//! extracting computational terms from cut-free proofs.

use lolli_core::{Formula, Proof, Rule, Term};

/// Term extractor using Curry-Howard correspondence.
///
/// Extracts linear lambda terms from linear logic proofs.
pub struct Extractor {
    /// Counter for generating fresh variable names
    var_counter: usize,
}

impl Default for Extractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Extractor {
    /// Create a new extractor.
    pub fn new() -> Self {
        Self {
            var_counter: 0,
        }
    }

    /// Generate a fresh variable name.
    pub fn fresh_var(&mut self) -> String {
        let v = format!("x{}", self.var_counter);
        self.var_counter += 1;
        v
    }

    /// Generate a variable name based on a formula.
    pub fn var_for_formula(&mut self, formula: &Formula) -> String {
        match formula {
            Formula::Atom(name) | Formula::NegAtom(name) => {
                let base = name.to_lowercase();
                let v = format!("{}{}", base, self.var_counter);
                self.var_counter += 1;
                v
            }
            _ => self.fresh_var(),
        }
    }

    /// Extract a term from a proof.
    ///
    /// The proof should be cut-free for best results. The extracted term
    /// represents the computational content of the proof via Curry-Howard.
    pub fn extract(&mut self, proof: &Proof) -> Term {
        self.extract_with_env(proof, &mut Vec::new())
    }

    /// Extract with an environment mapping formula indices to variables.
    fn extract_with_env(&mut self, proof: &Proof, vars: &mut Vec<(Formula, String)>) -> Term {
        match &proof.rule {
            Rule::Axiom => {
                // Axiom: ⊢ A⊥, A
                // This represents identity: λx. x
                // Find the positive formula and create an identity
                self.extract_axiom(proof, vars)
            }

            Rule::OneIntro => {
                // ⊢ 1 corresponds to ()
                Term::Unit
            }

            Rule::TopIntro => {
                // ⊢ Γ, ⊤ corresponds to ⟨⟩ (trivial)
                Term::Trivial
            }

            Rule::BottomIntro => {
                // From ⊢ Γ derive ⊢ Γ, ⊥
                // Bottom is the unit for par, it doesn't contribute a term
                if !proof.premises.is_empty() {
                    self.extract_with_env(&proof.premises[0], vars)
                } else {
                    Term::Unit
                }
            }

            Rule::TensorIntro => {
                // From ⊢ Γ, A and ⊢ Δ, B derive ⊢ Γ, Δ, A ⊗ B
                // Tensor corresponds to pairing: (a, b)
                self.extract_tensor(proof, vars)
            }

            Rule::ParIntro => {
                // From ⊢ Γ, A, B derive ⊢ Γ, A ⅋ B
                // Par is the dual of tensor, corresponds to curried function
                if !proof.premises.is_empty() {
                    self.extract_with_env(&proof.premises[0], vars)
                } else {
                    Term::Unit
                }
            }

            Rule::WithIntro => {
                // From ⊢ Γ, A and ⊢ Γ, B derive ⊢ Γ, A & B
                // With corresponds to lazy pair: ⟨fst, snd⟩
                self.extract_with(proof, vars)
            }

            Rule::PlusIntroLeft => {
                // From ⊢ Γ, A derive ⊢ Γ, A ⊕ B
                // Plus left corresponds to: inl a
                if !proof.premises.is_empty() {
                    let inner = self.extract_with_env(&proof.premises[0], vars);
                    Term::Inl(Box::new(inner))
                } else {
                    Term::Inl(Box::new(Term::Unit))
                }
            }

            Rule::PlusIntroRight => {
                // From ⊢ Γ, B derive ⊢ Γ, A ⊕ B
                // Plus right corresponds to: inr b
                if !proof.premises.is_empty() {
                    let inner = self.extract_with_env(&proof.premises[0], vars);
                    Term::Inr(Box::new(inner))
                } else {
                    Term::Inr(Box::new(Term::Unit))
                }
            }

            Rule::OfCourseIntro => {
                // From ⊢ ?Γ, A derive ⊢ ?Γ, !A
                // Promote the term to be copyable
                if !proof.premises.is_empty() {
                    let inner = self.extract_with_env(&proof.premises[0], vars);
                    Term::Promote(Box::new(inner))
                } else {
                    Term::Promote(Box::new(Term::Unit))
                }
            }

            Rule::WhyNotIntro => {
                // From ⊢ Γ, A derive ⊢ Γ, ?A
                if !proof.premises.is_empty() {
                    self.extract_with_env(&proof.premises[0], vars)
                } else {
                    Term::Unit
                }
            }

            Rule::Weakening => {
                // From ⊢ Γ derive ⊢ Γ, ?A
                // Discard a resource
                if !proof.premises.is_empty() {
                    let body = self.extract_with_env(&proof.premises[0], vars);
                    Term::Discard(Box::new(Term::Unit), Box::new(body))
                } else {
                    Term::Unit
                }
            }

            Rule::Contraction => {
                // From ⊢ Γ, ?A, ?A derive ⊢ Γ, ?A
                // Copy a resource
                if !proof.premises.is_empty() {
                    let x = self.fresh_var();
                    let y = self.fresh_var();
                    let src = Term::Var(self.fresh_var());
                    let body = self.extract_with_env(&proof.premises[0], vars);
                    Term::Copy(Box::new(src), x, y, Box::new(body))
                } else {
                    Term::Unit
                }
            }

            Rule::Dereliction => {
                // Use !A as A
                if !proof.premises.is_empty() {
                    let inner = self.extract_with_env(&proof.premises[0], vars);
                    Term::Derelict(Box::new(inner))
                } else {
                    Term::Derelict(Box::new(Term::Unit))
                }
            }

            Rule::Cut(formula) => {
                // Cut: from ⊢ Γ, A and ⊢ Δ, A⊥ derive ⊢ Γ, Δ
                // This corresponds to let-binding or application
                self.extract_cut(proof, formula, vars)
            }

            Rule::FocusPositive(_) | Rule::FocusNegative(_) | Rule::Blur => {
                // Focus rules don't change the term, just pass through
                if !proof.premises.is_empty() {
                    self.extract_with_env(&proof.premises[0], vars)
                } else {
                    Term::Unit
                }
            }
        }
    }

    /// Extract term for axiom rule.
    fn extract_axiom(&mut self, proof: &Proof, vars: &[(Formula, String)]) -> Term {
        // The axiom ⊢ A⊥, A represents identity
        // Look for matching formulas in the conclusion
        let formulas = &proof.conclusion.linear;

        // Find atoms and their negations
        for formula in formulas {
            match formula {
                Formula::Atom(name) => {
                    // Check if we have a variable for this in the environment
                    for (f, v) in vars {
                        if let Formula::NegAtom(neg_name) = f {
                            if neg_name == name {
                                return Term::Var(v.clone());
                            }
                        }
                    }
                    // Otherwise create a fresh variable representing identity
                    return Term::Var(name.to_lowercase());
                }
                Formula::NegAtom(name) => {
                    // Check if we have a variable for the positive in the environment
                    for (f, v) in vars {
                        if let Formula::Atom(pos_name) = f {
                            if pos_name == name {
                                return Term::Var(v.clone());
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Default: create identity function
        let var = self.fresh_var();
        Term::Abs(var.clone(), Box::new(Term::Var(var)))
    }

    /// Extract term for tensor introduction.
    fn extract_tensor(&mut self, proof: &Proof, vars: &mut Vec<(Formula, String)>) -> Term {
        if proof.premises.len() == 2 {
            let left = self.extract_with_env(&proof.premises[0], vars);
            let right = self.extract_with_env(&proof.premises[1], vars);
            Term::Pair(Box::new(left), Box::new(right))
        } else if proof.premises.len() == 1 {
            self.extract_with_env(&proof.premises[0], vars)
        } else {
            Term::Unit
        }
    }

    /// Extract term for with introduction.
    fn extract_with(&mut self, proof: &Proof, vars: &mut Vec<(Formula, String)>) -> Term {
        if proof.premises.len() == 2 {
            let left = self.extract_with_env(&proof.premises[0], vars);
            let right = self.extract_with_env(&proof.premises[1], vars);
            // With creates a pair that can be projected
            Term::Pair(Box::new(left), Box::new(right))
        } else if proof.premises.len() == 1 {
            self.extract_with_env(&proof.premises[0], vars)
        } else {
            Term::Trivial
        }
    }

    /// Extract term for cut rule.
    fn extract_cut(
        &mut self,
        proof: &Proof,
        cut_formula: &Formula,
        vars: &mut Vec<(Formula, String)>,
    ) -> Term {
        if proof.premises.len() != 2 {
            return Term::Unit;
        }

        // Generate a variable for the cut formula
        let cut_var = self.var_for_formula(cut_formula);

        // Extract the term that produces the cut formula
        let producer = self.extract_with_env(&proof.premises[0], vars);

        // Add the cut variable to the environment for the consumer
        vars.push((cut_formula.clone(), cut_var.clone()));
        let consumer = self.extract_with_env(&proof.premises[1], vars);
        vars.pop();

        // Cut corresponds to application or let-binding depending on the formula
        match cut_formula {
            Formula::Tensor(_, _) => {
                // Tensor cut becomes let-pair
                let x = self.fresh_var();
                let y = self.fresh_var();
                Term::LetPair(x, y, Box::new(producer), Box::new(consumer))
            }
            Formula::Plus(_, _) => {
                // Plus cut becomes case
                let x = self.fresh_var();
                let y = self.fresh_var();
                Term::Case(
                    Box::new(producer),
                    x,
                    Box::new(consumer.clone()),
                    y,
                    Box::new(consumer),
                )
            }
            _ => {
                // Default: application
                Term::App(Box::new(consumer), Box::new(producer))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lolli_core::Sequent;

    #[test]
    fn test_extract_axiom() {
        let proof = Proof {
            conclusion: Sequent::new(vec![Formula::neg_atom("A"), Formula::atom("A")]),
            rule: Rule::Axiom,
            premises: vec![],
        };

        let mut extractor = Extractor::new();
        let term = extractor.extract(&proof);

        // Axiom gives us a variable (identity)
        assert!(matches!(term, Term::Var(_) | Term::Abs(_, _)));
    }

    #[test]
    fn test_extract_one() {
        let proof = Proof {
            conclusion: Sequent::new(vec![Formula::One]),
            rule: Rule::OneIntro,
            premises: vec![],
        };

        let mut extractor = Extractor::new();
        let term = extractor.extract(&proof);

        assert_eq!(term, Term::Unit);
    }

    #[test]
    fn test_extract_top() {
        let proof = Proof {
            conclusion: Sequent::new(vec![Formula::atom("A"), Formula::Top]),
            rule: Rule::TopIntro,
            premises: vec![],
        };

        let mut extractor = Extractor::new();
        let term = extractor.extract(&proof);

        assert_eq!(term, Term::Trivial);
    }

    #[test]
    fn test_extract_tensor() {
        // Proof of ⊢ A⊥, B⊥, A ⊗ B
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

        let mut extractor = Extractor::new();
        let term = extractor.extract(&proof);

        // Should be a pair
        assert!(matches!(term, Term::Pair(_, _)));
    }

    #[test]
    fn test_extract_plus_left() {
        let inner = Proof {
            conclusion: Sequent::new(vec![Formula::neg_atom("A"), Formula::atom("A")]),
            rule: Rule::Axiom,
            premises: vec![],
        };
        let proof = Proof {
            conclusion: Sequent::new(vec![
                Formula::neg_atom("A"),
                Formula::plus(Formula::atom("A"), Formula::atom("B")),
            ]),
            rule: Rule::PlusIntroLeft,
            premises: vec![inner],
        };

        let mut extractor = Extractor::new();
        let term = extractor.extract(&proof);

        // Should be inl
        assert!(matches!(term, Term::Inl(_)));
    }

    #[test]
    fn test_extract_plus_right() {
        let inner = Proof {
            conclusion: Sequent::new(vec![Formula::neg_atom("B"), Formula::atom("B")]),
            rule: Rule::Axiom,
            premises: vec![],
        };
        let proof = Proof {
            conclusion: Sequent::new(vec![
                Formula::neg_atom("B"),
                Formula::plus(Formula::atom("A"), Formula::atom("B")),
            ]),
            rule: Rule::PlusIntroRight,
            premises: vec![inner],
        };

        let mut extractor = Extractor::new();
        let term = extractor.extract(&proof);

        // Should be inr
        assert!(matches!(term, Term::Inr(_)));
    }

    #[test]
    fn test_extract_with() {
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
            conclusion: Sequent::new(vec![Formula::with(Formula::atom("A"), Formula::atom("B"))]),
            rule: Rule::WithIntro,
            premises: vec![left, right],
        };

        let mut extractor = Extractor::new();
        let term = extractor.extract(&proof);

        // Should be a pair
        assert!(matches!(term, Term::Pair(_, _)));
    }

    #[test]
    fn test_extract_promote() {
        let inner = Proof {
            conclusion: Sequent::new(vec![Formula::atom("A")]),
            rule: Rule::Axiom,
            premises: vec![],
        };
        let proof = Proof {
            conclusion: Sequent::new(vec![Formula::of_course(Formula::atom("A"))]),
            rule: Rule::OfCourseIntro,
            premises: vec![inner],
        };

        let mut extractor = Extractor::new();
        let term = extractor.extract(&proof);

        // Should be promote
        assert!(matches!(term, Term::Promote(_)));
    }

    #[test]
    fn test_pretty_print() {
        let proof = Proof {
            conclusion: Sequent::new(vec![
                Formula::neg_atom("A"),
                Formula::neg_atom("B"),
                Formula::tensor(Formula::atom("A"), Formula::atom("B")),
            ]),
            rule: Rule::TensorIntro,
            premises: vec![
                Proof {
                    conclusion: Sequent::new(vec![Formula::neg_atom("A"), Formula::atom("A")]),
                    rule: Rule::Axiom,
                    premises: vec![],
                },
                Proof {
                    conclusion: Sequent::new(vec![Formula::neg_atom("B"), Formula::atom("B")]),
                    rule: Rule::Axiom,
                    premises: vec![],
                },
            ],
        };

        let mut extractor = Extractor::new();
        let term = extractor.extract(&proof);
        let pretty = term.pretty();

        // Should contain pair syntax
        assert!(pretty.contains(',') || pretty.contains('('));
    }
}
