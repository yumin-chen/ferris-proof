use anyhow::{Result, anyhow, bail};
use serde_json::{Value, json};
use jsonschema::{JSONSchema, ValidationError};
use tracing::debug;

pub struct SchemaValidator {
    config_schema: JSONSchema,
    module_schema: JSONSchema,
}

impl std::fmt::Debug for SchemaValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SchemaValidator")
            .field("config_schema", &"<JSONSchema>")
            .field("module_schema", &"<JSONSchema>")
            .finish()
    }
}

impl SchemaValidator {
    pub fn new() -> Result<Self> {
        let config_schema = Self::build_config_schema()?;
        let module_schema = Self::build_module_schema()?;
        
        Ok(Self {
            config_schema,
            module_schema,
        })
    }

    /// Validate a root configuration file
    pub fn validate(&self, config: &Value) -> Result<()> {
        debug!("Validating configuration against schema");
        
        match self.config_schema.validate(config) {
            Ok(_) => {
                debug!("Configuration schema validation passed");
                Ok(())
            }
            Err(errors) => {
                let error_messages: Vec<String> = errors
                    .into_iter()
                    .map(|e| format!("{:?} at {}", e.kind, e.instance_path))
                    .collect();
                
                bail!("Configuration validation failed:\n{}", error_messages.join("\n"));
            }
        }
    }

    /// Validate a module configuration (partial config)
    pub fn validate_module(&self, config: &Value) -> Result<()> {
        debug!("Validating module configuration against schema");
        
        match self.module_schema.validate(config) {
            Ok(_) => {
                debug!("Module configuration schema validation passed");
                Ok(())
            }
            Err(errors) => {
                let error_messages: Vec<String> = errors
                    .into_iter()
                    .map(|e| format!("{:?} at {}", e.kind, e.instance_path))
                    .collect();
                
                bail!("Module configuration validation failed:\n{}", error_messages.join("\n"));
            }
        }
    }

    /// Get detailed validation errors with suggestions
    pub fn validate_with_details(&self, config: &Value) -> ValidationResult {
        let mut result = ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
        };

        // Validate against schema
        if let Err(errors) = self.config_schema.validate(config) {
            result.is_valid = false;
            for error in errors {
                let location = error.instance_path.to_string();
                let message = format!("{:?}", error.kind);
                
                let validation_error = ValidationErrorDetail {
                    field: if location.is_empty() { "root".to_string() } else { location.clone() },
                    message,
                    location: location.clone(),
                    expected_value: self.get_expected_value_for_field(&location),
                    suggestion: self.get_suggestion_for_field(&location, &error),
                };
                
                result.errors.push(validation_error);
            }
        }

        // Additional business logic validation
        self.validate_business_logic(config, &mut result);

