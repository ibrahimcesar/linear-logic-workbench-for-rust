//! Term normalization and reduction.
//!
//! This module provides normalization (beta reduction) for linear lambda terms.
//! Since terms are linear, reduction is strongly normalizing.

use lolli_core::Term;

/// Perform one step of reduction, if possible.
///
/// Returns `Some(reduced)` if a reduction was performed, `None` if the term is normal.
///
/// # Example
///
/// ```
/// use lolli_extract::step;
/// use lolli_core::Term;
///
/// let t = Term::App(
///     Box::new(Term::Abs("x".to_string(), Box::new(Term::Var("x".to_string())))),
///     Box::new(Term::Unit),
/// );
///
/// let reduced = step(&t);
/// assert_eq!(reduced, Some(Term::Unit));
/// ```
pub fn step(term: &Term) -> Option<Term> {
    match term {
        // Beta reduction: (λx. e) v → e[v/x]
        Term::App(f, arg) => {
            if let Term::Abs(x, body) = f.as_ref() {
                Some(body.substitute(x, arg))
            } else {
                // Try to reduce the function
                if let Some(f_reduced) = step(f) {
                    Some(Term::App(Box::new(f_reduced), arg.clone()))
                } else if let Some(arg_reduced) = step(arg) {
                    Some(Term::App(f.clone(), Box::new(arg_reduced)))
                } else {
                    None
                }
            }
        }

        // Let-pair reduction: let (x, y) = (a, b) in e → e[a/x][b/y]
        Term::LetPair(x, y, pair, body) => {
            if let Term::Pair(a, b) = pair.as_ref() {
                let substituted = body.substitute(x, a).substitute(y, b);
                Some(substituted)
            } else if let Some(pair_reduced) = step(pair) {
                Some(Term::LetPair(
                    x.clone(),
                    y.clone(),
                    Box::new(pair_reduced),
                    body.clone(),
                ))
            } else if let Some(body_reduced) = step(body) {
                Some(Term::LetPair(
                    x.clone(),
                    y.clone(),
                    pair.clone(),
                    Box::new(body_reduced),
                ))
            } else {
                None
            }
        }

        // Case reduction: case inl v of { inl x => e1 | inr y => e2 } → e1[v/x]
        Term::Case(scrut, x, left, y, right) => {
            match scrut.as_ref() {
                Term::Inl(v) => Some(left.substitute(x, v)),
                Term::Inr(v) => Some(right.substitute(y, v)),
                _ => {
                    if let Some(scrut_reduced) = step(scrut) {
                        Some(Term::Case(
                            Box::new(scrut_reduced),
                            x.clone(),
                            left.clone(),
                            y.clone(),
                            right.clone(),
                        ))
                    } else {
                        None
                    }
                }
            }
        }

        // Fst reduction: fst (a, b) → a
        Term::Fst(pair) => {
            if let Term::Pair(a, _) = pair.as_ref() {
                Some(a.as_ref().clone())
            } else if let Some(pair_reduced) = step(pair) {
                Some(Term::Fst(Box::new(pair_reduced)))
            } else {
                None
            }
        }

        // Snd reduction: snd (a, b) → b
        Term::Snd(pair) => {
            if let Term::Pair(_, b) = pair.as_ref() {
                Some(b.as_ref().clone())
            } else if let Some(pair_reduced) = step(pair) {
                Some(Term::Snd(Box::new(pair_reduced)))
            } else {
                None
            }
        }

        // Dereliction: derelict (!v) → v
        Term::Derelict(e) => {
            if let Term::Promote(v) = e.as_ref() {
                Some(v.as_ref().clone())
            } else if let Some(e_reduced) = step(e) {
                Some(Term::Derelict(Box::new(e_reduced)))
            } else {
                None
            }
        }

        // Copy reduction: copy !v as (x, y) in e → e[!v/x][!v/y]
        Term::Copy(src, x, y, body) => {
            if let Term::Promote(v) = src.as_ref() {
                let promoted = Term::Promote(v.clone());
                let substituted = body.substitute(x, &promoted).substitute(y, &promoted);
                Some(substituted)
            } else if let Some(src_reduced) = step(src) {
                Some(Term::Copy(
                    Box::new(src_reduced),
                    x.clone(),
                    y.clone(),
                    body.clone(),
                ))
            } else {
                None
            }
        }

        // Discard reduction: discard !v in e → e
        Term::Discard(discarded, body) => {
            if matches!(discarded.as_ref(), Term::Promote(_)) {
                Some(body.as_ref().clone())
            } else if let Some(discarded_reduced) = step(discarded) {
                Some(Term::Discard(Box::new(discarded_reduced), body.clone()))
            } else if let Some(body_reduced) = step(body) {
                Some(Term::Discard(discarded.clone(), Box::new(body_reduced)))
            } else {
                None
            }
        }

        // Reduce inside abstractions
        Term::Abs(x, body) => {
            step(body).map(|reduced| Term::Abs(x.clone(), Box::new(reduced)))
        }

        // Reduce inside pairs
        Term::Pair(a, b) => {
            if let Some(a_reduced) = step(a) {
                Some(Term::Pair(Box::new(a_reduced), b.clone()))
            } else {
                step(b).map(|b_reduced| Term::Pair(a.clone(), Box::new(b_reduced)))
            }
        }

        // Reduce inside injections
        Term::Inl(e) => step(e).map(|reduced| Term::Inl(Box::new(reduced))),
        Term::Inr(e) => step(e).map(|reduced| Term::Inr(Box::new(reduced))),

        // Reduce inside promote
        Term::Promote(e) => step(e).map(|reduced| Term::Promote(Box::new(reduced))),

        // Abort and values are already normal
        Term::Var(_) | Term::Unit | Term::Trivial | Term::Abort(_) => None,
    }
}

