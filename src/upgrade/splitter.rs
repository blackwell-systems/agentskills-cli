use crate::types::Error;
use crate::upgrade::analyzer::BloatAnalysis;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Result of splitting SKILL.md into core + references/
#[derive(Debug)]
pub struct SplitResult {
    pub core_content: String,
    pub reference_files: HashMap<String, String>,
    pub triggers: Vec<String>,
}

/// Splits SKILL.md based on BloatAnalysis
pub fn split_content(
    skill_path: &Path,
    analysis: &BloatAnalysis,
) -> Result<SplitResult, Error> {
    let content = fs::read_to_string(skill_path)
        .map_err(|e| Error::IoError(format!("Failed to read SKILL.md: {}", e)))?;

    let lines: Vec<&str> = content.lines().collect();
    let mut reference_files = HashMap::new();

    // Track which lines should be moved to references
    let mut lines_to_remove: Vec<(usize, usize)> = analysis
        .suggested_splits
        .iter()
        .map(|s| (s.start_line, s.end_line))
        .collect();

    // Sort by start line to process in order
    lines_to_remove.sort_by_key(|&(start, _)| start);

    // Extract sections and create reference files
    for suggestion in &analysis.suggested_splits {
        let section_lines: Vec<String> = lines[suggestion.start_line..suggestion.end_line]
            .iter()
            .map(|&s| s.to_string())
            .collect();

        // Add dedup marker at the start
        let dedup_marker = format!("<!-- injected: references/{} -->\n", suggestion.target_file);
        let reference_content = format!("{}{}", dedup_marker, section_lines.join("\n"));

        reference_files.insert(suggestion.target_file.clone(), reference_content);
    }

    // Build core content by removing extracted sections
    let mut core_lines = Vec::new();
    let mut current_idx = 0;

    for (start, end) in &lines_to_remove {
        // Add lines before this section
        core_lines.extend(lines[current_idx..*start].iter().map(|&s| s.to_string()));
        current_idx = *end;
    }

    // Add remaining lines
    core_lines.extend(lines[current_idx..].iter().map(|&s| s.to_string()));

    let core_body = core_lines.join("\n");

    // Generate triggers frontmatter
    let triggers_yaml = generate_triggers_frontmatter(&analysis.trigger_patterns);

    // Extract existing frontmatter if present
    let (existing_frontmatter, body_without_frontmatter) = extract_frontmatter(&core_body);

    // Merge frontmatter
    let mut new_frontmatter = if existing_frontmatter.is_empty() {
        triggers_yaml.clone()
    } else {
        // Merge triggers into existing frontmatter
        merge_frontmatter(&existing_frontmatter, &triggers_yaml, analysis)
    };

    // Add agent-references if needed
    if analysis.needs_agent_references {
        let reference_list: Vec<String> = reference_files.keys().cloned().collect();
        let agent_refs = format!(
            "agent-references:\n{}",
            reference_list
                .iter()
                .map(|f| format!("  - references/{}", f))
                .collect::<Vec<_>>()
                .join("\n")
        );

        new_frontmatter = if new_frontmatter.is_empty() {
            format!("---\n{}\n---\n", agent_refs)
        } else {
            // Insert before closing ---
            new_frontmatter = new_frontmatter
                .trim_end_matches("---\n")
                .trim_end_matches("---")
                .to_string();
            format!("{}{}---\n", new_frontmatter, agent_refs)
        };
    }

    let core_content = format!("{}{}", new_frontmatter, body_without_frontmatter);

    Ok(SplitResult {
        core_content,
        reference_files,
        triggers: analysis.trigger_patterns.clone(),
    })
}

/// Extracts frontmatter from content (returns frontmatter including delimiters and body)
fn extract_frontmatter(content: &str) -> (String, String) {
    if content.trim_start().starts_with("---") {
        if let Some(end_idx) = content[3..].find("\n---") {
            let frontmatter_end = end_idx + 3 + 4; // +3 for first "---", +4 for "\n---"
            let frontmatter = &content[..frontmatter_end];
            let body = &content[frontmatter_end..];
            return (frontmatter.to_string(), body.to_string());
        }
    }
    (String::new(), content.to_string())
}

