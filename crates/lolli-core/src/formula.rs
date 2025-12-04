//! Linear logic formula representation.
//!
//! This module provides the [`Formula`] enum representing linear logic formulas
//! with all standard connectives.

/// A linear logic formula.
///
/// Linear logic has a rich set of connectives split into multiplicative and additive families,
/// plus exponentials for controlled structural rules.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Formula {
    // Atoms
    /// Atomic proposition
    Atom(String),
    /// Negated atomic proposition (A⊥)
    NegAtom(String),

    // Multiplicatives
    /// Tensor product (A ⊗ B) - "both A and B independently"
    Tensor(Box<Formula>, Box<Formula>),
    /// Par (A ⅋ B) - "A or B, opponent chooses"
    Par(Box<Formula>, Box<Formula>),
    /// Multiplicative unit (1)
    One,
    /// Multiplicative false (⊥)
    Bottom,

    // Additives
    /// With (A & B) - "both available, you choose one"
    With(Box<Formula>, Box<Formula>),
    /// Plus (A ⊕ B) - "one of them, I choose which"
    Plus(Box<Formula>, Box<Formula>),
    /// Additive truth (⊤)
    Top,
    /// Additive false (0)
    Zero,

    // Exponentials
    /// Of course (!A) - "unlimited supply of A, can copy/discard"
    OfCourse(Box<Formula>),
    /// Why not (?A) - "demand for A"
    WhyNot(Box<Formula>),

    // Derived (syntactic sugar)
    /// Linear implication (A ⊸ B) - sugar for A⊥ ⅋ B
    Lolli(Box<Formula>, Box<Formula>),
}

impl Formula {
    /// Compute the linear negation of a formula.
    ///
    /// Linear negation is involutive: (A⊥)⊥ = A
    ///
    /// De Morgan dualities:
    /// - (A ⊗ B)⊥ = A⊥ ⅋ B⊥
    /// - (A ⅋ B)⊥ = A⊥ ⊗ B⊥
    /// - (A & B)⊥ = A⊥ ⊕ B⊥
    /// - (A ⊕ B)⊥ = A⊥ & B⊥
    /// - 1⊥ = ⊥
    /// - ⊥⊥ = 1
    /// - (!A)⊥ = ?(A⊥)
    /// - (?A)⊥ = !(A⊥)
    pub fn negate(&self) -> Formula {
        match self {
            Formula::Atom(a) => Formula::NegAtom(a.clone()),
            Formula::NegAtom(a) => Formula::Atom(a.clone()),

            Formula::Tensor(a, b) => {
                Formula::Par(Box::new(a.negate()), Box::new(b.negate()))
            }
            Formula::Par(a, b) => {
                Formula::Tensor(Box::new(a.negate()), Box::new(b.negate()))
            }
            Formula::One => Formula::Bottom,
            Formula::Bottom => Formula::One,

            Formula::With(a, b) => {
                Formula::Plus(Box::new(a.negate()), Box::new(b.negate()))
            }
            Formula::Plus(a, b) => {
                Formula::With(Box::new(a.negate()), Box::new(b.negate()))
            }
            Formula::Top => Formula::Zero,
            Formula::Zero => Formula::Top,

            Formula::OfCourse(a) => Formula::WhyNot(Box::new(a.negate())),
            Formula::WhyNot(a) => Formula::OfCourse(Box::new(a.negate())),

            Formula::Lolli(a, b) => {
                // (A ⊸ B)⊥ = (A⊥ ⅋ B)⊥ = A ⊗ B⊥
                Formula::Tensor(a.clone(), Box::new(b.negate()))
            }
        }
    }

    /// Desugar the formula by expanding A ⊸ B to A⊥ ⅋ B.
    pub fn desugar(&self) -> Formula {
        match self {
            Formula::Lolli(a, b) => {
                Formula::Par(Box::new(a.negate().desugar()), Box::new(b.desugar()))
            }
            Formula::Tensor(a, b) => {
                Formula::Tensor(Box::new(a.desugar()), Box::new(b.desugar()))
            }
            Formula::Par(a, b) => {
                Formula::Par(Box::new(a.desugar()), Box::new(b.desugar()))
            }
            Formula::With(a, b) => {
                Formula::With(Box::new(a.desugar()), Box::new(b.desugar()))
            }
            Formula::Plus(a, b) => {
                Formula::Plus(Box::new(a.desugar()), Box::new(b.desugar()))
            }
            Formula::OfCourse(a) => Formula::OfCourse(Box::new(a.desugar())),
            Formula::WhyNot(a) => Formula::WhyNot(Box::new(a.desugar())),
            _ => self.clone(),
        }
    }

