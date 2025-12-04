//! # lolli-parse
//!
//! Parser for the Lolli linear logic workbench.
//!
//! This crate provides parsing functionality for linear logic formulas and sequents.
//!
//! ## Supported Syntax
//!
//! | Connective | Unicode | ASCII |
//! |------------|---------|-------|
//! | Tensor | ⊗ | * |
//! | Par | ⅋ | \`|\` |
//! | Lolli | ⊸ | -o |
//! | With | & | & |
//! | Plus | ⊕ | + |
//! | Bang | ! | ! |
//! | Why not | ? | ? |
//! | Negation | A⊥ | A^ |
//! | Turnstile | ⊢ | \|- |
//!
//! ## Example
//!
//! ```
//! use lolli_parse::{parse_formula, parse_sequent};
//!
//! let formula = parse_formula("A -o B").unwrap();
//! assert_eq!(formula.pretty(), "(A ⊸ B)");
//!
//! let sequent = parse_sequent("A, B |- A * B").unwrap();
//! assert_eq!(sequent.antecedent.len(), 2);
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

use pest::Parser;
use pest_derive::Parser;

pub use lolli_core::{Formula, Sequent, TwoSidedSequent};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct LolliParser;

/// Parse error type.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// Unexpected token in input
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String),

    /// Unknown operator
    #[error("Unknown operator: {0}")]
    UnknownOperator(String),

    /// Unexpected rule during parsing
    #[error("Unexpected rule: {0}")]
    UnexpectedRule(String),

    /// Pest parsing error
    #[error("Parse error: {0}")]
    PestError(#[from] pest::error::Error<Rule>),

    /// Empty input
    #[error("Empty input")]
    EmptyInput,
}

/// Parse a formula from a string.
///
/// # Examples
///
/// ```
/// use lolli_parse::parse_formula;
///
/// // Simple atom
/// let f = parse_formula("A").unwrap();
///
/// // Linear implication
/// let f = parse_formula("A -o B").unwrap();
///
/// // Complex formula
/// let f = parse_formula("!A * B -o C + D").unwrap();
/// ```
///
/// # Errors
///
/// Returns a `ParseError` if the input is not a valid formula.
pub fn parse_formula(input: &str) -> Result<Formula, ParseError> {
    let pairs = LolliParser::parse(Rule::formula, input)?;
    let pair = pairs.into_iter().next().ok_or(ParseError::EmptyInput)?;
    build_formula(pair)
}

/// Parse a sequent from a string.
///
/// # Examples
///
/// ```
/// use lolli_parse::parse_sequent;
///
/// // Two-sided sequent
/// let s = parse_sequent("A, B |- C").unwrap();
///
/// // One-sided sequent (right side only)
/// let s = parse_sequent("|- A, B").unwrap();
/// ```
///
/// # Errors
///
/// Returns a `ParseError` if the input is not a valid sequent.
pub fn parse_sequent(input: &str) -> Result<TwoSidedSequent, ParseError> {
    let pairs = LolliParser::parse(Rule::sequent, input)?;
    let pair = pairs.into_iter().next().ok_or(ParseError::EmptyInput)?;
    build_sequent(pair)
}

use pest::iterators::Pair;

fn build_formula(pair: Pair<Rule>) -> Result<Formula, ParseError> {
    match pair.as_rule() {
        Rule::formula => {
            let inner = pair.into_inner().next().ok_or(ParseError::EmptyInput)?;
            build_formula(inner)
        }
        Rule::lolli_expr => build_lolli_expr(pair),
        Rule::par_expr => build_left_assoc_binary(pair, Rule::par_op, Formula::par),
        Rule::tensor_expr => build_left_assoc_binary(pair, Rule::tensor_op, Formula::tensor),
        Rule::plus_expr => build_left_assoc_binary(pair, Rule::plus_op, Formula::plus),
        Rule::with_expr => build_left_assoc_binary(pair, Rule::with_op, Formula::with),
        Rule::unary_expr => build_unary_expr(pair),
        Rule::primary_expr => build_primary_expr(pair),
        Rule::ident => Ok(Formula::Atom(pair.as_str().to_string())),
        Rule::one => Ok(Formula::One),
        Rule::bottom => Ok(Formula::Bottom),
        Rule::top => Ok(Formula::Top),
        Rule::zero => Ok(Formula::Zero),
        _ => Err(ParseError::UnexpectedRule(format!("{:?}", pair.as_rule()))),
    }
}

fn build_lolli_expr(pair: Pair<Rule>) -> Result<Formula, ParseError> {
    let mut inner = pair.into_inner();
    let first = inner.next().ok_or(ParseError::EmptyInput)?;
    let mut result = build_formula(first)?;

    // Check for lolli operator and right side
    while let Some(op_or_expr) = inner.next() {
        if op_or_expr.as_rule() == Rule::lolli_op {
            // Get the right side (which is itself a lolli_expr for right associativity)
            let right = inner.next().ok_or(ParseError::EmptyInput)?;
            let right_formula = build_formula(right)?;
            result = Formula::lolli(result, right_formula);
        } else {
            // It's the right side directly
            let right_formula = build_formula(op_or_expr)?;
            result = Formula::lolli(result, right_formula);
        }
    }

    Ok(result)
}

