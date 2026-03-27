use crate::error::Error;
use crate::models::{ SkillMetadata, UpgradeOptions};
use crate::upgrade::pattern_detector;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Analysis result from scanning SKILL.md for upgrade opportunities
#[derive(Debug, Clone)]
pub struct BloatAnalysis {
    pub total_lines: usize,
    pub suggested_splits: Vec<SplitSuggestion>,
    pub trigger_patterns: Vec<String>,
    pub needs_agent_references: bool,
    pub subcommands: Vec<String>,
    pub agent_types: Vec<String>,
}

/// Represents a section that should be moved to references/
#[derive(Debug, Clone)]
pub struct SplitSuggestion {
    pub section_name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub target_file: String,
}

/// Analyzes SKILL.md for bloat and suggests upgrades
pub fn analyze_bloat(skill_path: &Path, options: &UpgradeOptions) -> Result<BloatAnalysis, Error> {
    let content = fs::read_to_string(skill_path)
        .map_err(|e| Error::ValidationError(format!("Failed to read SKILL.md: {}", e)))?;

    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    // Parse metadata to get skill name for trigger generation
    let metadata = SkillMetadata::from_path(skill_path)?;

    // Extract patterns for routing
    let subcommands = pattern_detector::extract_subcommands(&content)
        .unwrap_or_else(|_| vec![]);
    let agent_types = pattern_detector::extract_agent_types(&content)
        .unwrap_or_else(|_| vec![]);

    // Detect markdown sections using ## headers
    let header_regex = Regex::new(r"^##\s+(.+)$").unwrap();
    let mut sections = Vec::new();
    let mut current_section: Option<(String, usize)> = None;

    for (idx, line) in lines.iter().enumerate() {
        if let Some(captures) = header_regex.captures(line) {
            // Close previous section if exists
            if let Some((name, start)) = current_section {
                sections.push((name, start, idx));
            }
            // Start new section
            let section_name = captures.get(1).unwrap().as_str().to_string();
            current_section = Some((section_name, idx));
        }
    }

    // Close final section
    if let Some((name, start)) = current_section {
        sections.push((name, start, total_lines));
    }

    // Generate split suggestions based on heuristics
    let mut suggested_splits = Vec::new();

    for (section_name, start_line, end_line) in sections {
        let section_length = end_line - start_line;
        let should_split = section_length > 50
            || section_name.contains("Reference")
            || section_name.contains("Procedure")
            || section_name.contains("Implementation")
            || section_name.contains("Steps");

        if should_split {
            let target_file = format!(
                "{}.md",
                section_name
                    .to_lowercase()
                    .replace(' ', "-")
                    .replace(['(', ')', '/', '\\', ':'], "")
            );

            suggested_splits.push(SplitSuggestion {
                section_name,
                start_line,
                end_line,
                target_file,
            });
        }
    }

    // Check for large code blocks
    let code_block_regex = Regex::new(r"```").unwrap();
    let mut in_code_block = false;
    let mut code_block_start = 0;

    for (idx, line) in lines.iter().enumerate() {
        if code_block_regex.is_match(line) {
            if in_code_block {
                // End of code block
                let block_length = idx - code_block_start;
                if block_length > 30 {
                    // Suggest splitting large code blocks
                    suggested_splits.push(SplitSuggestion {
                        section_name: format!("Code Block (lines {}-{})", code_block_start, idx),
                        start_line: code_block_start,
                        end_line: idx + 1,
                        target_file: format!("code-block-{}.md", code_block_start),
                    });
                }
                in_code_block = false;
            } else {
                // Start of code block
                in_code_block = true;
                code_block_start = idx;
            }
        }
    }

    // Generate trigger patterns from skill name
    let skill_name = metadata.name;
    let trigger_patterns = vec![
        format!("/{}", skill_name.to_lowercase()),
        format!("{}:", skill_name.to_lowercase()),
        skill_name.to_lowercase(),
    ];

    Ok(BloatAnalysis {
        total_lines,
        suggested_splits,
        trigger_patterns,
        needs_agent_references: options.with_agent_references,
        subcommands,
        agent_types,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_analyze_bloat_detects_long_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let mut content = String::from("---\nname: test-skill\ndescription: test\n---\n\n");

        // Add 250 lines
        for i in 0..250 {
            content.push_str(&format!("Line {}\n", i));
        }

        temp_file.write_all(content.as_bytes()).unwrap();
        let options = UpgradeOptions {
            dry_run: false,
            with_agent_references: false,
            interactive: None,
        };

        let result = analyze_bloat(temp_file.path(), &options).unwrap();
        assert_eq!(result.total_lines, 254); // 4 frontmatter lines + 250 content lines
    }

    #[test]
    fn test_analyze_bloat_detects_sections() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = r#"---
name: test-skill
description: test
---

## Reference Section

This is a reference section that should be split.

## Implementation Steps

This section has 'Steps' in the title.

## Regular Section

This is a small section.
"#;
        temp_file.write_all(content.as_bytes()).unwrap();
        let options = UpgradeOptions {
            dry_run: false,
            with_agent_references: false,
            interactive: None,
        };

        let result = analyze_bloat(temp_file.path(), &options).unwrap();

        // Should suggest splitting "Reference Section" and "Implementation Steps"
        assert!(result.suggested_splits.len() >= 2);
        assert!(result
            .suggested_splits
            .iter()
            .any(|s| s.section_name.contains("Reference")));
        assert!(result
            .suggested_splits
            .iter()
            .any(|s| s.section_name.contains("Steps")));
    }

    #[test]
    fn test_analyze_bloat_generates_triggers() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = r#"---
name: MySkill
description: test
argument-hint: "/myskill [command1|command2]"
---

Content here.
"#;
        temp_file.write_all(content.as_bytes()).unwrap();
        let options = UpgradeOptions {
            dry_run: false,
            with_agent_references: false,
            interactive: None,
        };

        let result = analyze_bloat(temp_file.path(), &options).unwrap();

        // Should generate trigger patterns
        assert!(!result.trigger_patterns.is_empty());
        assert!(result.trigger_patterns.contains(&"/myskill".to_string()));
        assert!(result.trigger_patterns.contains(&"myskill:".to_string()));

        // Should extract subcommands
        assert!(!result.subcommands.is_empty());
        assert!(result.subcommands.contains(&"command1".to_string()));
        assert!(result.subcommands.contains(&"command2".to_string()));
    }

    #[test]
    fn test_analyze_bloat_extracts_patterns() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = r#"---
