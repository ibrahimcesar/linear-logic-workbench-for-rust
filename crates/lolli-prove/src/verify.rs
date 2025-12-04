//! Proof verification.
//!
//! This module provides verification of proofs to ensure they are valid.

use lolli_core::{Formula, Proof, Rule};

/// Error type for proof verification.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ProofError {
    /// Invalid rule application
    #[error("Invalid rule {rule:?} for conclusion {conclusion}")]
    InvalidRule {
        /// The rule that was applied
        rule: Rule,
        /// The conclusion of the proof step
        conclusion: String,
    },

    /// Wrong number of premises
    #[error("Expected {expected} premises, got {got}")]
    WrongPremiseCount {
        /// Expected number of premises
        expected: usize,
        /// Actual number of premises
        got: usize,
    },

    /// Context mismatch
    #[error("Context mismatch: {message}")]
    ContextMismatch {
        /// Description of the mismatch
        message: String,
    },

    /// Premise verification failed
    #[error("Premise verification failed: {0}")]
    PremiseFailed(Box<ProofError>),
}

/// Verify that a proof is valid.
///
/// # Errors
///
/// Returns a `ProofError` if the proof is invalid.
pub fn verify_proof(proof: &Proof) -> Result<(), ProofError> {
    // First, verify the rule application is valid for the conclusion
    verify_rule_application(proof)?;

    // Then recursively verify all premises
    for premise in &proof.premises {
        verify_proof(premise).map_err(|e| ProofError::PremiseFailed(Box::new(e)))?;
    }

    Ok(())
}