fn build_left_assoc_binary<F>(
    pair: Pair<Rule>,
    _op_rule: Rule,
    constructor: F,
) -> Result<Formula, ParseError>
where
    F: Fn(Formula, Formula) -> Formula,
{
    let mut inner = pair.into_inner();
    let first = inner.next().ok_or(ParseError::EmptyInput)?;
    let mut result = build_formula(first)?;

    while let Some(next) = inner.next() {
        // Skip operator tokens
        if next.as_rule() == _op_rule {
            continue;
        }
        let right = build_formula(next)?;
        result = constructor(result, right);
    }

    Ok(result)
}

fn build_unary_expr(pair: Pair<Rule>) -> Result<Formula, ParseError> {
    let mut inner = pair.into_inner().peekable();

    // Check for prefix operators
    let first = inner.peek().ok_or(ParseError::EmptyInput)?;

    match first.as_rule() {
        Rule::bang_op => {
            inner.next(); // consume the operator
            let operand = inner.next().ok_or(ParseError::EmptyInput)?;
            let formula = build_formula(operand)?;
            Ok(Formula::of_course(formula))
        }
        Rule::whynot_op => {
            inner.next(); // consume the operator
            let operand = inner.next().ok_or(ParseError::EmptyInput)?;
            let formula = build_formula(operand)?;
            Ok(Formula::why_not(formula))
        }
        _ => {
            // Primary expression with optional negation suffix
            let primary = inner.next().ok_or(ParseError::EmptyInput)?;
            let mut formula = build_formula(primary)?;

            // Check for negation suffix
            if let Some(suffix) = inner.next() {
                if suffix.as_rule() == Rule::negation_suffix {
                    formula = formula.negate();
                }
            }

            Ok(formula)
        }
    }
}

fn build_primary_expr(pair: Pair<Rule>) -> Result<Formula, ParseError> {
    let inner = pair.into_inner().next().ok_or(ParseError::EmptyInput)?;
    build_formula(inner)
}

fn build_sequent(pair: Pair<Rule>) -> Result<TwoSidedSequent, ParseError> {
    let mut antecedent = Vec::new();
    let mut succedent = Vec::new();
    let mut seen_turnstile = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::turnstile => {
                seen_turnstile = true;
            }
            Rule::formula_list => {
                let formulas = build_formula_list(inner)?;
                if seen_turnstile {
                    succedent = formulas;
                } else {
                    antecedent = formulas;
                }
            }
            _ => {}
        }
    }

    Ok(TwoSidedSequent::new(antecedent, succedent))
}