name: test-skill
description: test
argument-hint: "/test [scout|wave|status]"
allowed-tools: ["Agent(subagent_type=wave-agent)", "Agent(subagent_type=scout)"]
---

Content here.
"#;
        temp_file.write_all(content.as_bytes()).unwrap();
        let options = UpgradeOptions {
            dry_run: false,
            with_agent_references: false,
            interactive: None,
        };

        let result = analyze_bloat(temp_file.path(), &options).unwrap();

        // Should extract subcommands from argument-hint
        assert!(result.subcommands.contains(&"scout".to_string()));
        assert!(result.subcommands.contains(&"wave".to_string()));
        assert!(result.subcommands.contains(&"status".to_string()));

        // Should extract agent types from allowed-tools
        assert!(result.agent_types.contains(&"wave-agent".to_string()));
        assert!(result.agent_types.contains(&"scout".to_string()));
    }

    #[test]
    fn test_analyze_bloat_detects_large_code_blocks() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let mut content = String::from("---\nname: test-skill\ndescription: test\n---\n\n");

        content.push_str("```bash\n");
        // Add 35 lines of code
        for i in 0..35 {
            content.push_str(&format!("echo 'Line {}'\n", i));
        }
        content.push_str("```\n");

        temp_file.write_all(content.as_bytes()).unwrap();
        let options = UpgradeOptions {
            dry_run: false,
            with_agent_references: false,
            interactive: None,
        };

        let result = analyze_bloat(temp_file.path(), &options).unwrap();

        // Should suggest splitting the large code block
        assert!(result
            .suggested_splits
            .iter()
            .any(|s| s.section_name.contains("Code Block")));
    }
}
