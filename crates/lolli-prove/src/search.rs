//! Proof search implementation.
//!
//! This module implements focused proof search for MALL (Multiplicative-Additive Linear Logic).

use lolli_core::{Formula, Proof, Rule, Sequent, TwoSidedSequent};
use std::collections::HashSet;

/// A prover for linear logic sequents.
///
/// Uses focused proof search to efficiently find proofs.
pub struct Prover {
    /// Maximum search depth
    pub max_depth: usize,
    /// Enable caching of failed sequents
    pub use_cache: bool,
    /// Cache of unprovable sequents (normalized form)
    cache: HashSet<Vec<String>>,
    /// Statistics
    stats: ProverStats,
}

/// Statistics about the proof search.
#[derive(Debug, Default, Clone)]
pub struct ProverStats {
    /// Number of sequents explored
    pub sequents_explored: usize,
    /// Number of cache hits
    pub cache_hits: usize,
    /// Maximum depth reached
    pub max_depth_reached: usize,
}

impl Prover {
    /// Create a new prover with the given maximum depth.
    pub fn new(max_depth: usize) -> Self {
        Self {
            max_depth,
            use_cache: true,
            cache: HashSet::new(),
            stats: ProverStats::default(),
        }
    }

    /// Get proof search statistics.
    pub fn stats(&self) -> &ProverStats {
        &self.stats
    }

    /// Clear the cache and reset statistics.
    pub fn reset(&mut self) {
        self.cache.clear();
        self.stats = ProverStats::default();
    }

    /// Prove a two-sided sequent Γ ⊢ Δ.
    ///
    /// Returns `Some(proof)` if provable, `None` otherwise.
    pub fn prove_two_sided(&mut self, seq: &TwoSidedSequent) -> Option<Proof> {
        let one_sided = seq.to_one_sided();
        self.prove(&one_sided)
    }

    /// Prove a one-sided sequent ⊢ Γ.
    ///
    /// Returns `Some(proof)` if provable, `None` otherwise.
    pub fn prove(&mut self, seq: &Sequent) -> Option<Proof> {
        self.stats.sequents_explored = 0;
        self.stats.cache_hits = 0;
        self.stats.max_depth_reached = 0;
        self.prove_with_depth(seq, 0)
    }

    fn prove_with_depth(&mut self, seq: &Sequent, depth: usize) -> Option<Proof> {
        self.stats.sequents_explored += 1;
        if depth > self.stats.max_depth_reached {
            self.stats.max_depth_reached = depth;
        }

        // Depth limit check
        if depth > self.max_depth {
            return None;
        }

        // Check cache
        if self.use_cache {
            let key = self.sequent_key(seq);
            if self.cache.contains(&key) {
                self.stats.cache_hits += 1;
                return None;
            }
        }

        // Try to prove
        let result = self.prove_async(seq, depth);

        // Cache negative result
        if result.is_none() && self.use_cache {
            let key = self.sequent_key(seq);
            self.cache.insert(key);
        }

        result
    }