        result
    }

    /// Validate business logic beyond schema validation
    fn validate_business_logic(&self, config: &Value, result: &mut ValidationResult) {
        // Check verification level vs enabled techniques
        if let (Some(level), Some(techniques)) = (
            config.get("profile").and_then(|p| p.get("level")).and_then(|l| l.as_str()),
            config.get("profile").and_then(|p| p.get("enabled_techniques")).and_then(|t| t.as_array()),
        ) {
            self.validate_level_techniques_consistency(level, techniques, result);
        }

        // Check threshold values
        if let Some(thresholds) = config.get("thresholds") {
            self.validate_thresholds(thresholds, result);
        }

        // Check tool configurations
        if let Some(tools) = config.get("tools") {
            self.validate_tool_configs(tools, result);
        }
    }

    fn validate_level_techniques_consistency(&self, level: &str, techniques: &[Value], result: &mut ValidationResult) {
        let techniques_vec: Vec<String> = techniques
            .iter()
            .filter_map(|t| t.as_str().map(|s| s.to_string()))
            .collect();

        match level {
            "minimal" => {
                if !techniques_vec.contains(&"TypeSafety".to_string()) {
                    result.errors.push(ValidationErrorDetail {
                        field: "profile.enabled_techniques".to_string(),
                        message: "Minimal level must include TypeSafety technique".to_string(),
                        location: "profile.enabled_techniques".to_string(),
                        expected_value: Some("TypeSafety".to_string()),
                        suggestion: Some("Add TypeSafety to enabled_techniques or use a higher verification level".to_string()),
                    });
                    result.is_valid = false;
                }
            }
            "standard" => {
                if !techniques_vec.contains(&"PropertyTests".to_string()) {
                    result.errors.push(ValidationErrorDetail {
                        field: "profile.enabled_techniques".to_string(),
                        message: "Standard level must include PropertyTests technique".to_string(),
                        location: "profile.enabled_techniques".to_string(),
                        expected_value: Some("PropertyTests".to_string()),
                        suggestion: Some("Add PropertyTests to enabled_techniques or use a higher verification level".to_string()),
                    });
                    result.is_valid = false;
                }
            }
            "strict" => {
                if !techniques_vec.contains(&"SessionTypes".to_string()) {
                    result.errors.push(ValidationErrorDetail {
                        field: "profile.enabled_techniques".to_string(),
                        message: "Strict level must include SessionTypes technique".to_string(),
                        location: "profile.enabled_techniques".to_string(),
                        expected_value: Some("SessionTypes".to_string()),
                        suggestion: Some("Add SessionTypes to enabled_techniques or use the formal verification level".to_string()),
                    });
                    result.is_valid = false;
                }
            }
            "formal" => {
                if !techniques_vec.contains(&"FormalSpecs".to_string()) {
                    result.errors.push(ValidationErrorDetail {
                        field: "profile.enabled_techniques".to_string(),
                        message: "Formal level must include FormalSpecs technique".to_string(),
                        location: "profile.enabled_techniques".to_string(),
                        expected_value: Some("FormalSpecs".to_string()),
                        suggestion: Some("Add FormalSpecs to enabled_techniques".to_string()),
                    });
                    result.is_valid = false;
                }
            }
            _ => {
                result.errors.push(ValidationErrorDetail {
                    field: "profile.level".to_string(),
                    message: format!("Unknown verification level: {}", level),
                    location: "profile.level".to_string(),
                    expected_value: Some("minimal, standard, strict, formal".to_string()),
                    suggestion: Some("Use one of: minimal, standard, strict, formal".to_string()),
                });
                result.is_valid = false;
            }
        }
    }

    fn validate_thresholds(&self, thresholds: &Value, result: &mut ValidationResult) {
        if let Some(max_time) = thresholds.get("max_verification_time").and_then(|t| t.as_u64()) {
            if max_time == 0 {
                result.errors.push(ValidationErrorDetail {
                    field: "thresholds.max_verification_time".to_string(),
                    message: "max_verification_time must be > 0".to_string(),
                    location: "thresholds.max_verification_time".to_string(),
                    expected_value: Some("positive integer".to_string()),
                    suggestion: Some("Set to at least 60 seconds for basic verification".to_string()),
                });
                result.is_valid = false;
            }
            if max_time > 3600 {
                result.warnings.push(ValidationWarning {
                    field: "thresholds.max_verification_time".to_string(),
                    message: "max_verification_time is very high (> 1 hour)".to_string(),
                    location: "thresholds.max_verification_time".to_string(),
                    suggestion: Some("Consider using a shorter timeout for faster feedback".to_string()),
                });
            }
        }

        if let Some(max_memory) = thresholds.get("max_memory_usage").and_then(|m| m.as_u64()) {
            if max_memory == 0 {
                result.errors.push(ValidationErrorDetail {
                    field: "thresholds.max_memory_usage".to_string(),
                    message: "max_memory_usage must be > 0".to_string(),
                    location: "thresholds.max_memory_usage".to_string(),
                    expected_value: Some("positive integer (bytes)".to_string()),
                    suggestion: Some("Set to at least 1073741824 (1GB) for basic verification".to_string()),
                });
                result.is_valid = false;
            }
        }
    }

    fn validate_tool_configs(&self, tools: &Value, result: &mut ValidationResult) {
        if let Some(proptest) = tools.get("proptest") {
            if let Some(cases) = proptest.get("cases").and_then(|c| c.as_u64()) {
                if cases == 0 {
                    result.errors.push(ValidationErrorDetail {
                        field: "tools.proptest.cases".to_string(),
                        message: "proptest.cases must be > 0".to_string(),
                        location: "tools.proptest.cases".to_string(),
                        expected_value: Some("positive integer".to_string()),
                        suggestion: Some("Set to at least 100 test cases for reasonable coverage".to_string()),
                    });
                    result.is_valid = false;
                }
                if cases > 100000 {
                    result.warnings.push(ValidationWarning {
                        field: "tools.proptest.cases".to_string(),
                        message: "proptest.cases is very high (>100k)".to_string(),
                        location: "tools.proptest.cases".to_string(),
                        suggestion: Some("Consider reducing cases for faster test execution".to_string()),
                    });
                }
            }
        }
    }

    fn get_expected_value_for_field(&self, field: &str) -> Option<String> {
        match field {
            "profile.level" => Some("one of: minimal, standard, strict, formal".to_string()),
            "profile.enforcement" => Some("one of: advisory, warning, error".to_string()),
            "thresholds.max_verification_time" => Some("positive integer (seconds)".to_string()),
            "thresholds.max_memory_usage" => Some("positive integer (bytes)".to_string()),
            "tools.proptest.cases" => Some("positive integer".to_string()),
            _ => None,
        }
    }

    fn get_suggestion_for_field(&self, field: &str, _error: &ValidationError) -> Option<String> {
        match field {
            "profile.level" => Some("Use 'minimal' for basic checks, 'standard' for most projects, 'strict' for critical systems, or 'formal' for safety-critical systems".to_string()),
            "profile.enforcement" => Some("Use 'advisory' during development, 'warning' for CI, or 'error' for production builds".to_string()),
            _ => None,
        }
    }

    /// Build the JSON schema for root configuration
    fn build_config_schema() -> Result<JSONSchema> {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "FerrisProof Configuration",
            "description": "Configuration for FerrisProof multi-layer verification pipeline",
            "type": "object",
            "properties": {
                "profile": {
                    "type": "object",
                    "properties": {
                        "level": {
                            "type": "string",
                            "enum": ["minimal", "standard", "strict", "formal"],
                            "description": "Verification level for the project"
                        },
                        "enforcement": {
                            "type": "string",
                            "enum": ["advisory", "warning", "error"],
                            "description": "How verification violations should be handled"
                        },
                        "enabled_techniques": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["TypeSafety", "PropertyTests", "SessionTypes", "RefinementTypes", "ConcurrencyTesting", "FormalSpecs", "ModelChecking"]
                            },
                            "description": "List of verification techniques to enable"
                        }
                    },
                    "required": ["level", "enforcement", "enabled_techniques"]
                },
                "tools": {
                    "type": "object",
                    "properties": {
                        "tla_plus": {
                            "$ref": "#/definitions/TlaPlusConfig"
                        },
                        "alloy": {
                            "$ref": "#/definitions/AlloyConfig"
                        },
                        "proptest": {
                            "$ref": "#/definitions/ProptestConfig"
                        },
                        "kani": {
                            "$ref": "#/definitions/KaniConfig"
                        }
                    },
                    "additionalProperties": false
                },
                "modules": {
                    "type": "object",
                    "patternProperties": {
                        "^.*$": {
                            "$ref": "#/definitions/ModuleConfig"
                        }
                    },
                    "additionalProperties": false
                },
                "features": {
                    "$ref": "#/definitions/FeatureConfig"
                },
                "thresholds": {
                    "$ref": "#/definitions/Thresholds"
                },
                "ci": {
                    "$ref": "#/definitions/CiConfig"
                }
            },
            "required": ["profile"],
            "additionalProperties": false,
            "definitions": {
                "TlaPlusConfig": {
                    "type": "object",
                    "properties": {
                        "tlc_path": {
                            "type": "string",
                            "description": "Path to TLC model checker executable"
                        },
                        "timeout": {
                            "type": "integer",
                            "minimum": 1,
                            "description": "Timeout in seconds for TLA+ model checking"
                        },
                        "workers": {
                            "type": "integer",
                            "minimum": 1,
                            "description": "Number of parallel workers for model checking"
                        }
                    },
                    "additionalProperties": false
                },
                "AlloyConfig": {
                    "type": "object",
                    "properties": {
                        "analyzer_path": {
                            "type": "string",
                            "description": "Path to Alloy Analyzer executable"
                        },
                        "scope": {
                            "type": "integer",
                            "minimum": 1,
                            "description": "Scope for Alloy analysis"
                        }
                    },
                    "additionalProperties": false
                },
                "ProptestConfig": {
                    "type": "object",
                    "properties": {
                        "cases": {
                            "type": "integer",
                            "minimum": 1,
                            "description": "Number of test cases to generate"
                        },
                        "max_shrink_iters": {
                            "type": "integer",
                            "minimum": 1,
                            "description": "Maximum iterations for test case shrinking"
                        }
                    },
                    "additionalProperties": false
                },
                "KaniConfig": {
                    "type": "object",
                    "properties": {
                        "cbmc_path": {
                            "type": "string",
                            "description": "Path to CBMC executable"
                        },
                        "unwind": {
                            "type": "integer",
                            "minimum": 1,
                            "description": "Unwind depth for Kani verification"
                        }
                    },
                    "additionalProperties": false
                },
                "ModuleConfig": {
                    "type": "object",
                    "properties": {
                        "level": {
                            "type": "string",
                            "enum": ["minimal", "standard", "strict", "formal"]
                        },
                        "enforcement": {
                            "type": "string",
                            "enum": ["advisory", "warning", "error"]
                        },
                        "enabled_techniques": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            }
                        },
                        "spec_file": {
                            "type": "string",
                            "description": "Path to formal specification file"
                        }
                    },
                    "additionalProperties": false
                },
                "FeatureConfig": {
                    "type": "object",
                    "properties": {
                        "cache_enabled": {
                            "type": "boolean",
                            "description": "Enable verification result caching"
                        },
                        "parallel_execution": {
                            "type": "boolean",
                            "description": "Enable parallel verification execution"
                        },
                        "generate_reports": {
                            "type": "boolean",
                            "description": "Generate verification reports"
                        }
                    },
                    "required": ["cache_enabled", "parallel_execution", "generate_reports"]
                },
                "Thresholds": {
                    "type": "object",
                    "properties": {
                        "max_verification_time": {
                            "type": "integer",
                            "minimum": 1,
                            "description": "Maximum verification time in seconds"
                        },
                        "max_memory_usage": {
                            "type": "integer",
                            "minimum": 1,
                            "description": "Maximum memory usage in bytes"
                        },
                        "cache_ttl": {
                            "type": "integer",
                            "minimum": 1,
                            "description": "Cache time-to-live in seconds"
                        }
                    },
                    "required": ["max_verification_time", "max_memory_usage", "cache_ttl"]
                },
                "CiConfig": {
                    "type": "object",
                    "properties": {
                        "fail_on_violations": {
                            "type": "boolean",
                            "description": "Fail CI builds on verification violations"
                        },
                        "generate_artifacts": {
                            "type": "boolean",
                            "description": "Generate verification artifacts in CI"
                        },
                        "upload_reports": {
                            "type": "boolean",
                            "description": "Upload verification reports"
                        }
                    },
                    "required": ["fail_on_violations", "generate_artifacts", "upload_reports"]
                }
            }
        });

        JSONSchema::compile(&schema).map_err(|e| anyhow!("Failed to compile config schema: {}", e))
    }

    /// Build the JSON schema for module configuration (partial config)
    fn build_module_schema() -> Result<JSONSchema> {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "FerrisProof Module Configuration",
            "description": "Partial configuration for specific modules",
            "type": "object",
            "properties": {
                "profile": {
                    "type": "object",
                    "properties": {
                        "level": {
                            "type": "string",
                            "enum": ["minimal", "standard", "strict", "formal"]
                        },
                        "enforcement": {
                            "type": "string",
                            "enum": ["advisory", "warning", "error"]
                        },
                        "enabled_techniques": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            }
                        }
                    }
                },
                "modules": {
                    "type": "object",
                    "patternProperties": {
                        "^.*$": {
                            "$ref": "#/definitions/ModuleConfig"
                        }
                    }
                }
            },
            "additionalProperties": false,
            "definitions": {
                "ModuleConfig": {
                    "type": "object",
                    "properties": {
                        "level": {
                            "type": "string",
                            "enum": ["minimal", "standard", "strict", "formal"]
                        },
                        "enforcement": {
                            "type": "string",
                            "enum": ["advisory", "warning", "error"]
                        },
                        "enabled_techniques": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            }
                        },
                        "spec_file": {
                            "type": "string"
                        }
                    }
                }
            }
        });

        JSONSchema::compile(&schema).map_err(|e| anyhow!("Failed to compile module schema: {}", e))
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new().expect("Failed to create schema validator")
    }
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationErrorDetail>,
    pub warnings: Vec<ValidationWarning>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ValidationErrorDetail {
    pub field: String,
    pub message: String,
    pub location: String,
    pub expected_value: Option<String>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
    pub location: String,
    pub suggestion: Option<String>,
}