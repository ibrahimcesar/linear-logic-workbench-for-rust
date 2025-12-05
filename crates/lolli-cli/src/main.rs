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

                    // Convert to one-sided and prove
                    let one_sided = s.to_one_sided();
                    let mut prover = Prover::new(100);

                    match prover.prove(&one_sided) {
                        Some(proof) => {
                            println!("{}", "✓ Provable".green());
                            println!();

                            // Extract term from proof
                            let term = extract_term(&proof);

                            // Generate code
                            use lolli_codegen::RustCodegen;
                            let mut codegen = RustCodegen::new();

                            let code = if output.is_some() {
                                // Full module with prelude
                                codegen.generate_module("generated", &s, &term)
                            } else {
                                // Just the function
                                codegen.generate_function("f", &s, &term)
                            };

                            println!("{}", "Generated Rust code:".cyan().bold());
                            println!();
                            for line in code.lines() {
                                println!("{}", line);
                            }

                            // Write to file if output specified
                            if let Some(path) = output {
                                match std::fs::write(&path, &code) {
                                    Ok(_) => {
                                        println!();
                                        println!("{} {}", "Written to:".green(), path);
                                    }
                                    Err(e) => {
                                        eprintln!("{} Failed to write file: {}", "Error:".red().bold(), e);
                                    }
                                }
                            }
                        }
                        None => {
                            println!("{}", "✗ NOT PROVABLE".red().bold());
                            println!("  Cannot generate code from unprovable sequent");
                        }
                    }
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

                    // Convert to one-sided and prove
                    let one_sided = s.to_one_sided();
                    let mut prover = Prover::new(100);

                    match prover.prove(&one_sided) {
                        Some(proof) => {
                            println!("{}", "✓ Provable".green());
                            println!();

                            // Generate visualization
                            use lolli_viz::{render_ascii, render_latex, render_dot};

                            let viz = match format.as_str() {
                                "latex" => render_latex(&proof),
                                "dot" => render_dot(&proof),
                                "svg" => {
                                    println!("{}", "SVG output not yet implemented".yellow());
                                    render_dot(&proof) // Fall back to DOT
                                }
                                _ => render_ascii(&proof),
                            };

                            println!("{}", "Visualization:".cyan().bold());
                            println!();
                            println!("{}", viz);

                            // Write to file if output specified
                            if let Some(path) = output {
                                match std::fs::write(&path, &viz) {
                                    Ok(_) => {
                                        println!();
                                        println!("{} {}", "Written to:".green(), path);
                                    }
                                    Err(e) => {
                                        eprintln!("{} Failed to write file: {}", "Error:".red().bold(), e);
                                    }
                                }
                            }
                        }
                        None => {
                            println!("{}", "✗ NOT PROVABLE".red().bold());
                            println!("  Cannot visualize unprovable sequent");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Repl => {
            run_repl();
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

/// Run the interactive REPL.
fn run_repl() {
    use std::io::{self, Write};

    println!("{}", "╔════════════════════════════════════════════╗".cyan());
    println!("{}", "║  Lolli - Linear Logic Workbench REPL       ║".cyan());
    println!("{}", "╚════════════════════════════════════════════╝".cyan());
    println!();
    println!("Commands:");
    println!("  {}       - Parse and analyze a formula", "formula".green());
    println!("  {}    - Prove a sequent (e.g., A, B |- A * B)", "seq |-".green());
    println!("  {}           - Show this help", ":help".yellow());
    println!("  {}           - Exit the REPL", ":quit".yellow());
    println!();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("{} ", "lolli>".cyan().bold());
        stdout.flush().unwrap();

        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("{} {}", "Error reading input:".red(), e);
                continue;
            }
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // Handle commands
        if input.starts_with(':') {
            match input {
                ":quit" | ":q" | ":exit" => {
                    println!("{}", "Goodbye!".green());
                    break;
                }
                ":help" | ":h" | ":?" => {
                    print_repl_help();
                }
                _ => {
                    println!("{} Unknown command: {}", "Error:".red(), input);
                    println!("Type {} for help", ":help".yellow());
                }
            }
            continue;
        }

        // Check if it's a sequent (contains |- or ⊢)
        if input.contains("|-") || input.contains("⊢") {
            handle_sequent(input);
        } else {
            handle_formula(input);
        }
    }
}

fn print_repl_help() {
    println!();
    println!("{}", "Lolli REPL Help".cyan().bold());
    println!();
    println!("{}", "Formulas:".yellow());
    println!("  A, B, C...        Atoms (propositions)");
    println!("  A -o B or A ⊸ B   Linear implication");
    println!("  A * B or A ⊗ B    Tensor (both)");
    println!("  A + B or A ⊕ B    Plus (choice)");
    println!("  A & B             With (both available)");
    println!("  !A                Of course (replicable)");
    println!("  ?A                Why not");
    println!("  1                 Multiplicative unit");
    println!("  0                 Additive zero");
    println!("  top               Additive top");
    println!("  bot               Multiplicative bottom");
    println!();
    println!("{}", "Sequents:".yellow());
    println!("  A, B |- C         Two-sided sequent");
    println!("  |- A, B           One-sided sequent");
    println!();
    println!("{}", "Examples:".yellow());
    println!("  A -o B            Parse a formula");
    println!("  A, B |- A * B     Prove tensor introduction");
    println!("  A |- A + B        Prove plus introduction");
    println!();
}

fn handle_formula(input: &str) {
    match parse_formula(input) {
        Ok(f) => {
            println!();
            println!("{} {}", "Parsed:".green(), f.pretty());
            println!("{} {}", "Desugared:".cyan(), f.desugar().pretty());
            println!(
                "{} {}",
                "Polarity:".magenta(),
                if f.is_positive() { "positive (+)" } else { "negative (-)" }
            );
            println!();
        }
        Err(e) => {
            println!("{} {}", "Parse error:".red(), e);
        }
    }
}

fn handle_sequent(input: &str) {
    match parse_sequent(input) {
        Ok(s) => {
            println!();
            println!("{} {}", "Sequent:".green(), s.pretty());

            let one_sided = s.to_one_sided();
            let mut prover = Prover::new(100);

            match prover.prove(&one_sided) {
                Some(proof) => {
                    println!("{}", "✓ PROVABLE".green().bold());
                    println!();

                    // Show proof tree
                    println!("{}", "Proof:".cyan());
                    print_proof_tree(&proof, 0);
                    println!();

                    // Show extracted term
                    let term = extract_term(&proof);
                    println!("{} {}", "Extracted term:".yellow(), term.pretty());

                    // Show normalized term
                    let normalized = normalize(&term);
                    if normalized != term {
                        println!("{} {}", "Normalized:".yellow(), normalized.pretty());
                    }
                    println!();
                }
                None => {
                    println!("{}", "✗ NOT PROVABLE".red().bold());
                    println!("  (in linear logic without contraction/weakening)");
                    println!();
                }
            }
        }
        Err(e) => {
            println!("{} {}", "Parse error:".red(), e);
        }
    }
}