fn build_formula_list(pair: Pair<Rule>) -> Result<Vec<Formula>, ParseError> {
    let mut formulas = Vec::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::formula {
            formulas.push(build_formula(inner)?);
        }
    }
    Ok(formulas)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_atoms() {
        let f = parse_formula("A").unwrap();
        assert_eq!(f, Formula::atom("A"));

        let f = parse_formula("foo").unwrap();
        assert_eq!(f, Formula::atom("foo"));

        let f = parse_formula("FileHandle").unwrap();
        assert_eq!(f, Formula::atom("FileHandle"));
    }

    #[test]
    fn test_parse_units() {
        assert_eq!(parse_formula("1").unwrap(), Formula::One);
        assert_eq!(parse_formula("one").unwrap(), Formula::One);
        assert_eq!(parse_formula("bot").unwrap(), Formula::Bottom);
        assert_eq!(parse_formula("bottom").unwrap(), Formula::Bottom);
        assert_eq!(parse_formula("top").unwrap(), Formula::Top);
        assert_eq!(parse_formula("0").unwrap(), Formula::Zero);
        assert_eq!(parse_formula("zero").unwrap(), Formula::Zero);
    }

    #[test]
    fn test_parse_negation() {
        let f = parse_formula("A^").unwrap();
        assert_eq!(f, Formula::neg_atom("A"));

        let f = parse_formula("A⊥").unwrap();
        assert_eq!(f, Formula::neg_atom("A"));
    }

    #[test]
    fn test_parse_unary() {
        let f = parse_formula("!A").unwrap();
        assert_eq!(f, Formula::of_course(Formula::atom("A")));

        let f = parse_formula("?A").unwrap();
        assert_eq!(f, Formula::why_not(Formula::atom("A")));

        let f = parse_formula("!!A").unwrap();
        assert_eq!(
            f,
            Formula::of_course(Formula::of_course(Formula::atom("A")))
        );
    }

    #[test]
    fn test_parse_lolli() {
        let f = parse_formula("A -o B").unwrap();
        assert_eq!(f, Formula::lolli(Formula::atom("A"), Formula::atom("B")));

        // Right associativity: A -o B -o C = A -o (B -o C)
        let f = parse_formula("A -o B -o C").unwrap();
        assert_eq!(
            f,
            Formula::lolli(
                Formula::atom("A"),
                Formula::lolli(Formula::atom("B"), Formula::atom("C"))
            )
        );
    }

    #[test]
    fn test_parse_tensor() {
        let f = parse_formula("A * B").unwrap();
        assert_eq!(f, Formula::tensor(Formula::atom("A"), Formula::atom("B")));

        // Left associativity: A * B * C = (A * B) * C
        let f = parse_formula("A * B * C").unwrap();
        assert_eq!(
            f,
            Formula::tensor(
                Formula::tensor(Formula::atom("A"), Formula::atom("B")),
                Formula::atom("C")
            )
        );
    }

    #[test]
    fn test_parse_plus() {
        let f = parse_formula("A + B").unwrap();
        assert_eq!(f, Formula::plus(Formula::atom("A"), Formula::atom("B")));
    }

    #[test]
    fn test_parse_with() {
        let f = parse_formula("A & B").unwrap();
        assert_eq!(f, Formula::with(Formula::atom("A"), Formula::atom("B")));
    }

    #[test]
    fn test_parse_parentheses() {
        let f = parse_formula("(A -o B)").unwrap();
        assert_eq!(f, Formula::lolli(Formula::atom("A"), Formula::atom("B")));

        // Override precedence
        let f = parse_formula("(A + B) * C").unwrap();
        assert_eq!(
            f,
            Formula::tensor(
                Formula::plus(Formula::atom("A"), Formula::atom("B")),
                Formula::atom("C")
            )
        );
    }

    #[test]
    fn test_parse_precedence() {
        // * binds tighter than -o
        // A * B -o C = (A * B) -o C
        let f = parse_formula("A * B -o C").unwrap();
        assert_eq!(
            f,
            Formula::lolli(
                Formula::tensor(Formula::atom("A"), Formula::atom("B")),
                Formula::atom("C")
            )
        );

        // In our grammar (following the hierarchy):
        // lolli < par < tensor < plus < with < unary
        // So * binds tighter than +
        // A + B * C = (A + B) * C
        let f = parse_formula("A + B * C").unwrap();
        assert_eq!(
            f,
            Formula::tensor(
                Formula::plus(Formula::atom("A"), Formula::atom("B")),
                Formula::atom("C")
            )
        );

        // & binds tightest
        // A & B + C = (A & B) + C
        let f = parse_formula("A & B + C").unwrap();
        assert_eq!(
            f,
            Formula::plus(
                Formula::with(Formula::atom("A"), Formula::atom("B")),
                Formula::atom("C")
            )
        );
    }

    #[test]
    fn test_parse_complex() {
        let f = parse_formula("!A * B -o ?C + D").unwrap();
        // !A * B = tensor(!A, B)
        // ?C + D = plus(?C, D)
        // result = lolli(tensor(!A, B), plus(?C, D))
        assert_eq!(
            f,
            Formula::lolli(
                Formula::tensor(
                    Formula::of_course(Formula::atom("A")),
                    Formula::atom("B")
                ),
                Formula::plus(Formula::why_not(Formula::atom("C")), Formula::atom("D"))
            )
        );
    }

    #[test]
    fn test_parse_sequent() {
        let s = parse_sequent("A, B |- C").unwrap();
        assert_eq!(s.antecedent.len(), 2);
        assert_eq!(s.succedent.len(), 1);
        assert_eq!(s.antecedent[0], Formula::atom("A"));
        assert_eq!(s.antecedent[1], Formula::atom("B"));
        assert_eq!(s.succedent[0], Formula::atom("C"));
    }

    #[test]
    fn test_parse_sequent_one_sided() {
        let s = parse_sequent("|- A, B").unwrap();
        assert_eq!(s.antecedent.len(), 0);
        assert_eq!(s.succedent.len(), 2);
    }

    #[test]
    fn test_parse_sequent_complex() {
        let s = parse_sequent("A * B, C -o D |- E + F").unwrap();
        assert_eq!(s.antecedent.len(), 2);
        assert_eq!(s.succedent.len(), 1);
    }

    #[test]
    fn test_roundtrip() {
        let formulas = vec![
            "A",
            "A -o B",
            "A * B",
            "A + B",
            "A & B",
            "!A",
            "?A",
            "1",
            "bot",
            "top",
            "0",
            "A * B -o C",
            "!A * ?B -o C + D",
        ];

        for input in formulas {
            let f = parse_formula(input).unwrap();
            // Just verify it parses without error
            let _ = f.pretty();
        }
    }
}
