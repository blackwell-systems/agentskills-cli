use colored::Colorize;
use std::path::PathBuf;
use thiserror::Error;

/// Main error type for agentskills-cli operations
#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),
}

/// Severity level for validation issues
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

/// A single validation error or warning
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub error_type: String,
    pub message: String,
    pub file: Option<PathBuf>,
    pub line: Option<usize>,
    pub severity: Severity,
}

/// Aggregated validation results with errors and warnings
#[derive(Debug, Default)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
}

impl ValidationResult {
    /// Create a new empty validation result
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an error to the result
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Add a warning to the result
    pub fn add_warning(&mut self, warning: ValidationError) {
        self.warnings.push(warning);
    }

    /// Check if validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Format validation results for CLI display with colors
    pub fn format_output(&self) -> String {
        let mut output = String::new();

        // Display errors
        for err in &self.errors {
            let prefix = "ERROR".red().bold();
            let location = if let (Some(file), Some(line)) = (&err.file, err.line) {
                format!("{}:{}", file.display(), line)
            } else if let Some(file) = &err.file {
                format!("{}", file.display())
            } else {
                String::new()
            };

            let location_str = if !location.is_empty() {
                format!(" [{}]", location)
            } else {
                String::new()
            };

            output.push_str(&format!("{}{}: {}\n", prefix, location_str, err.message));
        }

        // Display warnings
        for warn in &self.warnings {
            let prefix = "WARNING".yellow().bold();
            let location = if let (Some(file), Some(line)) = (&warn.file, warn.line) {
                format!("{}:{}", file.display(), line)
            } else if let Some(file) = &warn.file {
                format!("{}", file.display())
            } else {
                String::new()
            };

            let location_str = if !location.is_empty() {
                format!(" [{}]", location)
            } else {
                String::new()
            };

            output.push_str(&format!("{}{}: {}\n", prefix, location_str, warn.message));
        }

        // Add summary line
        let error_count = self.errors.len();
        let warning_count = self.warnings.len();

        let summary = if error_count == 0 && warning_count == 0 {
            "No issues found".green().to_string()
        } else {
            let parts: Vec<String> = vec![
                if error_count > 0 {
                    Some(format!(
                        "{} error{}",
                        error_count,
                        if error_count == 1 { "" } else { "s" }
                    ))
                } else {
                    None
                },
                if warning_count > 0 {
                    Some(format!(
                        "{} warning{}",
                        warning_count,
                        if warning_count == 1 { "" } else { "s" }
                    ))
                } else {
                    None
                },
            ]
            .into_iter()
            .flatten()
            .collect();

            parts.join(", ")
        };

        output.push_str(&format!("\n{}\n", summary));
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_validation_result_new() {
        let result = ValidationResult::new();
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
        assert!(result.is_valid());
    }

    #[test]
    fn test_add_error() {
        let mut result = ValidationResult::new();
        result.add_error(ValidationError {
            error_type: "missing_field".to_string(),
            message: "name is required".to_string(),
            file: None,
            line: None,
            severity: Severity::Error,
        });

        assert_eq!(result.errors.len(), 1);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_add_warning() {
        let mut result = ValidationResult::new();
        result.add_warning(ValidationError {
            error_type: "unknown_field".to_string(),
            message: "unknown field 'extra'".to_string(),
            file: None,
            line: None,
            severity: Severity::Warning,
        });

        assert_eq!(result.warnings.len(), 1);
        assert!(result.is_valid()); // warnings don't affect validity
    }

    #[test]
    fn test_is_valid_with_errors() {
        let mut result = ValidationResult::new();
        result.add_error(ValidationError {
            error_type: "test".to_string(),
            message: "test error".to_string(),
            file: None,
            line: None,
            severity: Severity::Error,
        });

        assert!(!result.is_valid());
    }

    #[test]
    fn test_format_output_no_issues() {
        let result = ValidationResult::new();
        let output = result.format_output();
        assert!(output.contains("No issues found"));
    }

    #[test]
    fn test_format_output_with_errors() {
        let mut result = ValidationResult::new();
        result.add_error(ValidationError {
            error_type: "missing_field".to_string(),
            message: "name is required".to_string(),
            file: Some(PathBuf::from("SKILL.md")),
            line: Some(5),
            severity: Severity::Error,
        });

        let output = result.format_output();
        assert!(output.contains("ERROR"));
        assert!(output.contains("SKILL.md:5"));
        assert!(output.contains("name is required"));
        assert!(output.contains("1 error"));
    }

    #[test]
    fn test_format_output_with_warnings() {
        let mut result = ValidationResult::new();
        result.add_warning(ValidationError {
            error_type: "unknown_field".to_string(),
            message: "unknown field detected".to_string(),
            file: Some(PathBuf::from("SKILL.md")),
            line: None,
            severity: Severity::Warning,
        });

        let output = result.format_output();
        assert!(output.contains("WARNING"));
        assert!(output.contains("SKILL.md"));
        assert!(output.contains("unknown field detected"));
        assert!(output.contains("1 warning"));
    }

    #[test]
    fn test_format_output_mixed() {
        let mut result = ValidationResult::new();
        result.add_error(ValidationError {
            error_type: "error".to_string(),
            message: "error message".to_string(),
            file: None,
            line: None,
            severity: Severity::Error,
        });
        result.add_error(ValidationError {
            error_type: "error2".to_string(),
            message: "another error".to_string(),
            file: None,
            line: None,
            severity: Severity::Error,
        });
        result.add_warning(ValidationError {
            error_type: "warning".to_string(),
            message: "warning message".to_string(),
            file: None,
            line: None,
            severity: Severity::Warning,
        });

        let output = result.format_output();
        assert!(output.contains("2 errors"));
        assert!(output.contains("1 warning"));
    }
}