    /// Returns true if this formula is positive (async/eager).
    ///
    /// Positive formulas: ⊗, 1, ⊕, 0, !, atoms
    pub fn is_positive(&self) -> bool {
        matches!(
            self,
            Formula::Atom(_)
                | Formula::Tensor(_, _)
                | Formula::One
                | Formula::Plus(_, _)
                | Formula::Zero
                | Formula::OfCourse(_)
        )
    }

    /// Returns true if this formula is negative (sync/lazy).
    ///
    /// Negative formulas: ⅋, ⊥, &, ⊤, ?, negated atoms
    pub fn is_negative(&self) -> bool {
        !self.is_positive()
    }

    /// Pretty print the formula with Unicode symbols.
    pub fn pretty(&self) -> String {
        match self {
            Formula::Atom(a) => a.clone(),
            Formula::NegAtom(a) => format!("{}⊥", a),
            Formula::Tensor(a, b) => format!("({} ⊗ {})", a.pretty(), b.pretty()),
            Formula::Par(a, b) => format!("({} ⅋ {})", a.pretty(), b.pretty()),
            Formula::Lolli(a, b) => format!("({} ⊸ {})", a.pretty(), b.pretty()),
            Formula::With(a, b) => format!("({} & {})", a.pretty(), b.pretty()),
            Formula::Plus(a, b) => format!("({} ⊕ {})", a.pretty(), b.pretty()),
            Formula::OfCourse(a) => format!("!{}", a.pretty()),
            Formula::WhyNot(a) => format!("?{}", a.pretty()),
            Formula::One => "1".to_string(),
            Formula::Bottom => "⊥".to_string(),
            Formula::Top => "⊤".to_string(),
            Formula::Zero => "0".to_string(),
        }
    }

    /// Pretty print the formula with ASCII symbols.
    pub fn pretty_ascii(&self) -> String {
        match self {
            Formula::Atom(a) => a.clone(),
            Formula::NegAtom(a) => format!("{}^", a),
            Formula::Tensor(a, b) => format!("({} * {})", a.pretty_ascii(), b.pretty_ascii()),
            Formula::Par(a, b) => format!("({} | {})", a.pretty_ascii(), b.pretty_ascii()),
            Formula::Lolli(a, b) => format!("({} -o {})", a.pretty_ascii(), b.pretty_ascii()),
            Formula::With(a, b) => format!("({} & {})", a.pretty_ascii(), b.pretty_ascii()),
            Formula::Plus(a, b) => format!("({} + {})", a.pretty_ascii(), b.pretty_ascii()),
            Formula::OfCourse(a) => format!("!{}", a.pretty_ascii()),
            Formula::WhyNot(a) => format!("?{}", a.pretty_ascii()),
            Formula::One => "1".to_string(),
            Formula::Bottom => "bot".to_string(),
            Formula::Top => "top".to_string(),
            Formula::Zero => "0".to_string(),
        }
    }

    /// Pretty print the formula for LaTeX.
    pub fn pretty_latex(&self) -> String {
        match self {
            Formula::Atom(a) => a.clone(),
            Formula::NegAtom(a) => format!("{}^{{\\bot}}", a),
            Formula::Tensor(a, b) => {
                format!("({} \\otimes {})", a.pretty_latex(), b.pretty_latex())
            }
            Formula::Par(a, b) => {
                format!("({} \\parr {})", a.pretty_latex(), b.pretty_latex())
            }
            Formula::Lolli(a, b) => {
                format!("({} \\multimap {})", a.pretty_latex(), b.pretty_latex())
            }
            Formula::With(a, b) => {
                format!("({} \\with {})", a.pretty_latex(), b.pretty_latex())
            }
            Formula::Plus(a, b) => {
                format!("({} \\oplus {})", a.pretty_latex(), b.pretty_latex())
            }
            Formula::OfCourse(a) => format!("{{!}}{}", a.pretty_latex()),
            Formula::WhyNot(a) => format!("{{?}}{}", a.pretty_latex()),
            Formula::One => "\\mathbf{1}".to_string(),
            Formula::Bottom => "\\bot".to_string(),
            Formula::Top => "\\top".to_string(),
            Formula::Zero => "\\mathbf{0}".to_string(),
        }
    }

