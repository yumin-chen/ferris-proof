use anyhow::{Context, Result};
use ferris_proof_core::{VerificationLevel, EnforcementMode, Technique};
use ferris_proof_config::{Config, ProfileConfig};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use colored::Colorize;
use console::Term;

pub async fn run(
    level: VerificationLevel,
    interactive: bool,
    template: Option<String>,
) -> Result<i32> {
    let term = Term::stdout();
    
    if interactive {
        println!("{}", "🦀 FerrisProof Project Initialization".bold().cyan());
        println!();
        
        let level = prompt_verification_level(&term)?;
        let template = prompt_template(&term)?;
        
        initialize_project(level, template.as_deref(), &term).await
    } else {
        initialize_project(level, template.as_deref(), &Term::stdout()).await
    }
}

fn prompt_verification_level(term: &Term) -> Result<VerificationLevel> {
    println!("Select verification level:");
    println!("  1. {} - Type safety only", "Minimal".yellow());
    println!("  2. {} - + Property-based testing", "Standard".green());
    println!("  3. {} - + Session types, refinement types, concurrency testing", "Strict".blue());
    println!("  4. {} - + Formal specifications (TLA+, Alloy)", "Formal".magenta());
    println!();
    
    loop {
        print!("Enter choice (1-4) [default: 2]: ");
        io::stdout().flush()?;
        
        let input = term.read_line()?;
        let choice = input.trim();
        
        match choice {
            "" | "2" => return Ok(VerificationLevel::Standard),
            "1" => return Ok(VerificationLevel::Minimal),
            "3" => return Ok(VerificationLevel::Strict),
            "4" => return Ok(VerificationLevel::Formal),
            _ => {
                println!("{}", "Invalid choice. Please enter 1-4.".red());
                continue;
            }
        }
    }
}

fn prompt_template(term: &Term) -> Result<Option<String>> {
    println!("Available project templates:");
    println!("  1. {} - Basic Rust project with minimal verification", "minimal");
    println!("  2. {} - Standard web service with property testing", "standard");
    println!("  3. {} - Strict verification with session types", "strict");
    println!("  4. {} - Full formal verification setup", "formal");
    println!("  5. {} - Use current directory as-is", "none");
    println!();
    
    loop {
        print!("Select template (1-5) [default: 2]: ");
        io::stdout().flush()?;
        
        let input = term.read_line()?;
        let choice = input.trim();
        
        match choice {
            "" | "2" => return Ok(Some("standard".to_string())),
            "1" => return Ok(Some("minimal".to_string())),
            "3" => return Ok(Some("strict".to_string())),
            "4" => return Ok(Some("formal".to_string())),
            "5" => return Ok(None),
            _ => {
                println!("{}", "Invalid choice. Please enter 1-5.".red());
                continue;
            }
        }
    }
}

async fn initialize_project(
    level: VerificationLevel,
    _template: Option<&str>,
    _term: &Term,
) -> Result<i32> {
    println!("Initializing FerrisProof project with level: {}", format!("{:?}", level).green());
    
    // Check if ferrisproof.toml already exists
    if Path::new("ferrisproof.toml").exists() {
        println!("{}", "Warning: ferrisproof.toml already exists. Overwriting...".yellow());
    }
    
    // Create configuration based on verification level
    let config = create_config_for_level(level);
    
    // Write configuration file
    write_config_file(&config)?;
    println!("✓ Created {}", "ferrisproof.toml".green());
    
    // Create directory structure
    create_directory_structure(level, _template).await?;
    
    // Create template files if specified
    if let Some(template_name) = _template {
        create_template_files(template_name, level).await?;
    }
    
    println!();
    println!("{}", "🎉 Project initialized successfully!".bold().green());
    println!();
    println!("Next steps:");
    println!("  1. Review the generated {} file", "ferrisproof.toml".cyan());
    println!("  2. Run {} to check your project", "ferris-proof check".cyan());
    
    match level {
        VerificationLevel::Minimal => {
            println!("  3. Consider upgrading to {} level for property testing", "standard".green());
        }
        VerificationLevel::Standard => {
            println!("  3. Add property tests to your {} directory", "tests/".cyan());
        }
        VerificationLevel::Strict => {
            println!("  3. Define session types in the {} directory", "specs/session-types/".cyan());
        }
        VerificationLevel::Formal => {
            println!("  3. Create formal specifications in {} directory", "specs/formal/".cyan());
        }
    }
    
    Ok(0)
}

