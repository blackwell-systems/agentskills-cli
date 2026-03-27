use crate::error::Error;
use crate::models::{
    PreviewData, SectionPreview, ReferenceFilePreview, BreadcrumbPreview,
    SectionTiming, UpgradeOptions
};
use std::fs;
use std::path::Path;

pub mod analyzer;
pub mod generator;
pub mod splitter;
pub mod pattern_detector;
pub mod semantic_analyzer;
pub mod anthropic_api;
pub mod anthropic_cli;
pub mod openai_api;
pub mod gemini_api;
pub mod gemini_cli;
pub mod copilot_cli;
pub mod routing_graph;
pub mod frontmatter_gen;
pub mod routing_detector;
pub mod routing_generator;
pub mod interactive_recommender;

pub use analyzer::{analyze_bloat, BloatAnalysis, SplitSuggestion};
pub use generator::generate_inject_script;
pub use splitter::{split_content, SplitResult, SectionMetadata};
pub use pattern_detector::{extract_subcommands, extract_agent_types};
pub use routing_detector::detect_routing_patterns;
pub use routing_generator::generate_routing;
pub use interactive_recommender::show_interactive_preview;

/// Main upgrade entry point - converts Agent Skill to progressive disclosure pattern
pub async fn upgrade_skill(skill_path: &Path, options: &UpgradeOptions) -> Result<Option<PreviewData>, Error> {
    // Verify SKILL.md exists
    if !skill_path.exists() {
        return Err(Error::ValidationError(format!(
            "SKILL.md not found at {:?}",
            skill_path
        )));
    }

    // Step 1: Analyze bloat
    let analysis = analyzer::analyze_bloat(skill_path, options)?;

    // Create semantic analyzer (supports multiple providers: Anthropic, OpenAI, Gemini, Copilot)
    let detection = if let Some(ref provider_name) = options.provider {
        // User specified a provider explicitly
        let result = semantic_analyzer::new_analyzer_by_name(provider_name);
        if result.analyzer.is_none() {
            eprintln!("{}", result.error_message());
            return Err(Error::ValidationError(format!(
                "Failed to initialize provider '{}'",
                provider_name
            )));
        }
        result
    } else {
        // Auto-detect using cascade
        semantic_analyzer::new_analyzer()
    };

    // If no analyzer found, print helpful error message
    if detection.analyzer.is_none() {
        eprintln!("{}", detection.error_message());
        eprintln!("\nContinuing with mechanical splitting...\n");
    }

    // Step 2: Split content
    let split_result = splitter::split_content(skill_path, &analysis, detection.analyzer).await?;

    // Step 3: If dry-run, build preview data and return it
    if options.dry_run {
        let preview_data = build_preview_data(&analysis, &split_result);
        print_dry_run_preview(&preview_data);
        return Ok(Some(preview_data));
    }

    // Step 4: Generate inject script
    let reference_list: Vec<String> = split_result.reference_files.keys().cloned().collect();
    let inject_script = generator::generate_inject_script(skill_path, &reference_list)?;

    // Step 5: Write files
    let skill_dir = skill_path
        .parent()
        .ok_or_else(|| Error::ValidationError("Invalid skill path".to_string()))?;

    // Write updated SKILL.md
    fs::write(skill_path, &split_result.core_content)
        .map_err(|e| Error::ValidationError(format!("Failed to write SKILL.md: {}", e)))?;

    // Create references/ directory
    let references_dir = skill_dir.join("references");
    fs::create_dir_all(&references_dir)
        .map_err(|e| Error::ValidationError(format!("Failed to create references/ dir: {}", e)))?;

    // Write reference files
    for (filename, content) in &split_result.reference_files {
        let ref_path = references_dir.join(filename);
        fs::write(&ref_path, content)
            .map_err(|e| Error::ValidationError(format!("Failed to write reference file: {}", e)))?;
    }

    // Create scripts/ directory
    let scripts_dir = skill_dir.join("scripts");
    fs::create_dir_all(&scripts_dir)
        .map_err(|e| Error::ValidationError(format!("Failed to create scripts/ dir: {}", e)))?;

    // Write inject-context script
    if !inject_script.is_empty() {
        let inject_path = scripts_dir.join("inject-context");
        fs::write(&inject_path, inject_script)
            .map_err(|e| Error::ValidationError(format!("Failed to write inject-context script: {}", e)))?;

        // Set executable permissions (Unix only, no-op on Windows)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&inject_path)
                .map_err(|e| Error::ValidationError(format!("Failed to read script metadata: {}", e)))?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&inject_path, perms)
                .map_err(|e| Error::ValidationError(format!("Failed to set script permissions: {}", e)))?;
        }
    }

    Ok(None)
}

