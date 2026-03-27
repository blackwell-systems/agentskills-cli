use crate::error::{Severity, ValidationError, ValidationResult};
use crate::models::SkillMetadata;
use std::path::PathBuf;

/// Checks that required fields (name, description) are present and non-empty.
/// Verifies SKILL.md file exists.
pub fn validate_base_spec(metadata: &SkillMetadata, result: &mut ValidationResult) {
    // Check name field
    if metadata.name.trim().is_empty() {
        result.add_error(ValidationError {
            error_type: "missing_required_field".to_string(),
            message: "Field 'name' is required and must be non-empty".to_string(),
            file: Some(PathBuf::from("SKILL.md")),
            line: None,
            severity: Severity::Error,
        });
    }

    // Check description field
    if metadata.description.trim().is_empty() {
        result.add_error(ValidationError {
            error_type: "missing_required_field".to_string(),
            message: "Field 'description' is required and must be non-empty".to_string(),
            file: Some(PathBuf::from("SKILL.md")),
            line: None,
            severity: Severity::Error,
        });
    }

    // Note: SKILL.md existence is already verified by SkillMetadata::from_path()
    // which returns an error if the file doesn't exist. We don't need to check again here.
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_metadata(name: &str, description: &str) -> SkillMetadata {
        SkillMetadata {
            name: name.to_string(),
            description: description.to_string(),
            triggers: None,
            agent_references: None,
            model: None,
            model_context: None,
            version: None,
            unknown_fields: HashMap::new(),
        }
    }

    #[test]
    fn test_valid_base_spec() {
        let metadata = create_metadata("test-skill", "A test skill");
        let mut result = ValidationResult::new();

        validate_base_spec(&metadata, &mut result);

        assert!(result.is_valid());
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_missing_name() {
        let metadata = create_metadata("", "A test skill");
        let mut result = ValidationResult::new();

        validate_base_spec(&metadata, &mut result);

        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].message.contains("name"));
    }

    #[test]
    fn test_missing_description() {
        let metadata = create_metadata("test-skill", "");
        let mut result = ValidationResult::new();

        validate_base_spec(&metadata, &mut result);

        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].message.contains("description"));
    }

    #[test]
    fn test_missing_both_fields() {
        let metadata = create_metadata("", "");
        let mut result = ValidationResult::new();

        validate_base_spec(&metadata, &mut result);

        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 2);
    }

    #[test]
    fn test_whitespace_only_name() {
        let metadata = create_metadata("   ", "A test skill");
        let mut result = ValidationResult::new();

        validate_base_spec(&metadata, &mut result);

        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_whitespace_only_description() {
        let metadata = create_metadata("test-skill", "   \n\t");
        let mut result = ValidationResult::new();

        validate_base_spec(&metadata, &mut result);

        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
    }
}