    /// Create an atom formula.
    pub fn atom(name: impl Into<String>) -> Self {
        Formula::Atom(name.into())
    }

    /// Create a negated atom formula.
    pub fn neg_atom(name: impl Into<String>) -> Self {
        Formula::NegAtom(name.into())
    }

    /// Create a tensor (A ⊗ B).
    pub fn tensor(a: Formula, b: Formula) -> Self {
        Formula::Tensor(Box::new(a), Box::new(b))
    }

    /// Create a par (A ⅋ B).
    pub fn par(a: Formula, b: Formula) -> Self {
        Formula::Par(Box::new(a), Box::new(b))
    }

    /// Create a linear implication (A ⊸ B).
    pub fn lolli(a: Formula, b: Formula) -> Self {
        Formula::Lolli(Box::new(a), Box::new(b))
    }

    /// Create a with (A & B).
    pub fn with(a: Formula, b: Formula) -> Self {
        Formula::With(Box::new(a), Box::new(b))
    }

    /// Create a plus (A ⊕ B).
    pub fn plus(a: Formula, b: Formula) -> Self {
        Formula::Plus(Box::new(a), Box::new(b))
    }

    /// Create an of-course (!A).
    pub fn of_course(a: Formula) -> Self {
        Formula::OfCourse(Box::new(a))
    }

