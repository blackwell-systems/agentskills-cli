use crate::models::{Error, UpgradeOptions};
use std::fs;
use std::path::Path;

pub mod analyzer;
pub mod generator;
pub mod splitter;
pub mod pattern_detector;
pub mod semantic_analyzer;
pub mod routing_graph;
pub mod frontmatter_gen;

pub use analyzer::{analyze_bloat, BloatAnalysis, SplitSuggestion};
pub use generator::generate_inject_script;
pub use splitter::{split_content, SplitResult};
pub use pattern_detector::{extract_subcommands, extract_agent_types};

/// Main upgrade entry point - converts Agent Skill to progressive disclosure pattern
pub async fn upgrade_skill(skill_path: &Path, options: &UpgradeOptions) -> Result<(), Error> {
    // Verify SKILL.md exists
    if !skill_path.exists() {
        return Err(Error::ValidationError(format!(
            "SKILL.md not found at {:?}",
            skill_path
        )));
    }

    // Step 1: Analyze bloat
    let analysis = analyzer::analyze_bloat(skill_path, options)?;

    // Step 2: If dry-run, print analysis and exit
    if options.dry_run {
        print_dry_run_analysis(&analysis);
        return Ok(());
    }

    // Read API key from environment for semantic analysis
    let api_key = std::env::var("ANTHROPIC_API_KEY").ok();

    // Step 3: Split content
    let split_result = splitter::split_content(skill_path, &analysis, api_key).await?;

    // Step 4: Generate inject script
    let reference_list: Vec<String> = split_result.reference_files.keys().cloned().collect();
    let inject_script = generator::generate_inject_script(skill_path, &reference_list)?;

    // Step 5: Write files
    let skill_dir = skill_path
        .parent()
        .ok_or_else(|| Error::ValidationError("Invalid skill path".to_string()))?;

    // Write updated SKILL.md
    fs::write(skill_path, &split_result.core_content)
        .map_err(|e| Error::IoError(format!("Failed to write SKILL.md: {}", e)))?;

    // Create references/ directory
    let references_dir = skill_dir.join("references");
    fs::create_dir_all(&references_dir)
        .map_err(|e| Error::IoError(format!("Failed to create references/ dir: {}", e)))?;

    // Write reference files
    for (filename, content) in &split_result.reference_files {
        let ref_path = references_dir.join(filename);
        fs::write(&ref_path, content)
            .map_err(|e| Error::IoError(format!("Failed to write reference file: {}", e)))?;
    }

    // Create scripts/ directory
    let scripts_dir = skill_dir.join("scripts");
    fs::create_dir_all(&scripts_dir)
        .map_err(|e| Error::IoError(format!("Failed to create scripts/ dir: {}", e)))?;

    // Write inject-context script
    if !inject_script.is_empty() {
        let inject_path = scripts_dir.join("inject-context");
        fs::write(&inject_path, inject_script)
            .map_err(|e| Error::IoError(format!("Failed to write inject-context script: {}", e)))?;

        // Set executable permissions (Unix only, no-op on Windows)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&inject_path)
                .map_err(|e| Error::IoError(format!("Failed to read script metadata: {}", e)))?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&inject_path, perms)
                .map_err(|e| Error::IoError(format!("Failed to set script permissions: {}", e)))?;
        }
    }

    Ok(())
}

/// Prints dry-run analysis to stdout
fn print_dry_run_analysis(analysis: &BloatAnalysis) {
    println!("=== Upgrade Analysis (Dry Run) ===\n");
    println!("Total lines: {}", analysis.total_lines);
    println!();

    if analysis.total_lines > 200 {
        println!("⚠️  SKILL.md exceeds 200 lines (progressive disclosure threshold)");
    } else {
        println!("✓ SKILL.md is within 200-line threshold");
    }
    println!();

    println!("Suggested splits: {}", analysis.suggested_splits.len());
    for suggestion in &analysis.suggested_splits {
        println!(
            "  - {} (lines {}-{}) → references/{}",
            suggestion.section_name,
            suggestion.start_line,
            suggestion.end_line,
            suggestion.target_file
        );
    }
    println!();

    println!("Generated triggers:");
    for trigger in &analysis.trigger_patterns {
        println!("  - {}", trigger);
    }
    println!();

    if analysis.needs_agent_references {
        println!("✓ Will add agent-references field to frontmatter");
    }

    println!("\nTo apply changes, run without --dry-run flag.");
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
        };

        let result = upgrade_skill(&skill_path, &options).await;
        assert!(result.is_ok());

        // Should not create references/ or scripts/
        assert!(!temp_dir.path().join("references").exists());
        assert!(!temp_dir.path().join("scripts").exists());
    }

    #[tokio::test]
    async fn test_upgrade_skill_creates_directory_structure() {
        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path().join("SKILL.md");
        let mut file = fs::File::create(&skill_path).unwrap();

        // Create content with a section that should be split
        let mut content = String::from("---\nname: test-skill\ndescription: test\n---\n\n");
        content.push_str("## Reference Section\n\n");
        for i in 0..60 {
            content.push_str(&format!("Line {}\n", i));
        }
        writeln!(file, "{}", content).unwrap();

        let options = UpgradeOptions {
            dry_run: false,
            with_agent_references: false,
        };

        let result = upgrade_skill(&skill_path, &options).await;
        assert!(result.is_ok());

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

        let mut content = String::from("---\nname: test-skill\ndescription: test\n---\n\n");
        content.push_str("## Implementation Steps\n\n");
        for i in 0..60 {
            content.push_str(&format!("Step {}\n", i));
        }
        writeln!(file, "{}", content).unwrap();

        let options = UpgradeOptions {
            dry_run: false,
            with_agent_references: false,
        };

        let result = upgrade_skill(&skill_path, &options).await;
        assert!(result.is_ok());

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

        let mut content = String::from("---\nname: test-skill\ndescription: test\n---\n\n");
        content.push_str("## Reference Section\n\n");
        for i in 0..60 {
            content.push_str(&format!("Line {}\n", i));
        }
        writeln!(file, "{}", content).unwrap();

        let options = UpgradeOptions {
            dry_run: false,
            with_agent_references: false,
        };

        let result = upgrade_skill(&skill_path, &options).await;
        assert!(result.is_ok());

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
        };

        let result = upgrade_skill(Path::new("/nonexistent/SKILL.md"), &options).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::ValidationError(_)));
    }
}