/// Fully normalize a term to its normal form.
///
/// Since linear lambda calculus is strongly normalizing, this always terminates.
///
/// # Example
///
/// ```
/// use lolli_extract::normalize;
/// use lolli_core::Term;
///
/// let t = Term::App(
///     Box::new(Term::Abs("x".to_string(), Box::new(Term::Var("x".to_string())))),
///     Box::new(Term::Unit),
/// );
///
/// let normal = normalize(&t);
/// assert_eq!(normal, Term::Unit);
/// ```
pub fn normalize(term: &Term) -> Term {
    let mut current = term.clone();
    while let Some(reduced) = step(&current) {
        current = reduced;
    }
    current
}

/// Normalize with a maximum step count to prevent infinite loops.
///
/// Returns the term after at most `max_steps` reduction steps.
pub fn normalize_bounded(term: &Term, max_steps: usize) -> Term {
    let mut current = term.clone();
    for _ in 0..max_steps {
        if let Some(reduced) = step(&current) {
            current = reduced;
        } else {
            break;
        }
    }
    current
}

/// Check if a term is in normal form.
pub fn is_normal(term: &Term) -> bool {
    step(term).is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beta_reduction() {
        // (λx. x) () → ()
        let t = Term::App(
            Box::new(Term::Abs(
                "x".to_string(),
                Box::new(Term::Var("x".to_string())),
            )),
            Box::new(Term::Unit),
        );

        let result = normalize(&t);
        assert_eq!(result, Term::Unit);
    }

    #[test]
    fn test_let_pair_reduction() {
        // let (x, y) = ((), ⟨⟩) in x → ()
        let t = Term::LetPair(
            "x".to_string(),
            "y".to_string(),
            Box::new(Term::Pair(Box::new(Term::Unit), Box::new(Term::Trivial))),
            Box::new(Term::Var("x".to_string())),
        );

        let result = normalize(&t);
        assert_eq!(result, Term::Unit);
    }

    #[test]
    fn test_case_inl_reduction() {
        // case inl () of { inl x => x | inr y => y } → ()
        let t = Term::Case(
            Box::new(Term::Inl(Box::new(Term::Unit))),
            "x".to_string(),
            Box::new(Term::Var("x".to_string())),
            "y".to_string(),
            Box::new(Term::Var("y".to_string())),
        );

        let result = normalize(&t);
        assert_eq!(result, Term::Unit);
    }

    #[test]
    fn test_case_inr_reduction() {
        // case inr ⟨⟩ of { inl x => x | inr y => y } → ⟨⟩
        let t = Term::Case(
            Box::new(Term::Inr(Box::new(Term::Trivial))),
            "x".to_string(),
            Box::new(Term::Var("x".to_string())),
            "y".to_string(),
            Box::new(Term::Var("y".to_string())),
        );

        let result = normalize(&t);
        assert_eq!(result, Term::Trivial);
    }

    #[test]
    fn test_fst_reduction() {
        // fst ((), ⟨⟩) → ()
        let t = Term::Fst(Box::new(Term::Pair(
            Box::new(Term::Unit),
            Box::new(Term::Trivial),
        )));

        let result = normalize(&t);
        assert_eq!(result, Term::Unit);
    }

    #[test]
    fn test_snd_reduction() {
        // snd ((), ⟨⟩) → ⟨⟩
        let t = Term::Snd(Box::new(Term::Pair(
            Box::new(Term::Unit),
            Box::new(Term::Trivial),
        )));

        let result = normalize(&t);
        assert_eq!(result, Term::Trivial);
    }

    #[test]
    fn test_dereliction_reduction() {
        // derelict (!()) → ()
        let t = Term::Derelict(Box::new(Term::Promote(Box::new(Term::Unit))));

        let result = normalize(&t);
        assert_eq!(result, Term::Unit);
    }

    #[test]
    fn test_copy_reduction() {
        // copy !() as (x, y) in (x, y) → (!(), !())
        let t = Term::Copy(
            Box::new(Term::Promote(Box::new(Term::Unit))),
            "x".to_string(),
            "y".to_string(),
            Box::new(Term::Pair(
                Box::new(Term::Var("x".to_string())),
                Box::new(Term::Var("y".to_string())),
            )),
        );

        let result = normalize(&t);
        assert_eq!(
            result,
            Term::Pair(
                Box::new(Term::Promote(Box::new(Term::Unit))),
                Box::new(Term::Promote(Box::new(Term::Unit))),
            )
        );
    }

    #[test]
    fn test_discard_reduction() {
        // discard !() in ⟨⟩ → ⟨⟩
        let t = Term::Discard(
            Box::new(Term::Promote(Box::new(Term::Unit))),
            Box::new(Term::Trivial),
        );

        let result = normalize(&t);
        assert_eq!(result, Term::Trivial);
    }

    #[test]
    fn test_is_normal() {
        assert!(is_normal(&Term::Unit));
        assert!(is_normal(&Term::Var("x".to_string())));
        assert!(is_normal(&Term::Abs(
            "x".to_string(),
            Box::new(Term::Var("x".to_string()))
        )));

        // Redex is not normal
        let redex = Term::App(
            Box::new(Term::Abs(
                "x".to_string(),
                Box::new(Term::Var("x".to_string())),
            )),
            Box::new(Term::Unit),
        );
        assert!(!is_normal(&redex));
    }

    #[test]
    fn test_nested_reduction() {
        // (λf. f ()) (λx. x) → ()
        let t = Term::App(
            Box::new(Term::Abs(
                "f".to_string(),
                Box::new(Term::App(
                    Box::new(Term::Var("f".to_string())),
                    Box::new(Term::Unit),
                )),
            )),
            Box::new(Term::Abs(
                "x".to_string(),
                Box::new(Term::Var("x".to_string())),
            )),
        );

        let result = normalize(&t);
        assert_eq!(result, Term::Unit);
    }

    #[test]
    fn test_normalize_bounded() {
        // Test that bounded normalization respects the limit
        let t = Term::App(
            Box::new(Term::Abs(
                "x".to_string(),
                Box::new(Term::Var("x".to_string())),
            )),
            Box::new(Term::Unit),
        );

        // With 0 steps, should return the original term
        let result0 = normalize_bounded(&t, 0);
        assert_eq!(result0, t);

        // With enough steps, should fully normalize
        let result_full = normalize_bounded(&t, 10);
        assert_eq!(result_full, Term::Unit);
    }
}
