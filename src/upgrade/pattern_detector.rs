#![allow(clippy::collapsible_if)]
use crate::error::Error;
use regex::Regex;

/// Extract subcommand names from argument-hint field in SKILL.md frontmatter
///
/// Parses YAML frontmatter, extracts `argument-hint:` field, and returns list of
/// subcommands (e.g., ["scout", "wave", "status", "bootstrap", "interview"] from
/// "/saw [scout|wave|status] ...").
///
/// Returns Vec<String> of normalized subcommand names (lowercase, no delimiters).
pub fn extract_subcommands(content: &str) -> Result<Vec<String>, Error> {
    // Extract frontmatter
    let frontmatter = extract_frontmatter(content)?;

    // Parse YAML to get argument-hint field
    let yaml: serde_yaml::Value = serde_yaml::from_str(frontmatter)?;

    let argument_hint = yaml
        .get("argument-hint")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::ParseError("Missing argument-hint field".to_string()))?;

    // Extract subcommands using regex patterns
    let mut subcommands = Vec::new();

    // Pattern 1: [scout|wave|status] - bracket-delimited alternatives
    let bracket_regex = Regex::new(r"\[([^\]]+)\]").unwrap();
    for cap in bracket_regex.captures_iter(argument_hint) {
        let alternatives = cap.get(1).unwrap().as_str();
        for alt in alternatives.split('|') {
            let trimmed = alt.trim().to_lowercase();
            // Skip placeholders and special tokens
            if !trimmed.is_empty()
                && !trimmed.starts_with('<')
                && !trimmed.starts_with('-')
                && !trimmed.contains("...")
            {
                if !subcommands.contains(&trimmed) {
                    subcommands.push(trimmed);
                }
            }
        }
    }

    // Pattern 2: (bootstrap <name> | scout <feature> | wave) - paren-delimited alternatives
    let paren_regex = Regex::new(r"\(([^)]+)\)").unwrap();
    for cap in paren_regex.captures_iter(argument_hint) {
        let alternatives = cap.get(1).unwrap().as_str();
        for alt in alternatives.split('|') {
            // Extract first word (the subcommand)
            let trimmed = alt.trim();
            if let Some(first_word) = trimmed.split_whitespace().next() {
                let normalized = first_word.to_lowercase();
                if !normalized.is_empty()
                    && !normalized.starts_with('<')
                    && !normalized.starts_with('-')
                {
                    if !subcommands.contains(&normalized) {
                        subcommands.push(normalized);
                    }
                }
            }
        }
    }

    Ok(subcommands)
}

/// Extract agent types from allowed-tools Agent() calls in SKILL.md frontmatter
///
/// Parses `allowed-tools:` field and extracts `subagent_type` values from
/// `Agent(subagent_type=X)` patterns. Returns Vec<String> of agent type names
/// (e.g., ["scout", "wave-agent", "integration-agent"]).
pub fn extract_agent_types(content: &str) -> Result<Vec<String>, Error> {
    // Extract frontmatter
    let frontmatter = extract_frontmatter(content)?;

    // Parse YAML to get allowed-tools field
    let yaml: serde_yaml::Value = serde_yaml::from_str(frontmatter)?;

    let allowed_tools = match yaml.get("allowed-tools") {
        Some(value) => match value {
            // Handle pipe-delimited string
            serde_yaml::Value::String(s) => s.clone(),
            // Handle array of strings
            serde_yaml::Value::Sequence(seq) => {
                seq.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(" | ")
            }
            _ => {
                return Err(Error::ParseError(
                    "allowed-tools must be string or array".to_string(),
                ))
            }
        },
        None => return Ok(Vec::new()), // allowed-tools is optional
    };

    // Extract agent types using regex: Agent(subagent_type=X)
    let agent_regex = Regex::new(r"Agent\s*\(\s*subagent_type\s*=\s*([^),\s]+)\s*\)").unwrap();
    let mut agent_types = Vec::new();

    for cap in agent_regex.captures_iter(&allowed_tools) {
        let agent_type = cap.get(1).unwrap().as_str().to_string();
        if !agent_types.contains(&agent_type) {
            agent_types.push(agent_type);
        }
    }

    Ok(agent_types)
}