/// Build PreviewData from analysis and split results
fn build_preview_data(analysis: &BloatAnalysis, split_result: &SplitResult) -> PreviewData {
    let total_lines = analysis.total_lines;
    let core_lines_after = split_result.core_content.lines().count();

    // Build section previews from sections_metadata
    let sections: Vec<SectionPreview> = split_result
        .sections_metadata
        .iter()
        .map(|meta| SectionPreview {
            name: meta.name.clone(),
            line_range: (meta.start_line, meta.end_line),
            target_file: meta.target_file.clone(),
            timing: meta.timing.clone(),
        })
        .collect();

    // Build breadcrumb previews from runtime sections
    let breadcrumbs: Vec<BreadcrumbPreview> = split_result
        .sections_metadata
        .iter()
        .filter(|meta| meta.timing == SectionTiming::Runtime)
        .map(|meta| BreadcrumbPreview {
            section_name: meta.name.clone(),
            target_file: meta.target_file.clone(),
            condition: meta.condition.clone(),
        })
        .collect();

    // Build reference file previews
    let reference_files: Vec<ReferenceFilePreview> = split_result
        .reference_files
        .iter()
        .map(|(filename, content)| ReferenceFilePreview {
            filename: filename.clone(),
            line_count: content.lines().count(),
        })
        .collect();

    PreviewData {
        total_lines,
        core_lines_after,
        sections,
        reference_files,
        breadcrumbs,
    }
}

