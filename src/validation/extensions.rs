use crate::error::{Severity, ValidationError, ValidationResult};
use crate::models::SkillMetadata;
use regex::Regex;
use std::path::PathBuf;

/// Validates vendor extension fields commonly found in skills.
///
/// These fields are NOT part of the official Agent Skills specification at
/// https://agentskills.io/specification but are common vendor-specific extensions.
///
/// Validation approach:
/// - Recognized extensions (triggers, agent-references, model, etc.): validate format and warn
/// - Unrecognized fields: warn as potential vendor extensions
///
/// This encourages spec compliance while allowing vendor innovation.
pub fn validate_extensions(metadata: &SkillMetadata, result: &mut ValidationResult) {
    // Check for common vendor extensions in unknown_fields
    for (field_name, value) in &metadata.unknown_fields {
        match field_name.as_str() {
            "triggers" => {
                // Warn that this is a vendor extension
                result.add_warning(ValidationError {
                    error_type: "vendor_extension".to_string(),
                    message: "Field 'triggers' is a vendor extension (not in Agent Skills spec)".to_string(),
                    file: Some(PathBuf::from("SKILL.md")),
                    line: None,
                    severity: Severity::Warning,
                });

                // Validate format if it's a string array (some vendors use structured objects)
                if let serde_yaml::Value::Sequence(triggers) = value {
                    for (i, trigger) in triggers.iter().enumerate() {
                        if let serde_yaml::Value::String(s) = trigger {
                            if s.trim().is_empty() {
                                result.add_error(ValidationError {
                                    error_type: "invalid_trigger_format".to_string(),
                                    message: format!("Trigger at index {} is empty or whitespace-only", i),
                                    file: Some(PathBuf::from("SKILL.md")),
                                    line: None,
                                    severity: Severity::Error,
                                });
                            }
                        }
                        // Non-string triggers (objects, maps) are allowed - different vendors may use different formats
                    }
                }
            }
            "agent-references" => {
                result.add_warning(ValidationError {
                    error_type: "vendor_extension".to_string(),
                    message: "Field 'agent-references' is a vendor extension (not in Agent Skills spec)".to_string(),
                    file: Some(PathBuf::from("SKILL.md")),
                    line: None,
                    severity: Severity::Warning,
                });

                if let serde_yaml::Value::Sequence(refs) = value {
                    for (i, agent_ref) in refs.iter().enumerate() {
                        if let serde_yaml::Value::String(s) = agent_ref {
                            if s.trim().is_empty() {
                                result.add_error(ValidationError {
                                    error_type: "invalid_agent_reference_format".to_string(),
                                    message: format!("Agent reference at index {} is empty or whitespace-only", i),
                                    file: Some(PathBuf::from("SKILL.md")),
                                    line: None,
                                    severity: Severity::Error,
                                });
                            }
                        }
                    }
                }
            }
            "model" => {
                result.add_warning(ValidationError {
                    error_type: "vendor_extension".to_string(),
                    message: "Field 'model' is a vendor extension (not in Agent Skills spec)".to_string(),
                    file: Some(PathBuf::from("SKILL.md")),
                    line: None,
                    severity: Severity::Warning,
                });

                if let serde_yaml::Value::String(s) = value {
                    if s.trim().is_empty() {
                        result.add_error(ValidationError {
                            error_type: "invalid_model_value".to_string(),
                            message: "Field 'model' must be non-empty if present".to_string(),
                            file: Some(PathBuf::from("SKILL.md")),
                            line: None,
                            severity: Severity::Error,
                        });
                    }
                }
            }
            "model-context" => {
                result.add_warning(ValidationError {
                    error_type: "vendor_extension".to_string(),
                    message: "Field 'model-context' is a vendor extension (not in Agent Skills spec)".to_string(),
                    file: Some(PathBuf::from("SKILL.md")),
                    line: None,
                    severity: Severity::Warning,
                });

                if let serde_yaml::Value::String(s) = value {
                    if s.trim().is_empty() {
                        result.add_error(ValidationError {
                            error_type: "invalid_model_context_value".to_string(),
                            message: "Field 'model-context' must be non-empty if present".to_string(),
                            file: Some(PathBuf::from("SKILL.md")),
                            line: None,
                            severity: Severity::Error,
                        });
                    }
                }
            }
            "version" => {
                result.add_warning(ValidationError {
                    error_type: "vendor_extension".to_string(),
                    message: "Field 'version' is a vendor extension (not in Agent Skills spec). Consider using 'metadata.version' instead.".to_string(),
                    file: Some(PathBuf::from("SKILL.md")),
                    line: None,
                    severity: Severity::Warning,
                });

                if let serde_yaml::Value::String(version) = value {
                    if !version.trim().is_empty() {
                        let semver_regex = Regex::new(r"^\d+(\.\d+)*$").unwrap();
                        if !semver_regex.is_match(version.trim()) {
                            result.add_warning(ValidationError {
                                error_type: "invalid_version_format".to_string(),
                                message: format!(
                                    "Field 'version' has invalid semver format '{}'. Expected format like '1.0.0'",
                                    version
                                ),
                                file: Some(PathBuf::from("SKILL.md")),
                                line: None,
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
            _ => {
                // Truly unknown field - warn but don't block
                result.add_warning(ValidationError {
                    error_type: "unknown_field".to_string(),
                    message: format!(
                        "Unknown field '{}' - this may be a platform-specific extension",
                        field_name
                    ),
                    file: Some(PathBuf::from("SKILL.md")),
                    line: None,
                    severity: Severity::Warning,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_metadata() -> SkillMetadata {
        SkillMetadata {
            name: "test-skill".to_string(),
            description: "A test skill".to_string(),
            license: None,
            compatibility: None,
            metadata: None,
            allowed_tools: None,
            unknown_fields: HashMap::new(),
        }
    }

    #[test]
    fn test_valid_extensions() {
        let mut metadata = create_metadata();
        metadata.unknown_fields.insert(
            "triggers".to_string(),
            serde_yaml::Value::Sequence(vec![
                serde_yaml::Value::String("/test".to_string()),
                serde_yaml::Value::String("test:".to_string()),
            ]),
        );
        metadata.unknown_fields.insert(
            "agent-references".to_string(),
            serde_yaml::Value::Sequence(vec![serde_yaml::Value::String("ref1.md".to_string())]),
        );
        metadata.unknown_fields.insert(
            "model".to_string(),
            serde_yaml::Value::String("claude-3-5-sonnet".to_string()),
        );
        metadata.unknown_fields.insert(
            "version".to_string(),
            serde_yaml::Value::String("1.0.0".to_string()),
        );

        let mut result = ValidationResult::new();
        validate_extensions(&metadata, &mut result);

        assert!(result.is_valid());
        assert_eq!(result.errors.len(), 0);
        // Now we expect warnings for vendor extensions
        assert_eq!(result.warnings.len(), 4); // triggers, agent-references, model, version
    }

    #[test]
    fn test_empty_trigger() {
        let mut metadata = create_metadata();
        metadata.unknown_fields.insert(
            "triggers".to_string(),
            serde_yaml::Value::Sequence(vec![
                serde_yaml::Value::String("valid".to_string()),
                serde_yaml::Value::String("".to_string()),
                serde_yaml::Value::String("  ".to_string()),
            ]),
        );

        let mut result = ValidationResult::new();
        validate_extensions(&metadata, &mut result);

        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 2); // empty and whitespace-only
        assert_eq!(result.warnings.len(), 1); // vendor extension warning
    }

    #[test]
    fn test_empty_agent_reference() {
        let mut metadata = create_metadata();
        metadata.unknown_fields.insert(
            "agent-references".to_string(),
            serde_yaml::Value::Sequence(vec![
                serde_yaml::Value::String("valid.md".to_string()),
                serde_yaml::Value::String("".to_string()),
            ]),
        );

        let mut result = ValidationResult::new();
        validate_extensions(&metadata, &mut result);

        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.warnings.len(), 1); // vendor extension warning
    }

    #[test]
    fn test_empty_model() {
        let mut metadata = create_metadata();
        metadata
            .unknown_fields
            .insert("model".to_string(), serde_yaml::Value::String("".to_string()));

        let mut result = ValidationResult::new();
        validate_extensions(&metadata, &mut result);

        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.warnings.len(), 1); // vendor extension warning
    }

    #[test]
    fn test_empty_model_context() {
        let mut metadata = create_metadata();
        metadata.unknown_fields.insert(
            "model-context".to_string(),
            serde_yaml::Value::String("   ".to_string()),
        );

        let mut result = ValidationResult::new();
        validate_extensions(&metadata, &mut result);

        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.warnings.len(), 1); // vendor extension warning
    }

    #[test]
    fn test_valid_version_formats() {
        let valid_versions = vec!["1.0.0", "0.1", "2.3.4.5", "10.20.30"];

        for version in valid_versions {
            let mut metadata = create_metadata();
            metadata.unknown_fields.insert(
                "version".to_string(),
                serde_yaml::Value::String(version.to_string()),
            );

            let mut result = ValidationResult::new();
            validate_extensions(&metadata, &mut result);

            assert!(result.is_valid(), "Version {} should be valid", version);
            assert_eq!(result.warnings.len(), 1); // vendor extension warning
        }
    }

    #[test]
    fn test_invalid_version_format() {
        let invalid_versions = vec!["v1.0.0", "1.0.0-alpha", "1.x", "abc", "1-0-0"];

        for version in invalid_versions {
            let mut metadata = create_metadata();
            metadata.unknown_fields.insert(
                "version".to_string(),
                serde_yaml::Value::String(version.to_string()),
            );

            let mut result = ValidationResult::new();
            validate_extensions(&metadata, &mut result);

            assert!(
                result.is_valid(),
                "Invalid version should be warning, not error"
            );
            assert_eq!(
                result.warnings.len(),
                2, // vendor extension + invalid format
                "Version {} should produce 2 warnings",
                version
            );
        }
    }

    #[test]
    fn test_unknown_fields_warning() {
        let mut metadata = create_metadata();
        metadata.unknown_fields.insert(
            "custom-field".to_string(),
            serde_yaml::Value::String("custom value".to_string()),
        );
        metadata.unknown_fields.insert(
            "another-unknown".to_string(),
            serde_yaml::Value::Number(42.into()),
        );

        let mut result = ValidationResult::new();
        validate_extensions(&metadata, &mut result);

        assert!(result.is_valid()); // Warnings don't affect validity
        assert_eq!(result.warnings.len(), 2);
        assert!(result
            .warnings
            .iter()
            .any(|w| w.message.contains("custom-field")));
        assert!(result
            .warnings
            .iter()
            .any(|w| w.message.contains("another-unknown")));
    }

    #[test]
    fn test_multiple_validation_issues() {
        let mut metadata = create_metadata();
        metadata.unknown_fields.insert(
            "triggers".to_string(),
            serde_yaml::Value::Sequence(vec![serde_yaml::Value::String("".to_string())]),
        );
        metadata.unknown_fields.insert(
            "model".to_string(),
            serde_yaml::Value::String("  ".to_string()),
        );
        metadata.unknown_fields.insert(
            "version".to_string(),
            serde_yaml::Value::String("invalid".to_string()),
        );
        metadata
            .unknown_fields
            .insert("custom".to_string(), serde_yaml::Value::Bool(true));

        let mut result = ValidationResult::new();
        validate_extensions(&metadata, &mut result);

        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 2); // empty trigger and empty model
        assert_eq!(result.warnings.len(), 5); // triggers ext, model ext, version ext (2 warnings), custom field
    }
}