    /// Asynchronous phase: apply all invertible (negative) rules.
    fn prove_async(&mut self, seq: &Sequent, depth: usize) -> Option<Proof> {
        // First, check for empty sequent (contradiction/impossible)
        if seq.linear.is_empty() && seq.focus.is_none() {
            // Empty sequent is not provable in linear logic
            return None;
        }

        // Look for negative formulas to decompose
        for (i, formula) in seq.linear.iter().enumerate() {
            match formula {
                // Par (⅋) - invertible: ⊢ Γ, A ⅋ B becomes ⊢ Γ, A, B
                Formula::Par(a, b) => {
                    let mut new_linear = seq.linear.clone();
                    new_linear.remove(i);
                    new_linear.push(a.as_ref().clone());
                    new_linear.push(b.as_ref().clone());
                    let new_seq = Sequent {
                        linear: new_linear,
                        unrestricted: seq.unrestricted.clone(),
                        focus: None,
                    };

                    if let Some(premise) = self.prove_with_depth(&new_seq, depth + 1) {
                        return Some(Proof {
                            conclusion: seq.clone(),
                            rule: Rule::ParIntro,
                            premises: vec![premise],
                        });
                    }
                    return None;
                }

                // Bottom (⊥) - invertible: ⊢ Γ, ⊥ becomes ⊢ Γ
                Formula::Bottom => {
                    let mut new_linear = seq.linear.clone();
                    new_linear.remove(i);
                    let new_seq = Sequent {
                        linear: new_linear,
                        unrestricted: seq.unrestricted.clone(),
                        focus: None,
                    };

                    if let Some(premise) = self.prove_with_depth(&new_seq, depth + 1) {
                        return Some(Proof {
                            conclusion: seq.clone(),
                            rule: Rule::BottomIntro,
                            premises: vec![premise],
                        });
                    }
                    return None;
                }

                // Top (⊤) - always provable, no premise needed
                Formula::Top => {
                    return Some(Proof {
                        conclusion: seq.clone(),
                        rule: Rule::TopIntro,
                        premises: vec![],
                    });
                }

                // With (&) - invertible: ⊢ Γ, A & B needs ⊢ Γ, A AND ⊢ Γ, B
                Formula::With(a, b) => {
                    let mut left_linear = seq.linear.clone();
                    left_linear.remove(i);
                    left_linear.push(a.as_ref().clone());
                    let left_seq = Sequent {
                        linear: left_linear,
                        unrestricted: seq.unrestricted.clone(),
                        focus: None,
                    };

                    let mut right_linear = seq.linear.clone();
                    right_linear.remove(i);
                    right_linear.push(b.as_ref().clone());
                    let right_seq = Sequent {
                        linear: right_linear,
                        unrestricted: seq.unrestricted.clone(),
                        focus: None,
                    };

                    if let Some(left_proof) = self.prove_with_depth(&left_seq, depth + 1) {
                        if let Some(right_proof) = self.prove_with_depth(&right_seq, depth + 1) {
                            return Some(Proof {
                                conclusion: seq.clone(),
                                rule: Rule::WithIntro,
                                premises: vec![left_proof, right_proof],
                            });
                        }
                    }
                    return None;
                }

                // WhyNot (?) - move to unrestricted zone
                Formula::WhyNot(a) => {
                    let mut new_linear = seq.linear.clone();
                    new_linear.remove(i);
                    let mut new_unrestricted = seq.unrestricted.clone();
                    new_unrestricted.push(a.as_ref().clone());
                    let new_seq = Sequent {
                        linear: new_linear,
                        unrestricted: new_unrestricted,
                        focus: None,
                    };

                    if let Some(premise) = self.prove_with_depth(&new_seq, depth + 1) {
                        return Some(Proof {
                            conclusion: seq.clone(),
                            rule: Rule::WhyNotIntro,
                            premises: vec![premise],
                        });
                    }
                    return None;
                }

                // Lolli (⊸) is sugar for Par, so desugar it
                Formula::Lolli(a, b) => {
                    let desugared = Formula::Par(Box::new(a.negate()), b.clone());
                    let mut new_linear = seq.linear.clone();
                    new_linear[i] = desugared;
                    let new_seq = Sequent {
                        linear: new_linear,
                        unrestricted: seq.unrestricted.clone(),
                        focus: None,
                    };

                    return self.prove_with_depth(&new_seq, depth);
                }

                _ => {}
            }
        }

        // No invertible rules apply, go to synchronous phase
        self.prove_sync(seq, depth)
    }

    /// Synchronous phase: choose a formula to focus on.
    fn prove_sync(&mut self, seq: &Sequent, depth: usize) -> Option<Proof> {
        // Try focusing on each positive formula
        for i in 0..seq.linear.len() {
            if seq.linear[i].is_positive() {
                if let Some(proof) = self.prove_focused(seq, i, depth) {
                    return Some(proof);
                }
            }
        }

        // Also try focusing on negated atoms (they act like axioms with their positive counterpart)
        for i in 0..seq.linear.len() {
            if matches!(&seq.linear[i], Formula::NegAtom(_)) {
                if let Some(proof) = self.try_axiom(seq, i) {
                    return Some(proof);
                }
            }
        }

        None
    }

