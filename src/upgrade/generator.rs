use crate::error::Error;
use std::fs;
use std::path::Path;

/// Bundles the inject-agent-context script for advanced progressive disclosure.
///
/// This function replaces the previous simple script generator with the full-featured
/// inject-agent-context script, which supports:
/// - YAML frontmatter parsing from SKILL.md
/// - Agent-type-specific reference injection
/// - Conditional injection via `when:` patterns
/// - Deduplication markers to prevent double injection
///
/// The bundled script is production-tested and vendor-neutral.
pub fn generate_inject_script(
    _skill_path: &Path,
    _reference_files: &[String],
) -> Result<String, Error> {
    bundle_inject_script()
}

fn bundle_inject_script() -> Result<String, Error> {
    let script_path = Path::new("/Users/dayna.blackwell/code/scout-and-wave/implementations/claude-code/prompts/scripts/inject-agent-context");

    if !script_path.exists() {
        return Err(Error::ValidationError(
            format!("inject-agent-context script not found at {:?}", script_path)
        ));
    }

    let script_content = fs::read_to_string(script_path)
        .map_err(|e| Error::ValidationError(format!("Failed to read inject-agent-context script: {}", e)))?;

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

        // Should have inject script features
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
    fn test_bundle_inject_script() {
        let result = bundle_inject_script();

        // Should succeed (inject script exists)
        assert!(result.is_ok());

        let script = result.unwrap();

        // Should have shebang
        assert!(script.starts_with("#!/usr/bin/env bash"));

        // Should contain inject script features
        assert!(script.contains("inject-agent-context"));
        assert!(script.contains("--type"));
        assert!(script.contains("--prompt"));
        assert!(script.contains("agent-references"));
        assert!(script.contains("SKILL_DIR"));
    }
}