fn create_config_for_level(level: VerificationLevel) -> Config {
    let techniques = match level {
        VerificationLevel::Minimal => vec![Technique::TypeSafety],
        VerificationLevel::Standard => vec![
            Technique::TypeSafety,
            Technique::PropertyTests,
        ],
        VerificationLevel::Strict => vec![
            Technique::TypeSafety,
            Technique::PropertyTests,
            Technique::SessionTypes,
            Technique::RefinementTypes,
            Technique::ConcurrencyTesting,
        ],
        VerificationLevel::Formal => vec![
            Technique::TypeSafety,
            Technique::PropertyTests,
            Technique::SessionTypes,
            Technique::RefinementTypes,
            Technique::ConcurrencyTesting,
            Technique::FormalSpecs,
            Technique::ModelChecking,
        ],
    };
    
    Config {
        profile: ProfileConfig {
            level,
            enforcement: EnforcementMode::Warning,
            enabled_techniques: techniques,
        },
        ..Default::default()
    }
}

fn write_config_file(config: &Config) -> Result<()> {
    let toml_content = toml::to_string_pretty(config)
        .context("Failed to serialize configuration to TOML")?;
    
    let content = format!(
        "# FerrisProof Configuration\n# Generated by ferris-proof init\n\n{}\n",
        toml_content
    );
    
    fs::write("ferrisproof.toml", content)
        .context("Failed to write ferrisproof.toml")?;
    
    Ok(())
}

async fn create_directory_structure(level: VerificationLevel, _template: Option<&str>) -> Result<()> {
    // Always create these directories
    let base_dirs = vec!["specs", "tests"];
    
    for dir in base_dirs {
        fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create directory: {}", dir))?;
        println!("✓ Created directory {}", dir.green());
    }
    
    // Create level-specific directories
    match level {
        VerificationLevel::Minimal => {
            // No additional directories needed
        }
        VerificationLevel::Standard => {
            fs::create_dir_all("tests/property")?;
            println!("✓ Created directory {}", "tests/property".green());
        }
        VerificationLevel::Strict => {
            fs::create_dir_all("tests/property")?;
            fs::create_dir_all("specs/session-types")?;
            fs::create_dir_all("specs/refinement-types")?;
            println!("✓ Created directory {}", "tests/property".green());
            println!("✓ Created directory {}", "specs/session-types".green());
            println!("✓ Created directory {}", "specs/refinement-types".green());
        }
        VerificationLevel::Formal => {
            fs::create_dir_all("tests/property")?;
            fs::create_dir_all("specs/session-types")?;
            fs::create_dir_all("specs/refinement-types")?;
            fs::create_dir_all("specs/formal")?;
            fs::create_dir_all("specs/formal/tla")?;
            fs::create_dir_all("specs/formal/alloy")?;
            println!("✓ Created directory {}", "tests/property".green());
            println!("✓ Created directory {}", "specs/session-types".green());
            println!("✓ Created directory {}", "specs/refinement-types".green());
            println!("✓ Created directory {}", "specs/formal".green());
            println!("✓ Created directory {}", "specs/formal/tla".green());
            println!("✓ Created directory {}", "specs/formal/alloy".green());
        }
    }
    
    Ok(())
}

async fn create_template_files(template: &str, _level: VerificationLevel) -> Result<()> {
    match template {
        "minimal" => create_minimal_template().await,
        "standard" => create_standard_template().await,
        "strict" => create_strict_template().await,
        "formal" => create_formal_template().await,
        _ => {
            println!("{}", format!("Unknown template: {}", template).yellow());
            Ok(())
        }
    }
}

async fn create_minimal_template() -> Result<()> {
    let readme_content = r#"# FerrisProof Project

This project uses FerrisProof for verification at the **minimal** level.

## Verification Level: Minimal

- ✅ Type safety checking
- ❌ Property-based testing
- ❌ Session types
- ❌ Formal specifications

## Getting Started

1. Run verification: `ferris-proof check`
2. Upgrade verification level: `ferris-proof upgrade --to standard`

## Configuration

See `ferrisproof.toml` for current configuration.
"#;
    
    fs::write("README.md", readme_content)?;
    println!("✓ Created {}", "README.md".green());
    
    Ok(())
}