/// Merges triggers into existing frontmatter
fn merge_frontmatter(
    existing: &str,
    triggers_yaml: &str,
    analysis: &BloatAnalysis,
) -> String {
    // Strip delimiters from both
    let existing_stripped = existing
        .trim_start_matches("---\n")
        .trim_start_matches("---")
        .trim_end_matches("\n---")
        .trim_end_matches("---");

    let triggers_stripped = triggers_yaml
        .trim_start_matches("---\n")
        .trim_start_matches("---")
        .trim_end_matches("\n---")
        .trim_end_matches("---");

    // Check if triggers already exist
    if existing.contains("triggers:") {
        // Don't duplicate
        format!("---\n{}---\n", existing_stripped)
    } else {
        // Add triggers
        format!("---\n{}\n{}---\n", existing_stripped, triggers_stripped)
    }
}

/// Generates triggers frontmatter YAML
fn generate_triggers_frontmatter(patterns: &[String]) -> String {
    if patterns.is_empty() {
        return String::new();
    }

    let triggers_list = patterns
        .iter()
        .map(|p| format!("  - \"{}\"", p))
        .collect::<Vec<_>>()
        .join("\n");

    format!("---\ntriggers:\n{}\n---\n", triggers_list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upgrade::analyzer::{BloatAnalysis, SplitSuggestion};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_split_content_extracts_sections() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = r#"---
name: test-skill
description: test
---

## Section 1

Content 1

## Section 2

Content 2
"#;
        temp_file.write_all(content.as_bytes()).unwrap();

        let analysis = BloatAnalysis {
            total_lines: 13,
            suggested_splits: vec![SplitSuggestion {
                section_name: "Section 1".to_string(),
                start_line: 5,
                end_line: 8,
                target_file: "section-1.md".to_string(),
            }],
            trigger_patterns: vec!["/test".to_string()],
            needs_agent_references: false,
        };

        let result = split_content(temp_file.path(), &analysis).unwrap();

        // Should have one reference file
        assert_eq!(result.reference_files.len(), 1);
        assert!(result.reference_files.contains_key("section-1.md"));

        // Reference file should have dedup marker
        let ref_content = result.reference_files.get("section-1.md").unwrap();
        assert!(ref_content.starts_with("<!-- injected: references/section-1.md -->"));
    }

    #[test]
    fn test_split_content_adds_triggers_frontmatter() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = r#"## Section 1

Content here
"#;
        temp_file.write_all(content.as_bytes()).unwrap();

        let analysis = BloatAnalysis {
            total_lines: 3,
            suggested_splits: vec![],
            trigger_patterns: vec!["/test".to_string(), "test:".to_string()],
            needs_agent_references: false,
        };

        let result = split_content(temp_file.path(), &analysis).unwrap();

        // Should add triggers frontmatter
        assert!(result.core_content.contains("triggers:"));
        assert!(result.core_content.contains("\"/test\""));
        assert!(result.core_content.contains("\"test:\""));
    }

    #[test]
    fn test_split_content_preserves_existing_frontmatter() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = r#"---
name: existing-skill
description: existing
---

Content here
"#;
        temp_file.write_all(content.as_bytes()).unwrap();

        let analysis = BloatAnalysis {
            total_lines: 7,
            suggested_splits: vec![],
            trigger_patterns: vec!["/existing".to_string()],
            needs_agent_references: false,
        };

        let result = split_content(temp_file.path(), &analysis).unwrap();

        // Should preserve name and description
        assert!(result.core_content.contains("name: existing-skill"));
        assert!(result.core_content.contains("description: existing"));
        // Should add triggers
        assert!(result.core_content.contains("triggers:"));
    }

    #[test]
    fn test_split_content_adds_agent_references() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = r#"---
name: test-skill
description: test
---

## Reference Section

Content to extract
"#;
        temp_file.write_all(content.as_bytes()).unwrap();

        let analysis = BloatAnalysis {
            total_lines: 9,
            suggested_splits: vec![SplitSuggestion {
                section_name: "Reference Section".to_string(),
                start_line: 5,
                end_line: 8,
                target_file: "reference-section.md".to_string(),
            }],
            trigger_patterns: vec!["/test".to_string()],
            needs_agent_references: true,
        };

        let result = split_content(temp_file.path(), &analysis).unwrap();

        // Should add agent-references field
        assert!(result.core_content.contains("agent-references:"));
        assert!(result
            .core_content
            .contains("- references/reference-section.md"));
    }
}
