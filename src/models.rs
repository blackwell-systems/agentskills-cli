use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;

/// Parsed representation of Agent Skill SKILL.md frontmatter
///
/// Conforms to the official Agent Skills specification at https://agentskills.io/specification
///
/// Fields:
/// - `name` (required): skill identifier
/// - `description` (required): what the skill does and when to use it
/// - `license` (optional): license name or reference
/// - `compatibility` (optional): environment requirements
/// - `metadata` (optional): arbitrary key-value map for additional properties
/// - `allowed_tools` (optional): space-delimited list of pre-approved tools
/// - `unknown_fields`: captures vendor-specific extensions (triggers, agent-references, model, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub license: Option<String>,
    pub compatibility: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    #[serde(rename = "allowed-tools")]
    pub allowed_tools: Option<String>,
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

/// Routing node representing one reference file with routing metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingNode {
    pub reference_file: String,
    pub trigger_patterns: Vec<String>,
    pub agent_types: Vec<String>,
    pub condition_pattern: Option<String>,
}

/// Routing graph containing all routing nodes
#[derive(Debug, Clone)]
pub struct RoutingGraph {
    pub nodes: Vec<RoutingNode>,
}

/// Configuration for upgrade command
#[derive(Debug, Clone, Default)]
pub struct UpgradeOptions {
    pub dry_run: bool,
    pub with_agent_references: bool,
    pub interactive: Option<bool>,
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
        // triggers and version are now vendor extensions in unknown_fields
        assert!(metadata.unknown_fields.contains_key("triggers"));
        assert!(metadata.unknown_fields.contains_key("version"));
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
        // No vendor extensions in this minimal skill
        assert!(!metadata.unknown_fields.contains_key("triggers"));
        assert!(!metadata.unknown_fields.contains_key("agent-references"));
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
        // agent-references is now a vendor extension in unknown_fields
        assert!(metadata.unknown_fields.contains_key("agent-references"));
        if let Some(serde_yaml::Value::Sequence(refs)) = metadata.unknown_fields.get("agent-references")
        {
            assert_eq!(refs.len(), 2);
        } else {
            panic!("agent-references should be a sequence");
        }
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
        // model and model-context are now vendor extensions in unknown_fields
        assert!(metadata.unknown_fields.contains_key("model"));
        assert!(metadata.unknown_fields.contains_key("model-context"));
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

    #[test]
    fn test_upgrade_options_default() {
        let default_options = UpgradeOptions::default();
        assert!(!default_options.dry_run);
        assert!(!default_options.with_agent_references);
        assert_eq!(default_options.interactive, None);
    }

    #[test]
    fn test_upgrade_options_with_partial_init() {
        let options = UpgradeOptions {
            dry_run: true,
            ..Default::default()
        };
        assert!(options.dry_run);
        assert!(!options.with_agent_references);
        assert_eq!(options.interactive, None);
    }
}
