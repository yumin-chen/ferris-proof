use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum VerificationLevel {
    Minimal,    // Type safety only
    Standard,   // + Property-based testing
    Strict,     // + Session types, refinement types, concurrency testing
    Formal,     // + Formal specifications
}

impl std::str::FromStr for VerificationLevel {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "minimal" => Ok(VerificationLevel::Minimal),
            "standard" => Ok(VerificationLevel::Standard),
            "strict" => Ok(VerificationLevel::Strict),
            "formal" => Ok(VerificationLevel::Formal),
            _ => Err(format!("Invalid verification level: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Layer {
    Formal,         // Layer 1: Formal specifications
    TypeLevel,      // Layer 2: Type-level verification
    PropertyBased,  // Layer 3: Property-based testing
    Monitoring,     // Layer 4: Production monitoring
}

impl std::str::FromStr for Layer {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "formal" => Ok(Layer::Formal),
            "type-level" => Ok(Layer::TypeLevel),
            "property-based" => Ok(Layer::PropertyBased),
            "monitoring" => Ok(Layer::Monitoring),
            _ => Err(format!("Invalid layer: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnforcementMode {
    Advisory,   // Log violations, don't fail builds
    Warning,    // Emit compiler warnings
    Error,      // Fail compilation/tests
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Technique {
    TypeSafety,
    PropertyTests,
    SessionTypes,
    RefinementTypes,
    ConcurrencyTesting,
    FormalSpecs,
    ModelChecking,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Success,
    Warning,
    Error,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub overall_status: Status,
    pub layer_results: HashMap<Layer, LayerResult>,
    pub metrics: VerificationMetrics,
    pub artifacts: Vec<Artifact>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerResult {
    pub layer: Layer,
    pub status: Status,
    pub violations: Vec<Violation>,
    pub execution_time: std::time::Duration,
    pub tool_outputs: Vec<ToolOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub id: String,
    pub severity: Severity,
    pub location: Location,
    pub message: String,
    pub suggestion: Option<String>,
    pub rule: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub file: PathBuf,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMetrics {
    pub total_time: std::time::Duration,
    pub cache_hit_rate: f64,
    pub memory_usage: u64,
    pub test_cases_executed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub path: PathBuf,
    pub artifact_type: ArtifactType,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    PropertyTest,
    SessionType,
    RefinementType,
    FormalSpec,
    Report,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    pub tool: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub execution_time: std::time::Duration,
}