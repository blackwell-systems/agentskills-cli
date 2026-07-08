use crate::decompose::pattern_detector::{extract_agent_types, extract_subcommands};
use crate::error::Error;
use crate::models::{
    DecomposeOptions, RoutingDetectionResult, RoutingStyle, SectionTiming, TimingSection,
};
use std::fs;
use std::path::Path;

/// Main detection entry point. Analyzes SKILL.md content and returns structured
/// routing detection results with recommended style.
///
/// Extracts subcommands from `argument-hint` field, agent types from `allowed-tools`,
/// and classifies sections by timing (invocation vs runtime).
///
/// Note: Semantic analysis for timing classification is not yet implemented.
/// All timing sections will have Unknown timing until analyzer integration is added.
///
/// # Arguments
/// * `skill_path` - Path to SKILL.md file
/// * `options` - Upgrade options (unused currently, reserved for future filtering)
///
/// # Returns
/// RoutingDetectionResult with all detected patterns and recommended routing style
pub fn detect_routing_patterns(
    skill_path: &Path,
    _options: &DecomposeOptions,
) -> Result<RoutingDetectionResult, Error> {
    // Read SKILL.md content
    let content = fs::read_to_string(skill_path)?;

    // Extract subcommands from argument-hint
    let subcommands = extract_subcommands(&content).unwrap_or_default();

    // Extract agent types from allowed-tools
    let agent_types = extract_agent_types(&content).unwrap_or_default();

    // Classify timing sections (mechanical extraction for now)
    let timing_sections = classify_timing_sections(&content)?;

    // Recommend routing style based on subcommand count
    let recommended_style = if subcommands.len() >= 4 {
        RoutingStyle::Table
    } else {
        RoutingStyle::Inline
    };

    Ok(RoutingDetectionResult {
        subcommands,
        agent_types,
        timing_sections,
        recommended_style,
    })
}

/// Classify sections by timing (mechanical extraction)
///
/// Parses markdown sections and returns TimingSection metadata.
/// Currently marks all sections as Unknown timing - semantic analysis integration
/// will be added by Agent B (routing generator).
fn classify_timing_sections(content: &str) -> Result<Vec<TimingSection>, Error> {
    let mut timing_sections = Vec::new();

    // Extract frontmatter end to skip it
    let body_start = if let Some(end_pos) = content.find("\n---\n") {
        end_pos + 5 // Skip closing delimiter
    } else {
        0
    };

    let body = &content[body_start..];

    // Parse markdown headers (## Section Name)
    for line in body.lines() {
        if let Some(stripped) = line.strip_prefix("## ") {
            let section_name = stripped.trim().to_string();
            timing_sections.push(TimingSection {
                name: section_name,
                timing: SectionTiming::Unknown,
                trigger_pattern: None,
            });
        }
    }

    Ok(timing_sections)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_detect_patterns_recommends_inline_for_few_subcommands() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = r#"---
name: test-skill
description: Test skill
argument-hint: /test [cmd1|cmd2|cmd3]
---

# Test Skill

Some content here.
"#;
        temp_file.write_all(content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let options = DecomposeOptions::default();
        let result = detect_routing_patterns(temp_file.path(), &options).unwrap();

        assert_eq!(result.subcommands.len(), 3);
        assert_eq!(result.recommended_style, RoutingStyle::Inline);
    }

    #[test]
    fn test_detect_patterns_recommends_table_for_many_subcommands() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = r#"---
name: test-skill
description: Test skill
argument-hint: /test [cmd1|cmd2|cmd3|cmd4|cmd5]
---

# Test Skill

Some content here.
"#;
        temp_file.write_all(content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let options = DecomposeOptions::default();
        let result = detect_routing_patterns(temp_file.path(), &options).unwrap();

        assert_eq!(result.subcommands.len(), 5);
        assert_eq!(result.recommended_style, RoutingStyle::Table);
    }

    #[test]
    fn test_classify_timing_sections_invocation() {
        let content = r#"---
name: test-skill
description: Test
---

## Setup Instructions

Setup content here.

## Runtime Error Handling

Error handling content.
"#;

        let result = classify_timing_sections(content).unwrap();

        // Without semantic analyzer, all sections should be Unknown
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "Setup Instructions");
        assert_eq!(result[0].timing, SectionTiming::Unknown);
        assert_eq!(result[1].name, "Runtime Error Handling");
        assert_eq!(result[1].timing, SectionTiming::Unknown);
    }

    #[test]
    fn test_classify_timing_sections_runtime() {
        let content = r#"---
name: test-skill
description: Test
---

## Error Recovery

Recovery procedures.
"#;

        let result = classify_timing_sections(content).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Error Recovery");
        assert_eq!(result[0].timing, SectionTiming::Unknown);
    }

    #[test]
    fn test_detect_patterns_with_no_semantic_analyzer() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = r#"---
name: test-skill
description: Test skill
argument-hint: /test [cmd1|cmd2]
allowed-tools: Agent(subagent_type=scout) | Agent(subagent_type=wave)
---

## Section 1

Content.

## Section 2

More content.
"#;
        temp_file.write_all(content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let options = DecomposeOptions::default();
        let result = detect_routing_patterns(temp_file.path(), &options).unwrap();

        assert_eq!(result.subcommands.len(), 2);
        assert_eq!(result.agent_types.len(), 2);
        assert_eq!(result.timing_sections.len(), 2);
        // All timing sections should be Unknown without analyzer
        for section in &result.timing_sections {
            assert_eq!(section.timing, SectionTiming::Unknown);
        }
        assert_eq!(result.recommended_style, RoutingStyle::Inline);
    }

    #[test]
    fn test_detect_patterns_handles_missing_fields() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = r#"---
name: test-skill
description: Test skill
---

# Test Skill

Content without argument-hint or allowed-tools.
"#;
        temp_file.write_all(content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let options = DecomposeOptions::default();
        let result = detect_routing_patterns(temp_file.path(), &options).unwrap();

        // Should have empty lists but not error
        assert_eq!(result.subcommands.len(), 0);
        assert_eq!(result.agent_types.len(), 0);
        // Should still recommend inline (< 4 subcommands)
        assert_eq!(result.recommended_style, RoutingStyle::Inline);
    }
}
