use crate::error::Error;
use crate::upgrade::analyzer::BloatAnalysis;
use crate::upgrade::semantic_analyzer::{SemanticAnalyzer, SectionIntent, TriggerTiming};
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

/// Generate a breadcrumb stub for runtime-triggered sections
fn generate_breadcrumb(section_name: &str, target_file: &str, condition: Option<&str>) -> String {
    let condition_text = condition
        .map(|c| {
            // Strip leading "if " or "when " to avoid "when if X" or "when when X"
            let cleaned = c
                .trim_start_matches("if ")
                .trim_start_matches("when ");
            format!("when {}", cleaned)
        })
        .unwrap_or_else(|| "when needed".to_string());

    format!(
        "## {} — [See references/{} {}]\n\n\
        Read `${{SKILL_DIR}}/references/{}` and follow its instructions.\n",
        section_name,
        target_file,
        condition_text,
        target_file
    )
}

/// Splits SKILL.md based on BloatAnalysis
pub async fn split_content(
    skill_path: &Path,
    analysis: &BloatAnalysis,
    analyzer: Option<Box<dyn SemanticAnalyzer>>,
) -> Result<SplitResult, Error> {
    let content = fs::read_to_string(skill_path)
        .map_err(|e| Error::ValidationError(format!("Failed to read SKILL.md: {}", e)))?;

    let lines: Vec<&str> = content.lines().collect();
    let mut reference_files = HashMap::new();

    // Step 1: Find ALL sections (not just suggested splits)
    let all_sections = {
        use regex::Regex;
        let header_regex = Regex::new(r"^##\s+(.+)$").unwrap();
        let mut sections = Vec::new();
        let mut current_section: Option<(String, usize)> = None;

        for (idx, line) in lines.iter().enumerate() {
            if let Some(captures) = header_regex.captures(line) {
                if let Some((name, start)) = current_section {
                    sections.push((name, start, idx));
                }
                let section_name = captures.get(1).unwrap().as_str().to_string();
                current_section = Some((section_name, idx));
            }
        }

        if let Some((name, start)) = current_section {
            sections.push((name, start, lines.len()));
        }

        sections
    };

    // Step 2: Semantic analysis on ALL sections (if available)
    let sections_with_intent: Vec<(String, usize, usize, String, SectionIntent)> = if let Some(ref analyzer) = analyzer {
        let mut result = Vec::new();
        for (section_name, start_line, end_line) in all_sections {
            // Skip small sections (< 30 lines) - not worth splitting
            if end_line - start_line < 30 {
                continue;
            }

            let section_content: String = lines[start_line..end_line].join("\n");

            let intent = analyzer
                .analyze_section(&section_name, &section_content)
                .await?;

            let target_file = format!(
                "{}.md",
                section_name
                    .to_lowercase()
                    .replace(' ', "-")
                    .replace(['(', ')', '/', '\\', ':'], "")
            );

            result.push((
                section_name,
                start_line,
                end_line,
                target_file,
                intent,
            ));
        }
        result
    } else {
        Vec::new()
    };

    // Step 3: Extract sections to reference files (both invocation and runtime)
    for (_section_name, start_line, end_line, target_file, _intent) in &sections_with_intent {
        let section_lines: Vec<String> = lines[*start_line..*end_line]
            .iter()
            .map(|&s| s.to_string())
            .collect();

        // Add dedup marker at the start
        let dedup_marker = format!("<!-- injected: references/{} -->\n", target_file);
        let reference_content = format!("{}{}", dedup_marker, section_lines.join("\n"));

        reference_files.insert(target_file.clone(), reference_content);
    }

    // Also extract bloat analyzer suggestions not covered by semantic analysis
    for suggestion in &analysis.suggested_splits {
        // Skip if already handled by semantic analysis
        let already_handled = sections_with_intent
            .iter()
            .any(|(_, start, _, _, _)| *start == suggestion.start_line);

        if !already_handled {
            let section_lines: Vec<String> = lines[suggestion.start_line..suggestion.end_line]
                .iter()
                .map(|&s| s.to_string())
                .collect();

            let dedup_marker = format!("<!-- injected: references/{} -->\n", suggestion.target_file);
            let reference_content = format!("{}{}", dedup_marker, section_lines.join("\n"));

            reference_files.insert(suggestion.target_file.clone(), reference_content);
        }
    }

    // Step 4: Build core content with runtime breadcrumbs
    let mut core_lines = Vec::new();
    let mut current_idx = 0;

    // Collect all sections to remove (both from semantic analysis and bloat analyzer)
    let mut sections_to_process: Vec<(usize, usize, String, String, bool)> = Vec::new();

    // Add semantically analyzed sections
    for (section_name, start_line, end_line, target_file, intent) in &sections_with_intent {
        let is_runtime = intent.trigger_timing.as_ref()
            .map(|timing| matches!(timing, TriggerTiming::Runtime))
            .unwrap_or(false);

        sections_to_process.push((*start_line, *end_line, section_name.clone(), target_file.clone(), is_runtime));
    }

    // Add bloat analyzer suggestions not already covered
    for suggestion in &analysis.suggested_splits {
        let already_handled = sections_to_process
            .iter()
            .any(|(start, _, _, _, _)| *start == suggestion.start_line);

        if !already_handled {
            sections_to_process.push((
                suggestion.start_line,
                suggestion.end_line,
                suggestion.section_name.clone(),
                suggestion.target_file.clone(),
                false, // bloat suggestions are invocation-time by default
            ));
        }
    }

    // Sort by start line
    sections_to_process.sort_by_key(|(start, _, _, _, _)| *start);

    // Build core with breadcrumbs for runtime sections
    for (start_line, end_line, section_name, target_file, is_runtime) in sections_to_process {
        // Add lines before this section
        core_lines.extend(lines[current_idx..start_line].iter().map(|&s| s.to_string()));

        if is_runtime {
            // Leave a breadcrumb for runtime sections
            let condition = sections_with_intent
                .iter()
                .find(|(_, s, _, _, _)| *s == start_line)
                .and_then(|(_, _, _, _, intent)| intent.condition_pattern.as_deref());

            let breadcrumb = generate_breadcrumb(&section_name, &target_file, condition);
            core_lines.push(breadcrumb);
        }
        // For invocation-time sections, remove entirely (no breadcrumb)

        current_idx = end_line;
    }

    // Add remaining lines
    core_lines.extend(lines[current_idx..].iter().map(|&s| s.to_string()));

    let core_body = core_lines.join("\n");

    // Step 5: Preserve existing frontmatter
    // Extract existing frontmatter if present - we keep it unchanged
    let (existing_frontmatter, body_without_frontmatter) = extract_frontmatter(&core_body);

    // Reassemble with preserved frontmatter
    let core_content = format!("{}{}", existing_frontmatter, body_without_frontmatter);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upgrade::analyzer::{BloatAnalysis, SplitSuggestion};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_split_content_extracts_sections() {
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
            subcommands: vec![],
            agent_types: vec![],
        };

        let result = split_content(temp_file.path(), &analysis, None).await.unwrap();

        // Should have one reference file
        assert_eq!(result.reference_files.len(), 1);
        assert!(result.reference_files.contains_key("section-1.md"));

        // Reference file should have dedup marker
        let ref_content = result.reference_files.get("section-1.md").unwrap();
        assert!(ref_content.starts_with("<!-- injected: references/section-1.md -->"));
    }

    #[tokio::test]
    async fn test_split_content_no_frontmatter_generation() {
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
            subcommands: vec![],
            agent_types: vec![],
        };

        let result = split_content(temp_file.path(), &analysis, None).await.unwrap();

        // Should NOT add triggers frontmatter (no longer generated)
        assert!(!result.core_content.contains("triggers:"));
        assert!(result.core_content.contains("## Section 1"));
    }

    #[tokio::test]
    async fn test_split_content_preserves_existing_frontmatter() {
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
            subcommands: vec![],
            agent_types: vec![],
        };

        let result = split_content(temp_file.path(), &analysis, None).await.unwrap();

        // Should preserve name and description exactly as they were
        assert!(result.core_content.contains("name: existing-skill"));
        assert!(result.core_content.contains("description: existing"));
        // Should NOT add triggers (no longer generated)
        assert!(!result.core_content.contains("triggers:"));
    }

    #[tokio::test]
    async fn test_split_content_extracts_without_frontmatter_modification() {
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
            subcommands: vec![],
            agent_types: vec![],
        };

        let result = split_content(temp_file.path(), &analysis, None).await.unwrap();

        // Should preserve original frontmatter unchanged
        assert!(result.core_content.contains("name: test-skill"));
        assert!(result.core_content.contains("description: test"));
        // Should NOT add agent-references field (no longer generated)
        assert!(!result.core_content.contains("agent-references:"));
        // Should extract content to references
        assert!(result.reference_files.contains_key("reference-section.md"));
    }

    #[tokio::test]
    #[ignore] // Requires ANTHROPIC_API_KEY
    async fn test_split_content_with_routing_graph() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = r#"---