    /// Create a why-not (?A).
    pub fn why_not(a: Formula) -> Self {
        Formula::WhyNot(Box::new(a))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negation_involutive() {
        let a = Formula::atom("A");
        assert_eq!(a.negate().negate(), a);

        let complex = Formula::tensor(Formula::atom("A"), Formula::atom("B"));
        assert_eq!(complex.negate().negate(), complex);

        // Test primitive connectives (not lolli, which is sugar)
        let formulas = vec![
            Formula::One,
            Formula::Bottom,
            Formula::Top,
            Formula::Zero,
            Formula::of_course(Formula::atom("X")),
            Formula::why_not(Formula::atom("Y")),
            Formula::with(Formula::atom("A"), Formula::atom("B")),
            Formula::plus(Formula::atom("A"), Formula::atom("B")),
            Formula::par(Formula::atom("A"), Formula::atom("B")),
        ];

        for f in formulas {
            assert_eq!(f.negate().negate(), f, "Failed for: {:?}", f);
        }

        // Lolli is special: (A ⊸ B)⊥ = A ⊗ B⊥, which is not a Lolli
        // But we can verify the semantics are correct
        let lolli = Formula::lolli(Formula::atom("A"), Formula::atom("B"));
        let neg_lolli = lolli.negate();
        // (A ⊸ B)⊥ = A ⊗ B⊥
        assert_eq!(
            neg_lolli,
            Formula::tensor(Formula::atom("A"), Formula::neg_atom("B"))
        );
    }

    #[test]
    fn test_de_morgan() {
        // (A ⊗ B)⊥ = A⊥ ⅋ B⊥
        let tensor = Formula::tensor(Formula::atom("A"), Formula::atom("B"));
        let expected = Formula::par(Formula::neg_atom("A"), Formula::neg_atom("B"));
        assert_eq!(tensor.negate(), expected);

        // (A ⅋ B)⊥ = A⊥ ⊗ B⊥
        let par = Formula::par(Formula::atom("A"), Formula::atom("B"));
        let expected = Formula::tensor(Formula::neg_atom("A"), Formula::neg_atom("B"));
        assert_eq!(par.negate(), expected);

        // (A & B)⊥ = A⊥ ⊕ B⊥
        let with = Formula::with(Formula::atom("A"), Formula::atom("B"));
        let expected = Formula::plus(Formula::neg_atom("A"), Formula::neg_atom("B"));
        assert_eq!(with.negate(), expected);

        // (A ⊕ B)⊥ = A⊥ & B⊥
        let plus = Formula::plus(Formula::atom("A"), Formula::atom("B"));
        let expected = Formula::with(Formula::neg_atom("A"), Formula::neg_atom("B"));
        assert_eq!(plus.negate(), expected);

        // 1⊥ = ⊥
        assert_eq!(Formula::One.negate(), Formula::Bottom);

        // ⊥⊥ = 1
        assert_eq!(Formula::Bottom.negate(), Formula::One);

        // ⊤⊥ = 0
        assert_eq!(Formula::Top.negate(), Formula::Zero);

        // 0⊥ = ⊤
        assert_eq!(Formula::Zero.negate(), Formula::Top);

        // (!A)⊥ = ?(A⊥)
        let bang = Formula::of_course(Formula::atom("A"));
        let expected = Formula::why_not(Formula::neg_atom("A"));
        assert_eq!(bang.negate(), expected);

        // (?A)⊥ = !(A⊥)
        let whynot = Formula::why_not(Formula::atom("A"));
        let expected = Formula::of_course(Formula::neg_atom("A"));
        assert_eq!(whynot.negate(), expected);
    }

    #[test]
    fn test_polarity() {
        // Positive formulas
        assert!(Formula::atom("A").is_positive());
        assert!(Formula::tensor(Formula::atom("A"), Formula::atom("B")).is_positive());
        assert!(Formula::One.is_positive());
        assert!(Formula::plus(Formula::atom("A"), Formula::atom("B")).is_positive());
        assert!(Formula::Zero.is_positive());
        assert!(Formula::of_course(Formula::atom("A")).is_positive());

        // Negative formulas
        assert!(Formula::neg_atom("A").is_negative());
        assert!(Formula::par(Formula::atom("A"), Formula::atom("B")).is_negative());
        assert!(Formula::Bottom.is_negative());
        assert!(Formula::with(Formula::atom("A"), Formula::atom("B")).is_negative());
        assert!(Formula::Top.is_negative());
        assert!(Formula::why_not(Formula::atom("A")).is_negative());

        // Lolli is negative (it's sugar for A⊥ ⅋ B)
        assert!(Formula::lolli(Formula::atom("A"), Formula::atom("B")).is_negative());
    }

    #[test]
    fn test_desugar() {
        // A ⊸ B should desugar to A⊥ ⅋ B
        let lolli = Formula::lolli(Formula::atom("A"), Formula::atom("B"));
        let desugared = lolli.desugar();
        let expected = Formula::par(Formula::neg_atom("A"), Formula::atom("B"));
        assert_eq!(desugared, expected);

        // Nested: (A ⊸ B) ⊸ C should desugar correctly
        let nested = Formula::lolli(
            Formula::lolli(Formula::atom("A"), Formula::atom("B")),
            Formula::atom("C"),
        );
        let desugared = nested.desugar();
        // (A ⊸ B)⊥ ⅋ C = (A ⊗ B⊥) ⅋ C
        let expected = Formula::par(
            Formula::tensor(Formula::atom("A"), Formula::neg_atom("B")),
            Formula::atom("C"),
        );
        assert_eq!(desugared, expected);
    }

    #[test]
    fn test_pretty_print() {
        let f = Formula::lolli(Formula::atom("A"), Formula::atom("B"));
        assert_eq!(f.pretty(), "(A ⊸ B)");
        assert_eq!(f.pretty_ascii(), "(A -o B)");
        assert_eq!(f.pretty_latex(), "(A \\multimap B)");

        let f = Formula::tensor(
            Formula::of_course(Formula::atom("A")),
            Formula::why_not(Formula::atom("B")),
        );
        assert_eq!(f.pretty(), "(!A ⊗ ?B)");
        assert_eq!(f.pretty_ascii(), "(!A * ?B)");
    }

    #[test]
    fn test_units() {
        assert_eq!(Formula::One.pretty(), "1");
        assert_eq!(Formula::Bottom.pretty(), "⊥");
        assert_eq!(Formula::Top.pretty(), "⊤");
        assert_eq!(Formula::Zero.pretty(), "0");

        assert_eq!(Formula::One.pretty_ascii(), "1");
        assert_eq!(Formula::Bottom.pretty_ascii(), "bot");
        assert_eq!(Formula::Top.pretty_ascii(), "top");
        assert_eq!(Formula::Zero.pretty_ascii(), "0");
    }
}