    /// Focused phase: decompose a positive formula.
    fn prove_focused(&mut self, seq: &Sequent, idx: usize, depth: usize) -> Option<Proof> {
        let formula = &seq.linear[idx];

        match formula {
            // Atom - look for matching negated atom (axiom)
            Formula::Atom(name) => {
                // Look for A⊥ in the context
                for (j, other) in seq.linear.iter().enumerate() {
                    if j != idx {
                        if let Formula::NegAtom(other_name) = other {
                            if name == other_name {
                                // Check that these are the only two formulas
                                if seq.linear.len() == 2 {
                                    return Some(Proof {
                                        conclusion: seq.clone(),
                                        rule: Rule::Axiom,
                                        premises: vec![],
                                    });
                                }
                            }
                        }
                    }
                }
                None
            }

            // One (1) - context must be empty
            Formula::One => {
                if seq.linear.len() == 1 {
                    Some(Proof {
                        conclusion: seq.clone(),
                        rule: Rule::OneIntro,
                        premises: vec![],
                    })
                } else {
                    None
                }
            }

            // Zero (0) - never provable
            Formula::Zero => None,

            // Tensor (⊗) - split the context
            Formula::Tensor(a, b) => {
                let mut other_formulas: Vec<Formula> = seq.linear.clone();
                other_formulas.remove(idx);

                // Try all possible splits of the remaining context
                for split in all_splits(&other_formulas) {
                    let (left_ctx, right_ctx) = split;

                    let mut left_linear = left_ctx;
                    left_linear.push(a.as_ref().clone());
                    let left_seq = Sequent {
                        linear: left_linear,
                        unrestricted: seq.unrestricted.clone(),
                        focus: None,
                    };

                    let mut right_linear = right_ctx;
                    right_linear.push(b.as_ref().clone());
                    let right_seq = Sequent {
                        linear: right_linear,
                        unrestricted: seq.unrestricted.clone(),
                        focus: None,
                    };

                    if let Some(left_proof) = self.prove_with_depth(&left_seq, depth + 1) {
                        if let Some(right_proof) = self.prove_with_depth(&right_seq, depth + 1) {
                            return Some(Proof {
                                conclusion: seq.clone(),
                                rule: Rule::TensorIntro,
                                premises: vec![left_proof, right_proof],
                            });
                        }
                    }
                }
                None
            }

            // Plus (⊕) - choose left or right
            Formula::Plus(a, b) => {
                // Try left
                let mut left_linear = seq.linear.clone();
                left_linear[idx] = a.as_ref().clone();
                let left_seq = Sequent {
                    linear: left_linear,
                    unrestricted: seq.unrestricted.clone(),
                    focus: None,
                };

                if let Some(premise) = self.prove_with_depth(&left_seq, depth + 1) {
                    return Some(Proof {
                        conclusion: seq.clone(),
                        rule: Rule::PlusIntroLeft,
                        premises: vec![premise],
                    });
                }

                // Try right
                let mut right_linear = seq.linear.clone();
                right_linear[idx] = b.as_ref().clone();
                let right_seq = Sequent {
                    linear: right_linear,
                    unrestricted: seq.unrestricted.clone(),
                    focus: None,
                };

                if let Some(premise) = self.prove_with_depth(&right_seq, depth + 1) {
                    return Some(Proof {
                        conclusion: seq.clone(),
                        rule: Rule::PlusIntroRight,
                        premises: vec![premise],
                    });
                }

                None
            }

            // OfCourse (!) - requires all linear context to be empty
            Formula::OfCourse(a) => {
                // For !, we need the linear context to be only the !A itself
                // and derive from the unrestricted context + A
                if seq.linear.len() == 1 {
                    let new_seq = Sequent {
                        linear: vec![a.as_ref().clone()],
                        unrestricted: seq.unrestricted.clone(),
                        focus: None,
                    };

                    if let Some(premise) = self.prove_with_depth(&new_seq, depth + 1) {
                        return Some(Proof {
                            conclusion: seq.clone(),
                            rule: Rule::OfCourseIntro,
                            premises: vec![premise],
                        });
                    }
                }
                None
            }

            _ => None,
        }
    }

    /// Try to apply the axiom rule with a negated atom.
    fn try_axiom(&mut self, seq: &Sequent, neg_idx: usize) -> Option<Proof> {
        if let Formula::NegAtom(name) = &seq.linear[neg_idx] {
            // Look for matching positive atom
            for (j, other) in seq.linear.iter().enumerate() {
                if j != neg_idx {
                    if let Formula::Atom(other_name) = other {
                        if name == other_name && seq.linear.len() == 2 {
                            return Some(Proof {
                                conclusion: seq.clone(),
                                rule: Rule::Axiom,
                                premises: vec![],
                            });
                        }
                    }
                }
            }
        }
        None
    }

    /// Create a canonical key for a sequent (for caching).
    fn sequent_key(&self, seq: &Sequent) -> Vec<String> {
        let mut keys: Vec<String> = seq.linear.iter().map(|f| f.pretty()).collect();
        keys.sort();
        keys
    }
}