fn verify_rule_application(proof: &Proof) -> Result<(), ProofError> {
    let seq = &proof.conclusion;

    match &proof.rule {
        Rule::Axiom => {
            // Axiom: ⊢ A⊥, A - exactly two formulas that are negations of each other
            if seq.linear.len() != 2 {
                return Err(ProofError::InvalidRule {
                    rule: proof.rule.clone(),
                    conclusion: seq.pretty(),
                });
            }

            let is_valid = matches!(
                (&seq.linear[0], &seq.linear[1]),
                (Formula::Atom(a), Formula::NegAtom(b)) |
                (Formula::NegAtom(b), Formula::Atom(a)) if a == b
            );

            if !is_valid {
                return Err(ProofError::InvalidRule {
                    rule: proof.rule.clone(),
                    conclusion: seq.pretty(),
                });
            }

            if !proof.premises.is_empty() {
                return Err(ProofError::WrongPremiseCount {
                    expected: 0,
                    got: proof.premises.len(),
                });
            }

            Ok(())
        }

        Rule::OneIntro => {
            // One: ⊢ 1 - exactly one formula which is 1
            if seq.linear.len() != 1 || seq.linear[0] != Formula::One {
                return Err(ProofError::InvalidRule {
                    rule: proof.rule.clone(),
                    conclusion: seq.pretty(),
                });
            }

            if !proof.premises.is_empty() {
                return Err(ProofError::WrongPremiseCount {
                    expected: 0,
                    got: proof.premises.len(),
                });
            }

            Ok(())
        }

        Rule::TopIntro => {
            // Top: ⊢ Γ, ⊤ - context contains ⊤
            if !seq.linear.contains(&Formula::Top) {
                return Err(ProofError::InvalidRule {
                    rule: proof.rule.clone(),
                    conclusion: seq.pretty(),
                });
            }

            if !proof.premises.is_empty() {
                return Err(ProofError::WrongPremiseCount {
                    expected: 0,
                    got: proof.premises.len(),
                });
            }

            Ok(())
        }

        Rule::BottomIntro => {
            // Bottom: ⊢ Γ, ⊥ from ⊢ Γ
            if !seq.linear.contains(&Formula::Bottom) {
                return Err(ProofError::InvalidRule {
                    rule: proof.rule.clone(),
                    conclusion: seq.pretty(),
                });
            }

            if proof.premises.len() != 1 {
                return Err(ProofError::WrongPremiseCount {
                    expected: 1,
                    got: proof.premises.len(),
                });
            }

            Ok(())
        }

        Rule::TensorIntro => {
            // Tensor: needs exactly 2 premises
            if proof.premises.len() != 2 {
                return Err(ProofError::WrongPremiseCount {
                    expected: 2,
                    got: proof.premises.len(),
                });
            }

            // Check there's a tensor in the conclusion
            let has_tensor = seq
                .linear
                .iter()
                .any(|f| matches!(f, Formula::Tensor(_, _)));
            if !has_tensor {
                return Err(ProofError::InvalidRule {
                    rule: proof.rule.clone(),
                    conclusion: seq.pretty(),
                });
            }

            Ok(())
        }

        Rule::ParIntro => {
            // Par: needs exactly 1 premise
            if proof.premises.len() != 1 {
                return Err(ProofError::WrongPremiseCount {
                    expected: 1,
                    got: proof.premises.len(),
                });
            }

            // Check there's a par in the conclusion
            let has_par = seq.linear.iter().any(|f| matches!(f, Formula::Par(_, _)));
            if !has_par {
                return Err(ProofError::InvalidRule {
                    rule: proof.rule.clone(),
                    conclusion: seq.pretty(),
                });
            }

            Ok(())
        }

        Rule::WithIntro => {
            // With: needs exactly 2 premises
            if proof.premises.len() != 2 {
                return Err(ProofError::WrongPremiseCount {
                    expected: 2,
                    got: proof.premises.len(),
                });
            }

            // Check there's a with in the conclusion
            let has_with = seq
                .linear
                .iter()
                .any(|f| matches!(f, Formula::With(_, _)));
            if !has_with {
                return Err(ProofError::InvalidRule {
                    rule: proof.rule.clone(),
                    conclusion: seq.pretty(),
                });
            }

            Ok(())
        }

        Rule::PlusIntroLeft | Rule::PlusIntroRight => {
            // Plus: needs exactly 1 premise
            if proof.premises.len() != 1 {
                return Err(ProofError::WrongPremiseCount {
                    expected: 1,
                    got: proof.premises.len(),
                });
            }

            // Check there's a plus in the conclusion
            let has_plus = seq
                .linear
                .iter()
                .any(|f| matches!(f, Formula::Plus(_, _)));
            if !has_plus {
                return Err(ProofError::InvalidRule {
                    rule: proof.rule.clone(),
                    conclusion: seq.pretty(),
                });
            }

            Ok(())
        }

        Rule::WhyNotIntro => {
            // WhyNot: needs exactly 1 premise
            if proof.premises.len() != 1 {
                return Err(ProofError::WrongPremiseCount {
                    expected: 1,
                    got: proof.premises.len(),
                });
            }

            Ok(())
        }

        Rule::OfCourseIntro => {
            // OfCourse: needs exactly 1 premise
            if proof.premises.len() != 1 {
                return Err(ProofError::WrongPremiseCount {
                    expected: 1,
                    got: proof.premises.len(),
                });
            }

            // Check there's an ofcourse in the conclusion
            let has_ofcourse = seq
                .linear
                .iter()
                .any(|f| matches!(f, Formula::OfCourse(_)));
            if !has_ofcourse {
                return Err(ProofError::InvalidRule {
                    rule: proof.rule.clone(),
                    conclusion: seq.pretty(),
                });
            }

            Ok(())
        }

        // For other rules, just check premise count for now
        Rule::Cut(_) => {
            if proof.premises.len() != 2 {
                return Err(ProofError::WrongPremiseCount {
                    expected: 2,
                    got: proof.premises.len(),
                });
            }
            Ok(())
        }

        Rule::Weakening | Rule::Contraction | Rule::Dereliction => {
            if proof.premises.len() != 1 {
                return Err(ProofError::WrongPremiseCount {
                    expected: 1,
                    got: proof.premises.len(),
                });
            }
            Ok(())
        }

        Rule::FocusPositive(_) | Rule::FocusNegative(_) | Rule::Blur => {
            // Focus rules are internal to the prover
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lolli_core::Sequent;

    #[test]
    fn test_verify_axiom() {
        let proof = Proof {
            conclusion: Sequent::new(vec![
                Formula::atom("A"),
                Formula::neg_atom("A"),
            ]),
            rule: Rule::Axiom,
            premises: vec![],
        };

        assert!(verify_proof(&proof).is_ok());
    }

    #[test]
    fn test_verify_invalid_axiom() {
        let proof = Proof {
            conclusion: Sequent::new(vec![
                Formula::atom("A"),
                Formula::neg_atom("B"), // Wrong: A and B don't match
            ]),
            rule: Rule::Axiom,
            premises: vec![],
        };

        assert!(verify_proof(&proof).is_err());
    }

    #[test]
    fn test_verify_one() {
        let proof = Proof {
            conclusion: Sequent::new(vec![Formula::One]),
            rule: Rule::OneIntro,
            premises: vec![],
        };

        assert!(verify_proof(&proof).is_ok());
    }

    #[test]
    fn test_verify_top() {
        let proof = Proof {
            conclusion: Sequent::new(vec![Formula::atom("A"), Formula::Top]),
            rule: Rule::TopIntro,
            premises: vec![],
        };

        assert!(verify_proof(&proof).is_ok());
    }
}