/// Helper function to extract frontmatter from SKILL.md content
fn extract_frontmatter(content: &str) -> Result<&str, Error> {
    if !content.starts_with("---\n") {
        return Err(Error::ParseError(
            "SKILL.md must start with '---' frontmatter delimiter".to_string(),
        ));
    }

    let rest = &content[4..]; // Skip opening "---\n"
    let end_pos = rest.find("\n---\n").ok_or_else(|| {
        Error::ParseError("Missing closing '---' frontmatter delimiter".to_string())
    })?;

    Ok(&rest[..end_pos])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_subcommands_basic() {
        let content = r#"---
name: test-skill
description: Test
argument-hint: /saw [scout|wave|status]
---

Content here.
"#;

        let result = extract_subcommands(content).unwrap();
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"scout".to_string()));
        assert!(result.contains(&"wave".to_string()));
        assert!(result.contains(&"status".to_string()));
    }

    #[test]
    fn test_extract_subcommands_complex() {
        let content = r#"---
name: test-skill
description: Test
argument-hint: /saw (bootstrap <name> | scout <feature> | wave [--auto])
---

Content here.
"#;

        let result = extract_subcommands(content).unwrap();
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"bootstrap".to_string()));
        assert!(result.contains(&"scout".to_string()));
        assert!(result.contains(&"wave".to_string()));
        // --auto should not be included as it's a flag
        assert!(!result.contains(&"--auto".to_string()));
    }

    #[test]
    fn test_extract_subcommands_mixed_patterns() {
        let content = r#"---
name: test-skill
description: Test
argument-hint: /saw [scout|wave] (status | bootstrap <name>)
---

Content here.
"#;

        let result = extract_subcommands(content).unwrap();
        assert_eq!(result.len(), 4);
        assert!(result.contains(&"scout".to_string()));
        assert!(result.contains(&"wave".to_string()));
        assert!(result.contains(&"status".to_string()));
        assert!(result.contains(&"bootstrap".to_string()));
    }

    #[test]
    fn test_extract_agent_types() {
        let content = r#"---
name: test-skill
description: Test
allowed-tools: Agent(subagent_type=scout), Agent(subagent_type=wave-agent)
---

Content here.
"#;

        let result = extract_agent_types(content).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"scout".to_string()));
        assert!(result.contains(&"wave-agent".to_string()));
    }

    #[test]
    fn test_extract_agent_types_multiline() {
        let content = r#"---
name: test-skill
description: Test
allowed-tools: |
  Read | Write | Edit |
  Agent(subagent_type=scout) |
  Agent(subagent_type=wave-agent) |
  Agent(subagent_type=integration-agent)
---

Content here.
"#;

        let result = extract_agent_types(content).unwrap();
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"scout".to_string()));
        assert!(result.contains(&"wave-agent".to_string()));
        assert!(result.contains(&"integration-agent".to_string()));
    }

    #[test]
    fn test_extract_agent_types_array_format() {
        let content = r#"---
name: test-skill
description: Test
allowed-tools:
  - Read
  - Write
  - Agent(subagent_type=scout)
  - Agent(subagent_type=wave-agent)
---

Content here.
"#;

        let result = extract_agent_types(content).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"scout".to_string()));
        assert!(result.contains(&"wave-agent".to_string()));
    }

    #[test]
    fn test_extract_agent_types_no_field() {
        let content = r#"---
name: test-skill
description: Test
---

Content here.
"#;

        let result = extract_agent_types(content).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_missing_frontmatter() {
        let content = "# No frontmatter\n\nJust content";

        let result = extract_subcommands(content);
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

        let result = extract_subcommands(content);
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

        let result = extract_subcommands(content);
        assert!(result.is_err());
        match result {
            Err(Error::YamlError(_)) => {
                // Expected
            }
            _ => panic!("Expected YamlError"),
        }
    }

    #[test]
    fn test_missing_argument_hint() {
        let content = r#"---
name: test-skill
description: Test
---

Content here.
"#;

        let result = extract_subcommands(content);
        assert!(result.is_err());
        match result {
            Err(Error::ParseError(msg)) => {
                assert!(msg.contains("argument-hint"));
            }
            _ => panic!("Expected ParseError for missing argument-hint"),
        }
    }

    #[test]
    fn test_extract_subcommands_deduplicates() {
        let content = r#"---
name: test-skill
description: Test
argument-hint: /saw [scout|wave] (scout | wave)
---

Content here.
"#;

        let result = extract_subcommands(content).unwrap();
        // Should have only 2 items (scout, wave) not 4
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"scout".to_string()));
        assert!(result.contains(&"wave".to_string()));
    }

    #[test]
    fn test_extract_agent_types_deduplicates() {
        let content = r#"---
name: test-skill
description: Test
allowed-tools: Agent(subagent_type=scout) | Agent(subagent_type=scout)
---

Content here.
"#;

        let result = extract_agent_types(content).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains(&"scout".to_string()));
    }

    #[test]
    fn test_extract_subcommands_ignores_placeholders() {
        let content = r#"---
name: test-skill
description: Test
argument-hint: /saw [<command>|scout|<other>]
---

Content here.
"#;

        let result = extract_subcommands(content).unwrap();
        // Should only have "scout", not the placeholders
        assert_eq!(result.len(), 1);
        assert!(result.contains(&"scout".to_string()));
    }

    #[test]
    fn test_extract_agent_types_with_spacing_variations() {
        let content = r#"---
name: test-skill
description: Test
allowed-tools: Agent(subagent_type=scout)|Agent( subagent_type = wave-agent )
---

Content here.
"#;

        let result = extract_agent_types(content).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"scout".to_string()));
        assert!(result.contains(&"wave-agent".to_string()));
    }
}
