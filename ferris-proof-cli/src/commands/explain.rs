use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;

pub async fn run(error_code: String) -> Result<i32> {
    let error_catalog = create_error_catalog();

    match error_catalog.get(&error_code) {
        Some(explanation) => {
            display_error_explanation(&error_code, explanation);
            Ok(0)
        }
        None => {
            display_unknown_error(&error_code, &error_catalog);
            Ok(1)
        }
    }
}

#[derive(Debug, Clone)]
struct ErrorExplanation {
    title: String,
    description: String,
    causes: Vec<String>,
    solutions: Vec<String>,
    examples: Vec<String>,
    related_codes: Vec<String>,
}

fn create_error_catalog() -> HashMap<String, ErrorExplanation> {
    let mut catalog = HashMap::new();

    // Configuration Errors (FP-CF-xxx)
    catalog.insert(
        "FP-CF-001".to_string(),
        ErrorExplanation {
            title: "Invalid Verification Level".to_string(),
            description: "The specified verification level in the configuration is not recognized."
                .to_string(),
            causes: vec![
                "Typo in verification level name".to_string(),
                "Using an unsupported verification level".to_string(),
                "Configuration file corruption".to_string(),
            ],
            solutions: vec![
                "Use one of: minimal, standard, strict, formal".to_string(),
                "Check spelling and case sensitivity".to_string(),
                "Run 'ferris-proof init' to regenerate configuration".to_string(),
            ],
            examples: vec![
                "level = \"standard\"  # Correct".to_string(),
                "level = \"standrd\"   # Incorrect - typo".to_string(),
            ],
            related_codes: vec!["FP-CF-002".to_string(), "FP-CF-003".to_string()],
        },
    );

    catalog.insert("FP-CF-002".to_string(), ErrorExplanation {
        title: "Missing Required Configuration Field".to_string(),
        description: "A required field is missing from the configuration file.".to_string(),
        causes: vec![
            "Incomplete configuration file".to_string(),
            "Manual editing removed required fields".to_string(),
            "Configuration file version mismatch".to_string(),
        ],
        solutions: vec![
            "Add the missing field to ferrisproof.toml".to_string(),
            "Run 'ferris-proof init' to create a complete configuration".to_string(),
            "Check the documentation for required fields".to_string(),
        ],
        examples: vec![
            "[profile]\nlevel = \"standard\"\nenforcement = \"warning\"".to_string(),
        ],
        related_codes: vec!["FP-CF-001".to_string()],
    });

    catalog.insert(
        "FP-CF-003".to_string(),
        ErrorExplanation {
            title: "Conflicting Module-Level Overrides".to_string(),
            description:
                "Multiple glob patterns match the same module, creating conflicting configurations."
                    .to_string(),
            causes: vec![
                "Overlapping glob patterns".to_string(),
                "Duplicate module configurations".to_string(),
                "Incorrect pattern specificity".to_string(),
            ],
            solutions: vec![
                "Remove duplicate glob patterns".to_string(),
                "Use more specific paths to avoid conflicts".to_string(),
                "Review module override precedence rules".to_string(),
            ],
            examples: vec![
                "# Conflicting patterns:\n[modules.\"crypto::*\"]\n[modules.\"crypto::aes::*\"]"
                    .to_string(),
            ],
            related_codes: vec!["FP-CF-001".to_string()],
        },
    );

    // Verification Errors (FP-VR-xxx)
    catalog.insert(
        "FP-VR-001".to_string(),
        ErrorExplanation {
            title: "Property Test Failure".to_string(),
            description:
                "A property-based test found a counterexample that violates the specified property."
                    .to_string(),
            causes: vec![
                "Logic error in implementation".to_string(),
                "Incorrect property specification".to_string(),
                "Edge case not handled properly".to_string(),
            ],
            solutions: vec![
                "Review the counterexample and fix the implementation".to_string(),
                "Verify the property specification is correct".to_string(),
                "Add explicit handling for edge cases".to_string(),
                "Use test case shrinking to find minimal failing example".to_string(),
            ],
            examples: vec![
                "Property: addition is commutative\nCounterexample: a=MAX_INT, b=1 (overflow)"
                    .to_string(),
            ],
            related_codes: vec!["FP-VR-002".to_string()],
        },
    );

    catalog.insert("FP-VR-002".to_string(), ErrorExplanation {
        title: "Formal Specification Violation".to_string(),
        description: "The TLA+ or Alloy specification found an invariant violation or temporal property failure.".to_string(),
        causes: vec![
            "Race condition in concurrent code".to_string(),
            "Incorrect state transition logic".to_string(),
            "Missing synchronization primitives".to_string(),
        ],
        solutions: vec![
            "Review the TLA+ counterexample trace".to_string(),
            "Add proper synchronization mechanisms".to_string(),
            "Verify state transition correctness".to_string(),
            "Check for deadlock or livelock conditions".to_string(),
        ],
        examples: vec![
            "Invariant violated: mutex_count <= 1\nState: {mutex_count: 2, process1: \"critical\", process2: \"critical\"}".to_string(),
        ],
        related_codes: vec!["FP-VR-001".to_string()],
    });

    // Tool Errors (FP-TL-xxx)
    catalog.insert(
        "FP-TL-001".to_string(),
        ErrorExplanation {
            title: "TLA+ TLC Not Found".to_string(),
            description: "The TLA+ model checker (TLC) is not installed or not in the system PATH."
                .to_string(),
            causes: vec![
                "TLA+ tools not installed".to_string(),
                "TLC not in system PATH".to_string(),
                "Incorrect TLC path in configuration".to_string(),
            ],
            solutions: vec![
                "Install TLA+ tools from https://lamport.azurewebsites.net/tla/tools.html"
                    .to_string(),
                "Add TLC to your system PATH".to_string(),
                "Set tlc_path in ferrisproof.toml configuration".to_string(),
                "Run 'ferris-proof install tla' (if available)".to_string(),
            ],
            examples: vec!["[tools.tla_plus]\ntlc_path = \"/usr/local/bin/tlc\"".to_string()],
            related_codes: vec!["FP-TL-002".to_string()],
        },
    );

    catalog.insert(
        "FP-TL-002".to_string(),
        ErrorExplanation {
            title: "Tool Version Incompatible".to_string(),
            description:
                "The installed version of an external tool is not compatible with FerrisProof."
                    .to_string(),
            causes: vec![
                "Tool version too old".to_string(),
                "Tool version too new (breaking changes)".to_string(),
                "Tool not properly installed".to_string(),
            ],
            solutions: vec![
                "Upgrade tool to supported version range".to_string(),
                "Downgrade tool if using unsupported newer version".to_string(),
                "Check FerrisProof documentation for supported versions".to_string(),
                "Use version managers (e.g., rustup, nvm) for tool management".to_string(),
            ],
            examples: vec!["Required: TLC 1.7.0-1.8.x, Found: 1.6.2".to_string()],
            related_codes: vec!["FP-TL-001".to_string()],
        },
    );

    // I/O Errors (FP-IO-xxx)
    catalog.insert(
        "FP-IO-001".to_string(),
        ErrorExplanation {
            title: "Cannot Read Specification File".to_string(),
            description: "FerrisProof cannot read a required specification file.".to_string(),
            causes: vec![
                "File does not exist".to_string(),
                "Insufficient read permissions".to_string(),
                "File is locked by another process".to_string(),
                "Corrupted file system".to_string(),
            ],
            solutions: vec![
                "Check that the file exists at the specified path".to_string(),
                "Verify read permissions on the file and directory".to_string(),
                "Close any applications that might have the file open".to_string(),
                "Check file system integrity".to_string(),
            ],
            examples: vec!["File: specs/formal/protocol.tla\nError: Permission denied".to_string()],
            related_codes: vec!["FP-IO-002".to_string()],
        },
    );

    catalog
}

