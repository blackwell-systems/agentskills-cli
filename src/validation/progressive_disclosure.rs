use crate::error::{Error, Severity, ValidationError, ValidationResult};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Checks progressive disclosure patterns: SKILL.md length (<200 lines),
/// references/ structure, inject-context script existence, dedup markers.
pub fn validate_progressive_disclosure(
    skill_path: &Path,
    result: &mut ValidationResult,
) -> Result<(), Error> {
    let skill_md_path = skill_path.join("SKILL.md");

    // Check SKILL.md length
    if skill_md_path.exists() {
        let content = fs::read_to_string(&skill_md_path)?;
        let line_count = content.lines().count();

        if line_count > 200 {
            result.add_warning(ValidationError {
                error_type: "skill_md_too_long".to_string(),
                message: format!(
                    "SKILL.md has {} lines (>200). Consider running 'agentskills upgrade' to adopt progressive disclosure",
                    line_count
                ),
                file: Some(PathBuf::from("SKILL.md")),
                line: None,
                severity: Severity::Warning,
            });
        }
    }

    let references_dir = skill_path.join("references");
    let has_references_dir = references_dir.exists() && references_dir.is_dir();

    // If references/ exists, validate its structure
    if has_references_dir {
        validate_references_structure(&references_dir, result)?;

        // Check for inject-context script
        let inject_script_path = skill_path.join("scripts").join("inject-context");
        if !inject_script_path.exists() {
            result.add_error(ValidationError {
                error_type: "missing_inject_script".to_string(),
                message:
                    "references/ directory exists but scripts/inject-context script is missing"
                        .to_string(),
                file: None,
                line: None,
                severity: Severity::Error,
            });
        } else {
            validate_inject_script(&inject_script_path, result)?;
        }
    }

    // Check if SKILL.md has triggers frontmatter but no references/ (partial adoption)
    if !has_references_dir && skill_md_path.exists() {
        let content = fs::read_to_string(&skill_md_path)?;
        if has_triggers_frontmatter(&content) {
            result.add_warning(ValidationError {
                error_type: "partial_progressive_disclosure".to_string(),
                message: "SKILL.md has 'triggers:' frontmatter but no references/ directory. Progressive disclosure not fully adopted".to_string(),
                file: Some(PathBuf::from("SKILL.md")),
                line: None,
                severity: Severity::Warning,
            });
        }
    }

    Ok(())
}

/// Validates references/ directory structure
fn validate_references_structure(
    references_dir: &Path,
    result: &mut ValidationResult,
) -> Result<(), Error> {
    for entry in WalkDir::new(references_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Check that all files are .md files
        if path.is_file() {
            let is_md_file = if let Some(ext) = path.extension() {
                if ext != "md" {
                    result.add_warning(ValidationError {
                        error_type: "invalid_reference_file_type".to_string(),
                        message: format!(
                            "File '{}' in references/ should be a .md file",
                            path.file_name().unwrap().to_string_lossy()
                        ),
                        file: Some(path.to_path_buf()),
                        line: None,
                        severity: Severity::Warning,
                    });
                    false
                } else {
                    true
                }
            } else {
                false
            };

            // Check for dedup marker only for .md files
            if is_md_file {
                validate_dedup_marker(path, result)?;
            }
        }
    }

    Ok(())
}

/// Validates that a reference file has the correct dedup marker at the top
fn validate_dedup_marker(file_path: &Path, result: &mut ValidationResult) -> Result<(), Error> {
    let content = fs::read_to_string(file_path)?;
    let filename = file_path.file_name().unwrap().to_string_lossy();
    let expected_marker = format!("<!-- injected: references/{} -->", filename);

    // Check if the first non-empty line contains the exact dedup marker
    if let Some(first_line) = content.lines().next() {
        if first_line.trim() != expected_marker.trim() {
            result.add_error(ValidationError {
                error_type: "invalid_dedup_marker".to_string(),
                message: format!(
                    "File '{}' missing or has incorrect dedup marker. Expected: '{}'",
                    filename, expected_marker
                ),
                file: Some(file_path.to_path_buf()),
                line: Some(1),
                severity: Severity::Error,
            });
        }
    } else {
        result.add_error(ValidationError {
            error_type: "empty_reference_file".to_string(),
            message: format!("Reference file '{}' is empty", filename),
            file: Some(file_path.to_path_buf()),
            line: None,
            severity: Severity::Error,
        });
    }

    Ok(())
}

