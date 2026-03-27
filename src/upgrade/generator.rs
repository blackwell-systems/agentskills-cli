use crate::models::Error;
use std::fs;
use std::path::Path;

/// Bundles the SAW inject-agent-context script for advanced progressive disclosure.
///
/// This function replaces the previous simple script generator with the full-featured
/// SAW inject-agent-context script, which supports:
/// - YAML frontmatter parsing from SKILL.md
/// - Agent-type-specific reference injection
/// - Conditional injection via `when:` patterns
/// - Deduplication markers to prevent double injection
///
/// The bundled script is production-tested in the Scout-and-Wave protocol.
pub fn generate_inject_script(
    _skill_path: &Path,
    _reference_files: &[String],
) -> Result<String, Error> {
    bundle_saw_script()
}

fn bundle_saw_script() -> Result<String, Error> {
    let saw_script_path = Path::new("/Users/dayna.blackwell/code/scout-and-wave/implementations/claude-code/prompts/scripts/inject-agent-context");

    if !saw_script_path.exists() {
        return Err(Error::IoError(
            format!("SAW inject-agent-context script not found at {:?}", saw_script_path)
        ));
    }

    let script_content = fs::read_to_string(saw_script_path)
        .map_err(|e| Error::IoError(format!("Failed to read SAW script: {}", e)))?;

    Ok(script_content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_generate_inject_script_creates_valid_bash() {
        let refs = vec!["ref1.md".to_string(), "ref2.md".to_string()];
        let result = generate_inject_script(&PathBuf::from("/tmp/skill"), &refs).unwrap();

        // Should have shebang
        assert!(result.starts_with("#!/usr/bin/env bash"));

        // Should have SAW script features
        assert!(result.contains("inject-agent-context"));
        assert!(result.contains("agent-references"));
    }

    #[test]
    fn test_generate_inject_script_handles_skill_dir() {
        let refs = vec!["ref1.md".to_string()];
        let result = generate_inject_script(&PathBuf::from("/tmp/skill"), &refs).unwrap();

        // Should detect SKILL_DIR relative to script location
        assert!(result.contains("SKILL_DIR="));
        assert!(result.contains("dirname"));
    }

    #[test]
    fn test_bundle_saw_script() {
        let result = bundle_saw_script();

        // Should succeed (SAW script exists)
        assert!(result.is_ok());

        let script = result.unwrap();

        // Should have shebang
        assert!(script.starts_with("#!/usr/bin/env bash"));

        // Should contain SAW script features
        assert!(script.contains("inject-agent-context"));
        assert!(script.contains("--type"));
        assert!(script.contains("--prompt"));
        assert!(script.contains("agent-references"));
        assert!(script.contains("SKILL_DIR"));
    }
}