name: test-skill
description: test
argument-hint: "/test [scout|wave]"
allowed-tools: Agent(subagent_type=scout)
---

## Scout Section

This section is for scout agent only.

## Wave Section

This section is for wave agent only.
"#;
        temp_file.write_all(content.as_bytes()).unwrap();

        let analysis = BloatAnalysis {
            total_lines: 15,
            suggested_splits: vec![
                SplitSuggestion {
                    section_name: "Scout Section".to_string(),
                    start_line: 7,
                    end_line: 10,
                    target_file: "scout-section.md".to_string(),
                },
                SplitSuggestion {
                    section_name: "Wave Section".to_string(),
                    start_line: 11,
                    end_line: 14,
                    target_file: "wave-section.md".to_string(),
                },
            ],
            trigger_patterns: vec!["/test".to_string()],
            needs_agent_references: true,
            subcommands: vec![],
            agent_types: vec![],
        };

        // Use analyzer factory (test will be ignored if neither API key nor CLI available)
        let detection = crate::upgrade::semantic_analyzer::new_analyzer();
        let result = split_content(temp_file.path(), &analysis, detection.analyzer)
            .await
            .unwrap();

        // Should have generated frontmatter with triggers and agent-references
        assert!(result.core_content.contains("triggers:"));
        assert!(result.core_content.contains("agent-references:"));

        // Should have reference files
        assert_eq!(result.reference_files.len(), 2);
        assert!(result.reference_files.contains_key("scout-section.md"));
        assert!(result.reference_files.contains_key("wave-section.md"));
    }
}
