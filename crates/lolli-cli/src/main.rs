//! Lolli - Linear Logic Workbench CLI
//!
//! A toolkit for working with linear logic — parsing formulas, searching for proofs,
//! extracting computational content, and compiling to Rust.

use clap::{Parser, Subcommand};
use colored::Colorize;
use lolli_extract::{extract_term, normalize};
use lolli_parse::{parse_formula, parse_sequent};
use lolli_prove::Prover;

#[derive(Parser)]
#[command(name = "lolli")]
#[command(author = "Ibrahim Cesar")]
#[command(version)]
#[command(about = "Linear Logic Workbench - Parse, prove, extract, and compile linear logic", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse and pretty-print a formula
    Parse {
        /// Formula to parse
        formula: String,

        /// Output in ASCII instead of Unicode
        #[arg(short, long)]
        ascii: bool,

        /// Output in LaTeX format
        #[arg(short, long)]
        latex: bool,
    },

    /// Check if a sequent is provable
    Prove {
        /// Sequent to prove (e.g., "A, B |- A * B")
        sequent: String,

        /// Maximum search depth
        #[arg(short, long, default_value = "100")]
        depth: usize,

        /// Output format: tree, latex, dot
        #[arg(short, long, default_value = "tree")]
        format: String,
    },

    /// Extract a term from a proof
    Extract {
        /// Sequent to prove
        sequent: String,

        /// Normalize the extracted term
        #[arg(short, long)]
        normalize: bool,
    },

    /// Generate Rust code from a proof
    Codegen {
        /// Sequent to prove
        sequent: String,

        /// Output file
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Visualize a proof
    Viz {
        /// Sequent to prove
        sequent: String,

        /// Output format: tree, latex, dot, svg
        #[arg(short, long, default_value = "tree")]
        format: String,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Run interactive REPL
    Repl,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse {
            formula,
            ascii,
            latex,
        } => {
            match parse_formula(&formula) {
                Ok(f) => {
                    println!("{}", "Parsed:".green().bold());
                    if latex {
                        println!("  {}", f.pretty_latex());
                    } else if ascii {
                        println!("  {}", f.pretty_ascii());
                    } else {
                        println!("  {}", f.pretty());
                    }

                    println!();
                    println!("{}", "Desugared:".cyan().bold());
                    let desugared = f.desugar();
                    if latex {
                        println!("  {}", desugared.pretty_latex());
                    } else if ascii {
                        println!("  {}", desugared.pretty_ascii());
                    } else {
                        println!("  {}", desugared.pretty());
                    }

                    println!();
                    println!("{}", "Negation:".yellow().bold());
                    let negated = f.negate();
                    if latex {
                        println!("  {}", negated.pretty_latex());
                    } else if ascii {
                        println!("  {}", negated.pretty_ascii());
                    } else {
                        println!("  {}", negated.pretty());
                    }

                    println!();
                    println!(
                        "{} {}",
                        "Polarity:".magenta().bold(),
                        if f.is_positive() {
                            "positive (+)".green()
                        } else {
                            "negative (-)".red()
                        }
                    );
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Prove {
            sequent,
            depth,
            format,
        } => {
            match parse_sequent(&sequent) {
                Ok(s) => {
                    println!("{}", "Sequent:".green().bold());
                    println!("  {}", s.pretty());
                    println!();

                    // Convert two-sided sequent to one-sided for the prover
                    let one_sided = s.to_one_sided();
                    println!("{}", "One-sided form:".cyan().bold());
                    println!("  ⊢ {}", one_sided.linear.iter().map(|f| f.pretty()).collect::<Vec<_>>().join(", "));
                    println!();

                    let mut prover = Prover::new(depth);

                    match prover.prove(&one_sided) {
                        Some(proof) => {
                            println!("{}", "✓ PROVABLE".green().bold());
                            println!();
                            println!("{}", "Proof:".cyan().bold());
                            match format.as_str() {
                                "latex" => {
                                    println!("{}", proof_to_latex(&proof, 0));
                                }
                                "dot" => {
                                    println!("{}", proof_to_dot(&proof));
                                }
                                _ => {
                                    // Default tree format
                                    print_proof_tree(&proof, 0);
                                }
                            }
                            println!();
                            println!("{} {}", "Depth:".yellow(), proof.depth());
                            println!("{} {}", "Cut count:".yellow(), proof.cut_count());
                        }
                        None => {
                            println!("{}", "✗ NOT PROVABLE".red().bold());
                            println!("  (within depth limit of {})", depth);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Extract { sequent, normalize: should_normalize } => {
            match parse_sequent(&sequent) {
                Ok(s) => {
                    println!("{}", "Sequent:".green().bold());
                    println!("  {}", s.pretty());
                    println!();

                    // Convert to one-sided and prove
                    let one_sided = s.to_one_sided();
                    let mut prover = Prover::new(100);

                    match prover.prove(&one_sided) {
                        Some(proof) => {
                            println!("{}", "✓ Provable".green());
                            println!();

                            // Extract term from proof
                            let term = extract_term(&proof);

                            println!("{}", "Extracted term:".cyan().bold());
                            println!("  {}", term.pretty());

                            if should_normalize {
                                println!();
                                let normalized = normalize(&term);
                                println!("{}", "Normalized:".yellow().bold());
                                println!("  {}", normalized.pretty());
                            }
                        }
                        None => {
                            println!("{}", "✗ NOT PROVABLE".red().bold());
                            println!("  Cannot extract term from unprovable sequent");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Codegen { sequent, output } => {
            match parse_sequent(&sequent) {
                Ok(s) => {
                    println!("{}", "Sequent:".green().bold());
                    println!("  {}", s.pretty());
                    println!();

                    // Show the type that would be generated
                    use lolli_codegen::RustCodegen;
                    let codegen = RustCodegen::new();

                    println!("{}", "Type signature:".cyan().bold());
                    if !s.antecedent.is_empty() {
                        let args: Vec<String> = s
                            .antecedent
                            .iter()
                            .map(|f| codegen.formula_to_type(f))
                            .collect();
                        print!("  fn f(");
                        for (i, arg) in args.iter().enumerate() {
                            if i > 0 {
                                print!(", ");
                            }
                            print!("arg{}: {}", i, arg);
                        }
                        print!(")");
                    } else {
                        print!("  fn f()");
                    }

                    if !s.succedent.is_empty() {
                        let ret: Vec<String> = s
                            .succedent
                            .iter()
                            .map(|f| codegen.formula_to_type(f))
                            .collect();
                        if ret.len() == 1 {
                            println!(" -> {}", ret[0]);
                        } else {
                            println!(" -> ({})", ret.join(", "));
                        }
                    } else {
                        println!();
                    }

                    println!();
                    println!(
                        "{}",
                        format!("(Full codegen not yet implemented - output: {:?})", output)
                            .yellow()
                    );
                    println!("  See Issues #15-17 for code generation implementation");
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Viz {
            sequent,
            format,
            output,
        } => {
            match parse_sequent(&sequent) {
                Ok(s) => {
                    println!("{}", "Sequent:".green().bold());
                    println!("  {}", s.pretty());
                    println!();
                    println!(
                        "{}",
                        format!(
                            "(Visualization not yet implemented - format: {}, output: {:?})",
                            format, output
                        )
                        .yellow()
                    );
                    println!("  See Issues #18-20 for visualization implementation");
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Repl => {
            println!("{}", "Lolli Linear Logic Workbench REPL".green().bold());
            println!("{}", "(Full REPL not yet implemented - see Issue #22)".yellow());
            println!();
            println!("Commands:");
            println!("  :prove <sequent>   - Prove a sequent");
            println!("  :parse <formula>   - Parse and display a formula");
            println!("  :extract <sequent> - Extract term from proof");
            println!("  :codegen <sequent> - Generate Rust code");
            println!("  :help              - Show help");
            println!("  :quit              - Exit REPL");
            println!();
            println!("For now, use the subcommands directly:");
            println!("  {} parse \"A -o B\"", "lolli".cyan());
            println!("  {} prove \"A, B |- A * B\"", "lolli".cyan());
        }
    }
}

use lolli_core::Proof;

/// Print a proof tree in ASCII format
fn print_proof_tree(proof: &Proof, indent: usize) {
    let prefix = "  ".repeat(indent);

    // Print premises first (above the line)
    for premise in &proof.premises {
        print_proof_tree(premise, indent + 1);
    }

    // Print the inference line
    let conclusion_formulas = proof.conclusion.linear.iter()
        .map(|f| f.pretty())
        .collect::<Vec<_>>()
        .join(", ");
    let rule_name = format!("{:?}", proof.rule);

    if !proof.premises.is_empty() {
        let line_len = conclusion_formulas.len().max(20);
        println!("{}{}  {}", prefix, "─".repeat(line_len), rule_name.cyan());
    }

    println!("{}⊢ {}", prefix, conclusion_formulas);
}

/// Convert a proof to LaTeX format
fn proof_to_latex(proof: &Proof, _depth: usize) -> String {
    let mut lines = vec![];
    lines.push("\\begin{prooftree}".to_string());
    proof_to_latex_inner(proof, &mut lines);
    lines.push("\\end{prooftree}".to_string());
    lines.join("\n")
}

fn proof_to_latex_inner(proof: &Proof, lines: &mut Vec<String>) {
    // Premises first
    for premise in &proof.premises {
        proof_to_latex_inner(premise, lines);
    }

    let conclusion = proof.conclusion.linear.iter()
        .map(|f| f.pretty_latex())
        .collect::<Vec<_>>()
        .join(", ");

    let rule_name = format!("{:?}", proof.rule);

    match proof.premises.len() {
        0 => lines.push(format!("  \\AxiomC{{$\\vdash {}$}}", conclusion)),
        1 => lines.push(format!("  \\UnaryInfC{{$\\vdash {}$}} % {}", conclusion, rule_name)),
        2 => lines.push(format!("  \\BinaryInfC{{$\\vdash {}$}} % {}", conclusion, rule_name)),
        _ => lines.push(format!("  \\TrinaryInfC{{$\\vdash {}$}} % {}", conclusion, rule_name)),
    }
}

/// Convert a proof to DOT (Graphviz) format
fn proof_to_dot(proof: &Proof) -> String {
    let mut lines = vec![];
    lines.push("digraph proof {".to_string());
    lines.push("  rankdir=BT;".to_string());
    lines.push("  node [shape=box];".to_string());

    let mut counter = 0;
    proof_to_dot_inner(proof, &mut lines, &mut counter);

    lines.push("}".to_string());
    lines.join("\n")
}

fn proof_to_dot_inner(proof: &Proof, lines: &mut Vec<String>, counter: &mut usize) -> usize {
    let my_id = *counter;
    *counter += 1;

    let conclusion = proof.conclusion.pretty().replace('"', "\\\"");
    let rule_name = format!("{:?}", proof.rule);

    lines.push(format!("  n{} [label=\"⊢ {}\\n({})\"];", my_id, conclusion, rule_name));

    for premise in &proof.premises {
        let child_id = proof_to_dot_inner(premise, lines, counter);
        lines.push(format!("  n{} -> n{};", child_id, my_id));
    }

    my_id
}