async fn create_standard_template() -> Result<()> {
    let readme_content = r#"# FerrisProof Project

This project uses FerrisProof for verification at the **standard** level.

## Verification Level: Standard

- ✅ Type safety checking
- ✅ Property-based testing
- ❌ Session types
- ❌ Formal specifications

## Getting Started

1. Run verification: `ferris-proof check`
2. Add property tests in `tests/property/`
3. Upgrade verification level: `ferris-proof upgrade --to strict`

## Property Testing

Property tests should be placed in the `tests/property/` directory.
Example property test:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn my_property(input in any::<i32>()) {
        // Your property test here
        prop_assert!(my_function(input).is_ok());
    }
}
```

## Configuration

See `ferrisproof.toml` for current configuration.
"#;
    
    fs::write("README.md", readme_content)?;
    println!("✓ Created {}", "README.md".green());
    
    // Create example property test
    let property_test_content = r#"use proptest::prelude::*;

// Example property test
proptest! {
    #[test]
    /// **Feature: example, Property 1: Addition is commutative**
    fn addition_is_commutative(a in any::<i32>(), b in any::<i32>()) {
        // Avoid overflow by using saturating arithmetic
        let result1 = a.saturating_add(b);
        let result2 = b.saturating_add(a);
        prop_assert_eq!(result1, result2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn basic_addition_test() {
        assert_eq!(2 + 2, 4);
    }
}
"#;
    
    fs::write("tests/property/example_properties.rs", property_test_content)?;
    println!("✓ Created {}", "tests/property/example_properties.rs".green());
    
    Ok(())
}

async fn create_strict_template() -> Result<()> {
    create_standard_template().await?;
    
    // Create session type example
    let session_type_content = r#"// Example session type definition
// This would be expanded by FerrisProof macros

use ferris_proof_macros::session_type;

#[session_type]
pub enum ProtocolState {
    Init,
    Connected { peer_id: String },
    Authenticated { user_id: u64 },
    Closed,
}

// Example usage:
// let protocol = ProtocolState::Init;
// let protocol = protocol.connect("peer123")?;
// let protocol = protocol.authenticate(42)?;
// protocol.close();
"#;
    
    fs::write("specs/session-types/example_protocol.rs", session_type_content)?;
    println!("✓ Created {}", "specs/session-types/example_protocol.rs".green());
    
    Ok(())
}

async fn create_formal_template() -> Result<()> {
    create_strict_template().await?;
    
    // Create TLA+ specification example
    let tla_spec_content = r#"---- MODULE ExampleProtocol ----
EXTENDS Naturals, Sequences, TLC

CONSTANTS Nodes, MaxMessages

VARIABLES 
    messages,    \* Messages in transit
    nodeState    \* State of each node

TypeOK == 
    /\ messages \in Seq(Nat)
    /\ nodeState \in [Nodes -> {"init", "ready", "done"}]

Init ==
    /\ messages = <<>>
    /\ nodeState = [n \in Nodes |-> "init"]

SendMessage(sender, receiver) ==
    /\ nodeState[sender] = "ready"
    /\ messages' = Append(messages, receiver)
    /\ UNCHANGED nodeState

ReceiveMessage(receiver) ==
    /\ Len(messages) > 0
    /\ Head(messages) = receiver
    /\ nodeState[receiver] = "ready"
    /\ nodeState' = [nodeState EXCEPT ![receiver] = "done"]
    /\ messages' = Tail(messages)

Next ==
    \/ \E sender, receiver \in Nodes : SendMessage(sender, receiver)
    \/ \E receiver \in Nodes : ReceiveMessage(receiver)

Spec == Init /\ [][Next]_<<messages, nodeState>>

\* Safety property: No message is lost
NoMessageLoss == Len(messages) <= MaxMessages

\* Liveness property: All nodes eventually reach done state
AllNodesComplete == <>(\A n \in Nodes : nodeState[n] = "done")

====
"#;
    
    fs::write("specs/formal/tla/example_protocol.tla", tla_spec_content)?;
    println!("✓ Created {}", "specs/formal/tla/example_protocol.tla".green());
    
    // Create Alloy specification example
    let alloy_spec_content = r#"// Example Alloy specification
module ExampleProtocol

// Basic signatures
sig Node {
    state: one State
}

abstract sig State {}
one sig Init, Ready, Done extends State {}

sig Message {
    sender: one Node,
    receiver: one Node
}

// Predicates
pred validTransition[n: Node, s, s': State] {
    (s = Init and s' = Ready) or
    (s = Ready and s' = Done) or
    (s = s')  // No change
}

pred sendMessage[m: Message] {
    m.sender.state = Ready
}

pred receiveMessage[m: Message] {
    m.receiver.state = Ready
    // After receiving, node transitions to Done
}

// Facts
fact NoSelfMessages {
    no m: Message | m.sender = m.receiver
}

fact StateTransitions {
    all n: Node | validTransition[n, Init, n.state]
}

// Assertions
assert NoOrphanMessages {
    all m: Message | some n: Node | n = m.sender or n = m.receiver
}

check NoOrphanMessages for 5
"#;
    
    fs::write("specs/formal/alloy/example_protocol.als", alloy_spec_content)?;
    println!("✓ Created {}", "specs/formal/alloy/example_protocol.als".green());
    
    Ok(())
}