fn display_error_explanation(code: &str, explanation: &ErrorExplanation) {
    println!("{} {}", "Error Code:".bold().red(), code.bold());
    println!("{} {}", "Title:".bold().yellow(), explanation.title.bold());
    println!();

    println!("{}", "Description:".bold().cyan());
    println!("  {}", explanation.description);
    println!();

    if !explanation.causes.is_empty() {
        println!("{}", "Common Causes:".bold().cyan());
        for cause in &explanation.causes {
            println!("  • {}", cause);
        }
        println!();
    }

    if !explanation.solutions.is_empty() {
        println!("{}", "Solutions:".bold().green());
        for (i, solution) in explanation.solutions.iter().enumerate() {
            println!("  {}. {}", i + 1, solution);
        }
        println!();
    }

    if !explanation.examples.is_empty() {
        println!("{}", "Examples:".bold().magenta());
        for example in &explanation.examples {
            println!("  {}", example.dimmed());
        }
        println!();
    }

    if !explanation.related_codes.is_empty() {
        println!("{}", "Related Error Codes:".bold().blue());
        for related in &explanation.related_codes {
            println!("  • {} (run: ferris-proof explain {})", related, related);
        }
        println!();
    }

    println!("{}", "Need more help?".bold());
    println!("  • Check the documentation: https://ferris-proof.dev/docs");
    println!("  • Report issues: https://github.com/ferris-proof/ferris-proof/issues");
    println!("  • Community forum: https://github.com/ferris-proof/ferris-proof/discussions");
}

fn display_unknown_error(code: &str, catalog: &HashMap<String, ErrorExplanation>) {
    println!("{} {}", "Unknown Error Code:".bold().red(), code.bold());
    println!();

    println!("{}", "The specified error code is not recognized.".yellow());
    println!();

    // Suggest similar error codes
    let similar_codes = find_similar_codes(code, catalog);
    if !similar_codes.is_empty() {
        println!("{}", "Did you mean one of these?".bold().cyan());
        for similar in similar_codes {
            println!("  • {} (run: ferris-proof explain {})", similar, similar);
        }
        println!();
    }

    // Show available error code categories
    println!("{}", "Available Error Categories:".bold().cyan());
    println!("  • {} - Configuration errors", "FP-CF-xxx".green());
    println!("  • {} - Verification errors", "FP-VR-xxx".green());
    println!("  • {} - Tool errors", "FP-TL-xxx".green());
    println!("  • {} - I/O and file system errors", "FP-IO-xxx".green());
    println!("  • {} - Parse errors", "FP-PS-xxx".green());
    println!();

    println!("{}", "To see all available error codes:".bold());
    println!("  ferris-proof explain --list");
}

fn find_similar_codes(input: &str, catalog: &HashMap<String, ErrorExplanation>) -> Vec<String> {
    let mut similar = Vec::new();

    // Extract category from input (e.g., "FP-CF" from "FP-CF-001")
    if let Some(category) = input.get(0..5) {
        for code in catalog.keys() {
            if code.starts_with(category) && code != input {
                similar.push(code.clone());
            }
        }
    }

    // If no category match, look for partial matches
    if similar.is_empty() {
        for code in catalog.keys() {
            if code.contains(input) || input.contains(code) {
                similar.push(code.clone());
            }
        }
    }

    similar.sort();
    similar.truncate(5); // Limit to 5 suggestions
    similar
}