/// Prints detailed dry-run preview to stdout
fn print_dry_run_preview(preview: &PreviewData) {
    println!("=== Upgrade Analysis (Dry Run) ===\n");

    // Size impact
    let reduction_pct = if preview.total_lines > 0 {
        ((preview.total_lines - preview.core_lines_after) as f64 / preview.total_lines as f64 * 100.0) as usize
    } else {
        0
    };
    println!("Size impact:");
    println!("  Before: {} lines", preview.total_lines);
    println!("  After: {} lines ({}% reduction)\n", preview.core_lines_after, reduction_pct);

    // Sections to extract
    println!("Sections to extract ({}):", preview.sections.len());
    for section in &preview.sections {
        let timing_tag = match section.timing {
            SectionTiming::Invocation => "[invocation]",
            SectionTiming::Runtime => "[runtime]",
            SectionTiming::Unknown => "[unknown]",
        };
        println!("  {} {} (lines {}-{}) → references/{}",
            timing_tag, section.name, section.line_range.0, section.line_range.1, section.target_file);
    }
    println!();

    // Breadcrumbs
    if !preview.breadcrumbs.is_empty() {
        println!("Breadcrumbs to create ({}):", preview.breadcrumbs.len());
        for breadcrumb in &preview.breadcrumbs {
            let condition_text = breadcrumb.condition.as_ref()
                .map(|c| format!(" {}", c))
                .unwrap_or_default();
            println!("  ## {} — [See references/{}{}]",
                breadcrumb.section_name, breadcrumb.target_file, condition_text);
        }
        println!();
    }

    // Reference files
    println!("Reference files to create ({}):", preview.reference_files.len());
    for ref_file in &preview.reference_files {
        println!("  references/{} ({} lines)", ref_file.filename, ref_file.line_count);
    }
    println!();

    println!("To apply changes, run without --dry-run flag.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_upgrade_skill_dry_run_does_not_write() {
        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path().join("SKILL.md");
        let mut file = fs::File::create(&skill_path).unwrap();
        writeln!(
            file,
            "---\nname: test-skill\ndescription: test\n---\n\nContent"
        )
        .unwrap();

        let options = UpgradeOptions {
            dry_run: true,
            with_agent_references: false,
            interactive: None,
            provider: None,
            ..Default::default()
        };

        let result = upgrade_skill(&skill_path, &options).await;
        assert!(result.is_ok());

        // Should return Some(preview_data) in dry-run mode
        let preview_opt = result.unwrap();
        assert!(preview_opt.is_some());

        // Should not create references/ or scripts/
        assert!(!temp_dir.path().join("references").exists());
        assert!(!temp_dir.path().join("scripts").exists());
    }

    #[tokio::test]
    async fn test_upgrade_skill_dry_run_returns_preview() {
        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path().join("SKILL.md");
        let mut file = fs::File::create(&skill_path).unwrap();
        writeln!(file, "---\nname: test-skill\ndescription: test\n---\n\nContent").unwrap();

        let options = UpgradeOptions {
            dry_run: true,
            with_agent_references: false,
            interactive: None,
            provider: None,
            ..Default::default()
        };

        let result = upgrade_skill(&skill_path, &options).await.unwrap();

        // Should return Some(preview_data) in dry-run mode
        assert!(result.is_some());
        let preview = result.unwrap();
        assert!(preview.total_lines > 0);
    }

    #[tokio::test]
    async fn test_upgrade_skill_creates_directory_structure() {
        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path().join("SKILL.md");
        let mut file = fs::File::create(&skill_path).unwrap();

        // Create content with a section that should be split
        let mut content = String::from("---\nname: test-skill\ndescription: test\nargument-hint: test\n---\n\n");
        content.push_str("## Reference Section\n\n");
        for i in 0..60 {
            content.push_str(&format!("Line {}\n", i));
        }
        writeln!(file, "{}", content).unwrap();

        let options = UpgradeOptions {
            dry_run: false,
            with_agent_references: false,
            interactive: None,
            provider: None,
            ..Default::default()
        };

        let result = upgrade_skill(&skill_path, &options).await;
        assert!(result.is_ok());

        // Should return None in non-dry-run mode
        assert!(result.unwrap().is_none());

        // Should create references/ and scripts/
        assert!(temp_dir.path().join("references").exists());
        assert!(temp_dir.path().join("scripts").exists());
        assert!(temp_dir
            .path()
            .join("scripts")
            .join("inject-context")
            .exists());
    }

    #[tokio::test]
    async fn test_upgrade_skill_writes_reference_files() {
        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path().join("SKILL.md");
        let mut file = fs::File::create(&skill_path).unwrap();

        let mut content = String::from("---\nname: test-skill\ndescription: test\nargument-hint: test\n---\n\n");
        content.push_str("## Implementation Steps\n\n");
        for i in 0..60 {
            content.push_str(&format!("Step {}\n", i));
        }
        writeln!(file, "{}", content).unwrap();

        let options = UpgradeOptions {
            dry_run: false,
            with_agent_references: false,
            interactive: None,
            provider: None,
            ..Default::default()
        };

        let result = upgrade_skill(&skill_path, &options).await;
        assert!(result.is_ok());

        // Should return None in non-dry-run mode
        assert!(result.unwrap().is_none());

        // Should create reference file
        let ref_file = temp_dir
            .path()
            .join("references")
            .join("implementation-steps.md");
        assert!(ref_file.exists());

        // Reference file should have dedup marker
        let ref_content = fs::read_to_string(&ref_file).unwrap();
        assert!(ref_content.starts_with("<!-- injected: references/implementation-steps.md -->"));
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_upgrade_skill_sets_script_executable() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path().join("SKILL.md");
        let mut file = fs::File::create(&skill_path).unwrap();

        let mut content = String::from("---\nname: test-skill\ndescription: test\nargument-hint: test\n---\n\n");
        content.push_str("## Reference Section\n\n");
        for i in 0..60 {
            content.push_str(&format!("Line {}\n", i));
        }
        writeln!(file, "{}", content).unwrap();

        let options = UpgradeOptions {
            dry_run: false,
            with_agent_references: false,
            interactive: None,
            provider: None,
            ..Default::default()
        };

        let result = upgrade_skill(&skill_path, &options).await;
        assert!(result.is_ok());

        // Should return None in non-dry-run mode
        assert!(result.unwrap().is_none());

        // Check script is executable
        let script_path = temp_dir.path().join("scripts").join("inject-context");
        let metadata = fs::metadata(&script_path).unwrap();
        let permissions = metadata.permissions();
        assert_eq!(permissions.mode() & 0o111, 0o111); // At least user-executable
    }

    #[tokio::test]
    async fn test_upgrade_skill_validates_path() {
        let options = UpgradeOptions {
            dry_run: false,
            with_agent_references: false,
            interactive: None,
            provider: None,
            ..Default::default()
        };

        let result = upgrade_skill(Path::new("/nonexistent/SKILL.md"), &options).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::ValidationError(_)));
    }
}