/// Generate all possible ways to split a list into two parts.
fn all_splits<T: Clone>(items: &[T]) -> Vec<(Vec<T>, Vec<T>)> {
    if items.is_empty() {
        return vec![(vec![], vec![])];
    }

    let n = items.len();
    let mut splits = Vec::new();

    // Each item can go to left (0) or right (1)
    for mask in 0..(1 << n) {
        let mut left = Vec::new();
        let mut right = Vec::new();

        for (i, item) in items.iter().enumerate() {
            if (mask >> i) & 1 == 0 {
                left.push(item.clone());
            } else {
                right.push(item.clone());
            }
        }

        splits.push((left, right));
    }

    splits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atom(name: &str) -> Formula {
        Formula::atom(name)
    }

    fn neg_atom(name: &str) -> Formula {
        Formula::neg_atom(name)
    }

    #[test]
    fn test_identity() {
        // A ⊢ A
        let mut prover = Prover::new(100);
        let seq = TwoSidedSequent::new(vec![atom("A")], vec![atom("A")]);
        let result = prover.prove_two_sided(&seq);
        assert!(result.is_some(), "A ⊢ A should be provable");
    }

    #[test]
    fn test_tensor_intro() {
        // A, B ⊢ A ⊗ B
        let mut prover = Prover::new(100);
        let seq = TwoSidedSequent::new(
            vec![atom("A"), atom("B")],
            vec![Formula::tensor(atom("A"), atom("B"))],
        );
        let result = prover.prove_two_sided(&seq);
        assert!(result.is_some(), "A, B ⊢ A ⊗ B should be provable");
    }

    #[test]
    fn test_tensor_commutativity() {
        // A ⊗ B ⊢ B ⊗ A
        let mut prover = Prover::new(100);
        let seq = TwoSidedSequent::new(
            vec![Formula::tensor(atom("A"), atom("B"))],
            vec![Formula::tensor(atom("B"), atom("A"))],
        );
        let result = prover.prove_two_sided(&seq);
        assert!(result.is_some(), "A ⊗ B ⊢ B ⊗ A should be provable");
    }

    #[test]
    fn test_with_elimination() {
        // A & B ⊢ A
        let mut prover = Prover::new(100);
        let seq = TwoSidedSequent::new(
            vec![Formula::with(atom("A"), atom("B"))],
            vec![atom("A")],
        );
        let result = prover.prove_two_sided(&seq);
        assert!(result.is_some(), "A & B ⊢ A should be provable");
    }

    #[test]
    fn test_plus_intro() {
        // A ⊢ A ⊕ B
        let mut prover = Prover::new(100);
        let seq = TwoSidedSequent::new(
            vec![atom("A")],
            vec![Formula::plus(atom("A"), atom("B"))],
        );
        let result = prover.prove_two_sided(&seq);
        assert!(result.is_some(), "A ⊢ A ⊕ B should be provable");
    }

    #[test]
    fn test_no_contraction() {
        // A ⊢ A ⊗ A should NOT be provable (no contraction in linear logic)
        let mut prover = Prover::new(100);
        let seq = TwoSidedSequent::new(
            vec![atom("A")],
            vec![Formula::tensor(atom("A"), atom("A"))],
        );
        let result = prover.prove_two_sided(&seq);
        assert!(result.is_none(), "A ⊢ A ⊗ A should NOT be provable");
    }

    #[test]
    fn test_no_weakening() {
        // A, B ⊢ A should NOT be provable (no weakening in linear logic)
        let mut prover = Prover::new(100);
        let seq = TwoSidedSequent::new(vec![atom("A"), atom("B")], vec![atom("A")]);
        let result = prover.prove_two_sided(&seq);
        assert!(result.is_none(), "A, B ⊢ A should NOT be provable");
    }

    #[test]
    fn test_one() {
        // ⊢ 1
        let mut prover = Prover::new(100);
        let seq = TwoSidedSequent::new(vec![], vec![Formula::One]);
        let result = prover.prove_two_sided(&seq);
        assert!(result.is_some(), "⊢ 1 should be provable");
    }

    #[test]
    fn test_top() {
        // Γ ⊢ ⊤ (always provable)
        let mut prover = Prover::new(100);
        let seq = TwoSidedSequent::new(vec![atom("A"), atom("B")], vec![Formula::Top]);
        let result = prover.prove_two_sided(&seq);
        assert!(result.is_some(), "A, B ⊢ ⊤ should be provable");
    }

    #[test]
    fn test_lolli() {
        // A ⊢ A (identity via lolli)
        // Actually test: ⊢ A ⊸ A
        let mut prover = Prover::new(100);
        let seq = TwoSidedSequent::new(
            vec![],
            vec![Formula::lolli(atom("A"), atom("A"))],
        );
        let result = prover.prove_two_sided(&seq);
        assert!(result.is_some(), "⊢ A ⊸ A should be provable");
    }

    #[test]
    fn test_all_splits() {
        let items = vec![1, 2];
        let splits = all_splits(&items);
        assert_eq!(splits.len(), 4); // 2^2 = 4 ways to split 2 items
    }
}
