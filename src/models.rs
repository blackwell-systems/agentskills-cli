use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;

/// Parsed representation of Agent Skill SKILL.md frontmatter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub triggers: Option<Vec<String>>,
    #[serde(rename = "agent-references")]
    pub agent_references: Option<Vec<String>>,
    pub model: Option<String>,
    #[serde(rename = "model-context")]
    pub model_context: Option<String>,
    pub version: Option<String>,
    #[serde(flatten)]
    pub unknown_fields: HashMap<String, serde_yaml::Value>,
}

impl SkillMetadata {
    /// Parse SKILL.md from file path
    pub fn from_path(path: &Path) -> Result<Self, Error> {
        let content = fs::read_to_string(path)?;
        content.parse()
    }

    /// Parse SKILL.md from string content
    ///
    /// This is a convenience wrapper around the FromStr trait implementation.
    /// Required by interface contract.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(content: &str) -> Result<Self, Error> {
        content.parse()
    }
}

impl FromStr for SkillMetadata {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Check for frontmatter delimiters
        if !s.starts_with("---\n") {
            return Err(Error::ParseError(
                "SKILL.md must start with '---' frontmatter delimiter".to_string(),
            ));
        }

        // Find closing delimiter
        let rest = &s[4..]; // Skip opening "---\n"
        let end_pos = rest.find("\n---\n").ok_or_else(|| {
            Error::ParseError("Missing closing '---' frontmatter delimiter".to_string())
        })?;

        // Extract frontmatter content
        let frontmatter = &rest[..end_pos];

        // Parse YAML
        let metadata: SkillMetadata = serde_yaml::from_str(frontmatter)?;

        // Validate required fields
        if metadata.name.trim().is_empty() {
            return Err(Error::ValidationError(
                "name field cannot be empty".to_string(),
            ));
        }
        if metadata.description.trim().is_empty() {
            return Err(Error::ValidationError(
                "description field cannot be empty".to_string(),
            ));
        }

        Ok(metadata)
    }
}

/// Configuration for upgrade command
#[derive(Debug, Clone)]
pub struct UpgradeOptions {
    pub dry_run: bool,
    pub with_agent_references: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_frontmatter() {
        let content = r#"---
name: test-skill
description: A test skill
triggers:
  - "/test"
  - "/example"
version: "1.0.0"
---

# Test Skill

Some content here.
"#;

        let metadata = SkillMetadata::from_str(content).unwrap();
        assert_eq!(metadata.name, "test-skill");
        assert_eq!(metadata.description, "A test skill");
        assert_eq!(metadata.triggers.as_ref().unwrap().len(), 2);
        assert_eq!(metadata.version.as_ref().unwrap(), "1.0.0");
    }

    #[test]
    fn test_parse_minimal_frontmatter() {
        let content = r#"---
name: minimal
description: Minimal skill
---

Content
"#;

        let metadata = SkillMetadata::from_str(content).unwrap();
        assert_eq!(metadata.name, "minimal");
        assert_eq!(metadata.description, "Minimal skill");
        assert!(metadata.triggers.is_none());
        assert!(metadata.agent_references.is_none());
    }

    #[test]
    fn test_parse_with_agent_references() {
        let content = r#"---
name: test
description: Test
agent-references:
  - references/details.md
  - references/examples.md
---

Content
"#;

        let metadata = SkillMetadata::from_str(content).unwrap();
        assert_eq!(metadata.agent_references.as_ref().unwrap().len(), 2);
        assert_eq!(
            metadata.agent_references.as_ref().unwrap()[0],
            "references/details.md"
        );
    }

    #[test]
    fn test_parse_with_model_context() {
        let content = r#"---
name: test
description: Test
model: claude-3-opus
model-context: extended
---

Content
"#;

        let metadata = SkillMetadata::from_str(content).unwrap();
        assert_eq!(metadata.model.as_ref().unwrap(), "claude-3-opus");
        assert_eq!(metadata.model_context.as_ref().unwrap(), "extended");
    }

    #[test]
    fn test_parse_with_unknown_fields() {
        let content = r#"---
name: test
description: Test
custom_field: custom_value
another_field: 123
---

Content
"#;

        let metadata = SkillMetadata::from_str(content).unwrap();
        assert_eq!(metadata.unknown_fields.len(), 2);
        assert!(metadata.unknown_fields.contains_key("custom_field"));
        assert!(metadata.unknown_fields.contains_key("another_field"));
    }

    #[test]
    fn test_missing_frontmatter() {
        let content = "# No frontmatter\n\nJust content";
        let result = SkillMetadata::from_str(content);
        assert!(result.is_err());
        match result {
            Err(Error::ParseError(msg)) => {
                assert!(msg.contains("must start with"));
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_missing_closing_delimiter() {
        let content = r#"---
name: test
description: Test

No closing delimiter
"#;

        let result = SkillMetadata::from_str(content);
        assert!(result.is_err());
        match result {
            Err(Error::ParseError(msg)) => {
                assert!(msg.contains("closing"));
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_malformed_yaml() {
        let content = r#"---
name: test
description: Test
invalid_yaml: [unclosed array
---

Content
"#;

        let result = SkillMetadata::from_str(content);
        assert!(result.is_err());
        match result {
            Err(Error::YamlError(_)) => {
                // Expected
            }
            _ => panic!("Expected YamlError"),
        }
    }

    #[test]
    fn test_empty_name_field() {
        let content = r#"---
name: ""
description: Test
---

Content
"#;

        let result = SkillMetadata::from_str(content);
        assert!(result.is_err());
        match result {
            Err(Error::ValidationError(msg)) => {
                assert!(msg.contains("name"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_empty_description_field() {
        let content = r#"---
name: test
description: ""
---

Content
"#;

        let result = SkillMetadata::from_str(content);
        assert!(result.is_err());
        match result {
            Err(Error::ValidationError(msg)) => {
                assert!(msg.contains("description"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_whitespace_only_fields() {
        let content = r#"---
name: "   "
description: test
---

Content
"#;

        let result = SkillMetadata::from_str(content);
        assert!(result.is_err());
        match result {
            Err(Error::ValidationError(msg)) => {
                assert!(msg.contains("name"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }
}