/// Validates the inject-context script
fn validate_inject_script(script_path: &Path, result: &mut ValidationResult) -> Result<(), Error> {
    let content = fs::read_to_string(script_path)?;

    // Check for shebang
    if !content.starts_with("#!/usr/bin/env bash") {
        result.add_error(ValidationError {
            error_type: "missing_shebang".to_string(),
            message: "scripts/inject-context must start with shebang '#!/usr/bin/env bash'"
                .to_string(),
            file: Some(script_path.to_path_buf()),
            line: Some(1),
            severity: Severity::Error,
        });
    }

    // Check that script references the references/ directory
    if !content.contains("references/") {
        result.add_warning(ValidationError {
            error_type: "no_references_in_script".to_string(),
            message: "scripts/inject-context does not reference 'references/' directory"
                .to_string(),
            file: Some(script_path.to_path_buf()),
            line: None,
            severity: Severity::Warning,
        });
    }

    // Check if script is executable (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(script_path)?;
        let permissions = metadata.permissions();
        let mode = permissions.mode();

        // Check if owner execute bit is set (0o100)
        if mode & 0o100 == 0 {
            result.add_warning(ValidationError {
                error_type: "script_not_executable".to_string(),
                message:
                    "scripts/inject-context is not executable. Run: chmod +x scripts/inject-context"
                        .to_string(),
                file: Some(script_path.to_path_buf()),
                line: None,
                severity: Severity::Warning,
            });
        }
    }

    Ok(())
}

