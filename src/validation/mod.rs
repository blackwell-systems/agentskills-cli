mod base_spec;
mod extensions;
mod progressive_disclosure;

pub use base_spec::validate_base_spec;
pub use extensions::validate_extensions;
pub use progressive_disclosure::validate_progressive_disclosure;

use crate::error::{Error, ValidationResult};
use crate::models::SkillMetadata;
use std::path::Path;

/// Main validation entry point. Runs base spec checks, extension validation,
/// and progressive disclosure checks. Returns aggregated ValidationResult.
pub fn validate_skill(path: &Path) -> Result<ValidationResult, Error> {
    let metadata = SkillMetadata::from_path(path)?;
    let mut result = ValidationResult::new();

    validate_base_spec(&metadata, &mut result);
    validate_extensions(&metadata, &mut result);
    validate_progressive_disclosure(path, &mut result)?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Integration test placeholder - will test with actual fixtures after merge
    // Individual validation functions are tested in their respective modules
}
