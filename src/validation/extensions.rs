use crate::error::{Severity, ValidationError, ValidationResult};
use crate::models::SkillMetadata;
use regex::Regex;
use std::path::PathBuf;

/// Validates known extension fields (triggers format, agent-references format,
/// model values). Warns on unknown fields without blocking (encourages innovation).
pub fn validate_extensions(metadata: &SkillMetadata, result: &mut ValidationResult) {
    // Validate triggers format
    if let Some(triggers) = &metadata.triggers {
        for (i, trigger) in triggers.iter().enumerate() {
            if trigger.trim().is_empty() {
                result.add_error(ValidationError {
                    error_type: "invalid_trigger_format".to_string(),
                    message: format!("Trigger at index {} is empty or whitespace-only", i),
                    file: Some(PathBuf::from("SKILL.md")),
                    line: None,
                    severity: Severity::Error,
                });
            }
        }
    }

    // Validate agent-references format
    if let Some(agent_refs) = &metadata.agent_references {
        for (i, agent_ref) in agent_refs.iter().enumerate() {
            if agent_ref.trim().is_empty() {
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

    // Validate model field
    if let Some(model) = &metadata.model {
        if model.trim().is_empty() {
            result.add_error(ValidationError {
                error_type: "invalid_model_value".to_string(),
                message: "Field 'model' must be non-empty if present".to_string(),
                file: Some(PathBuf::from("SKILL.md")),
                line: None,
                severity: Severity::Error,
            });
        }
    }

    // Validate model-context field
    if let Some(model_context) = &metadata.model_context {
        if model_context.trim().is_empty() {
            result.add_error(ValidationError {
                error_type: "invalid_model_context_value".to_string(),
                message: "Field 'model-context' must be non-empty if present".to_string(),
                file: Some(PathBuf::from("SKILL.md")),
                line: None,
                severity: Severity::Error,
            });
        }
    }

    // Validate version format (semver-like: should have dots and numeric parts)
    if let Some(version) = &metadata.version {
        if !version.trim().is_empty() {
            // Simple semver check: should match pattern like "1.0.0" or "0.1" (at least one dot)
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

    // Warn on unknown fields (encourage innovation, don't block)
    for (field_name, _) in &metadata.unknown_fields {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_metadata() -> SkillMetadata {
        SkillMetadata {
            name: "test-skill".to_string(),
            description: "A test skill".to_string(),
            triggers: None,
            agent_references: None,
            model: None,
            model_context: None,
            version: None,
            unknown_fields: HashMap::new(),
        }
    }

    #[test]
    fn test_valid_extensions() {
        let mut metadata = create_metadata();
        metadata.triggers = Some(vec!["/test".to_string(), "test:".to_string()]);
        metadata.agent_references = Some(vec!["ref1.md".to_string()]);
        metadata.model = Some("claude-3-5-sonnet".to_string());
        metadata.version = Some("1.0.0".to_string());

        let mut result = ValidationResult::new();
        validate_extensions(&metadata, &mut result);

        assert!(result.is_valid());
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.warnings.len(), 0);
    }

    #[test]
    fn test_empty_trigger() {
        let mut metadata = create_metadata();
        metadata.triggers = Some(vec!["valid".to_string(), "".to_string(), "  ".to_string()]);

        let mut result = ValidationResult::new();
        validate_extensions(&metadata, &mut result);

        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 2); // empty and whitespace-only
    }

    #[test]
    fn test_empty_agent_reference() {
        let mut metadata = create_metadata();
        metadata.agent_references = Some(vec!["valid.md".to_string(), "".to_string()]);

        let mut result = ValidationResult::new();
        validate_extensions(&metadata, &mut result);

        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_empty_model() {
        let mut metadata = create_metadata();
        metadata.model = Some("".to_string());

        let mut result = ValidationResult::new();
        validate_extensions(&metadata, &mut result);

        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_empty_model_context() {
        let mut metadata = create_metadata();
        metadata.model_context = Some("   ".to_string());

        let mut result = ValidationResult::new();
        validate_extensions(&metadata, &mut result);

        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_valid_version_formats() {
        let valid_versions = vec!["1.0.0", "0.1", "2.3.4.5", "10.20.30"];

        for version in valid_versions {
            let mut metadata = create_metadata();
            metadata.version = Some(version.to_string());

            let mut result = ValidationResult::new();
            validate_extensions(&metadata, &mut result);

            assert!(result.is_valid(), "Version {} should be valid", version);
            assert_eq!(result.warnings.len(), 0);
        }
    }

    #[test]
    fn test_invalid_version_format() {
        let invalid_versions = vec!["v1.0.0", "1.0.0-alpha", "1.x", "abc", "1-0-0"];

        for version in invalid_versions {
            let mut metadata = create_metadata();
            metadata.version = Some(version.to_string());

            let mut result = ValidationResult::new();
            validate_extensions(&metadata, &mut result);

            assert!(result.is_valid(), "Invalid version should be warning, not error");
            assert_eq!(result.warnings.len(), 1, "Version {} should produce warning", version);
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
        assert!(result.warnings.iter().any(|w| w.message.contains("custom-field")));
        assert!(result.warnings.iter().any(|w| w.message.contains("another-unknown")));
    }

    #[test]
    fn test_multiple_validation_issues() {
        let mut metadata = create_metadata();
        metadata.triggers = Some(vec!["".to_string()]);
        metadata.model = Some("  ".to_string());
        metadata.version = Some("invalid".to_string());
        metadata.unknown_fields.insert(
            "custom".to_string(),
            serde_yaml::Value::Bool(true),
        );

        let mut result = ValidationResult::new();
        validate_extensions(&metadata, &mut result);

        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 2); // empty trigger and empty model
        assert_eq!(result.warnings.len(), 2); // invalid version and unknown field
    }
}