/// Checks if content has triggers frontmatter
fn has_triggers_frontmatter(content: &str) -> bool {
    // Look for YAML frontmatter between --- delimiters
    if !content.starts_with("---") {
        return false;
    }

    let lines: Vec<&str> = content.lines().collect();
    if lines.len() < 3 {
        return false;
    }

    // Find closing ---
    if let Some(closing_pos) = lines.iter().skip(1).position(|&line| line.trim() == "---") {
        let frontmatter = &lines[1..=closing_pos];
        return frontmatter
            .iter()
            .any(|line| line.trim().starts_with("triggers:"));
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_skill_md_under_200_lines() {
        let temp_dir = TempDir::new().unwrap();
        let skill_md = temp_dir.path().join("SKILL.md");
        fs::write(&skill_md, "---\nname: test\n---\n# Test\nShort content").unwrap();

        let mut result = ValidationResult::new();
        validate_progressive_disclosure(temp_dir.path(), &mut result).unwrap();

        assert!(result.is_valid());
        assert_eq!(result.warnings.len(), 0);
    }

    #[test]
    fn test_skill_md_over_200_lines() {
        let temp_dir = TempDir::new().unwrap();
        let skill_md = temp_dir.path().join("SKILL.md");
        let content = (0..250)
            .map(|i| format!("Line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&skill_md, content).unwrap();

        let mut result = ValidationResult::new();
        validate_progressive_disclosure(temp_dir.path(), &mut result).unwrap();

        assert!(result.is_valid()); // Warning, not error
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].message.contains("250 lines"));
    }

    #[test]
    fn test_missing_inject_script_when_references_exist() {
        let temp_dir = TempDir::new().unwrap();
        let references_dir = temp_dir.path().join("references");
        fs::create_dir(&references_dir).unwrap();
        let ref_file = references_dir.join("test.md");
        fs::write(&ref_file, "<!-- injected: references/test.md -->\nContent").unwrap();

        let mut result = ValidationResult::new();
        validate_progressive_disclosure(temp_dir.path(), &mut result).unwrap();

        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| e.error_type == "missing_inject_script"));
    }

    #[test]
    fn test_valid_dedup_marker() {
        let temp_dir = TempDir::new().unwrap();
        let references_dir = temp_dir.path().join("references");
        fs::create_dir_all(&references_dir).unwrap();
        let ref_file = references_dir.join("test-ref.md");
        fs::write(
            &ref_file,
            "<!-- injected: references/test-ref.md -->\nSome content",
        )
        .unwrap();

        let scripts_dir = temp_dir.path().join("scripts");
        fs::create_dir_all(&scripts_dir).unwrap();
        let inject_script = scripts_dir.join("inject-context");
        fs::write(&inject_script, "#!/usr/bin/env bash\ncat references/*.md").unwrap();

        let mut result = ValidationResult::new();
        validate_progressive_disclosure(temp_dir.path(), &mut result).unwrap();

        assert!(result.is_valid());
    }

    #[test]
    fn test_invalid_dedup_marker() {
        let temp_dir = TempDir::new().unwrap();
        let references_dir = temp_dir.path().join("references");
        fs::create_dir_all(&references_dir).unwrap();
        let ref_file = references_dir.join("test.md");
        fs::write(&ref_file, "<!-- wrong marker -->\nContent").unwrap();

        let scripts_dir = temp_dir.path().join("scripts");
        fs::create_dir_all(&scripts_dir).unwrap();
        let inject_script = scripts_dir.join("inject-context");
        fs::write(&inject_script, "#!/usr/bin/env bash\necho references/").unwrap();

        let mut result = ValidationResult::new();
        validate_progressive_disclosure(temp_dir.path(), &mut result).unwrap();

        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| e.error_type == "invalid_dedup_marker"));
    }

    #[test]
    fn test_missing_shebang() {
        let temp_dir = TempDir::new().unwrap();
        let references_dir = temp_dir.path().join("references");
        fs::create_dir_all(&references_dir).unwrap();

        let scripts_dir = temp_dir.path().join("scripts");
        fs::create_dir_all(&scripts_dir).unwrap();
        let inject_script = scripts_dir.join("inject-context");
        fs::write(&inject_script, "echo references/").unwrap();

        let mut result = ValidationResult::new();
        validate_progressive_disclosure(temp_dir.path(), &mut result).unwrap();

        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| e.error_type == "missing_shebang"));
    }

    #[test]
    fn test_triggers_without_references() {
        let temp_dir = TempDir::new().unwrap();
        let skill_md = temp_dir.path().join("SKILL.md");
        fs::write(
            &skill_md,
            "---\nname: test\ntriggers:\n  - /test\n---\nContent",
        )
        .unwrap();

        let mut result = ValidationResult::new();
        validate_progressive_disclosure(temp_dir.path(), &mut result).unwrap();

        assert!(result.is_valid()); // Warning, not error
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].error_type == "partial_progressive_disclosure");
    }

    #[test]
    fn test_has_triggers_frontmatter() {
        let content = "---\nname: test\ntriggers:\n  - /test\n---\nBody";
        assert!(has_triggers_frontmatter(content));

        let no_triggers = "---\nname: test\n---\nBody";
        assert!(!has_triggers_frontmatter(no_triggers));

        let no_frontmatter = "# Just a heading";
        assert!(!has_triggers_frontmatter(no_frontmatter));
    }

    #[test]
    fn test_non_md_file_in_references() {
        let temp_dir = TempDir::new().unwrap();
        let references_dir = temp_dir.path().join("references");
        fs::create_dir_all(&references_dir).unwrap();
        fs::write(references_dir.join("test.txt"), "text file").unwrap();

        let scripts_dir = temp_dir.path().join("scripts");
        fs::create_dir_all(&scripts_dir).unwrap();
        let inject_script = scripts_dir.join("inject-context");
        fs::write(&inject_script, "#!/usr/bin/env bash\necho references/").unwrap();

        let mut result = ValidationResult::new();
        validate_progressive_disclosure(temp_dir.path(), &mut result).unwrap();

        assert!(result.is_valid()); // Warning, not error
        assert!(result
            .warnings
            .iter()
            .any(|w| w.error_type == "invalid_reference_file_type"));
    }
}
