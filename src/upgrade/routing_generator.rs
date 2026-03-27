use crate::error::Error;
use crate::models::{
    InlineBreadcrumb, RoutingDetectionResult, RoutingOutput, RoutingStyle, SectionTiming,
};
use std::collections::HashMap;

/// Takes detection results and selected style, generates all routing artifacts.
/// Returns RoutingOutput with table, breadcrumbs, and back-link headers.
///
/// # Arguments
/// * `detection` - Routing detection results from routing_detector
/// * `style` - Selected routing style (Table, Inline, Smart, or None)
/// * `skill_name` - Name of the skill (used in routing table triggers)
///
/// # Returns
/// RoutingOutput with routing_table (if table style), inline_breadcrumbs (if inline),
/// and back_link_headers for all reference files
pub fn generate_routing(
    detection: &RoutingDetectionResult,
    style: RoutingStyle,
    skill_name: &str,
) -> Result<RoutingOutput, Error> {
    // Resolve Smart style to concrete style
    let resolved_style = match style {
        RoutingStyle::Smart => detection.recommended_style.clone(),
        other => other,
    };

    // Generate routing table if table style
    let routing_table = if resolved_style == RoutingStyle::Table {
        Some(generate_routing_table(detection, skill_name))
    } else {
        None
    };

    // Generate inline breadcrumbs if not table style and not None
    let inline_breadcrumbs = if resolved_style != RoutingStyle::Table
        && resolved_style != RoutingStyle::None
    {
        generate_inline_breadcrumbs(detection)
    } else {
        vec![]
    };

    // Generate back-link headers for reference files
    let back_link_headers = generate_back_link_headers(detection);

    Ok(RoutingOutput {
        routing_table,
        inline_breadcrumbs,
        back_link_headers,
    })
}

/// Generate routing table in markdown format
///
/// Format: | Subcommand | Reference file |
///
/// Example:
/// ```markdown
/// | Subcommand | Reference file |
/// |------------|----------------|
/// | /saw scout | references/scout.md |
/// | /saw wave  | references/wave.md |
/// ```
fn generate_routing_table(detection: &RoutingDetectionResult, skill_name: &str) -> String {
    let mut table = String::new();
    table.push_str("| Subcommand | Reference file |\n");
    table.push_str("|------------|----------------|\n");

    for subcommand in &detection.subcommands {
        // Generate reference file path from subcommand
        let ref_file = format!("references/{}.md", subcommand.to_lowercase());
        // Generate trigger using skill name
        let trigger = format!("/{} {}", skill_name, subcommand);
        table.push_str(&format!("| {} | {} |\n", trigger, ref_file));
    }

    table
}

/// Generate inline breadcrumbs for runtime sections
///
/// Format: ## Section — [See references/file.md when X]
///
/// Only generates breadcrumbs for Runtime timing sections.
/// Invocation sections are assumed to be read at startup.
fn generate_inline_breadcrumbs(detection: &RoutingDetectionResult) -> Vec<InlineBreadcrumb> {
    let mut breadcrumbs = Vec::new();

    for section in &detection.timing_sections {
        // Only generate breadcrumbs for Runtime sections
        // Unknown timing sections are treated as Runtime (conservative approach)
        if section.timing == SectionTiming::Runtime || section.timing == SectionTiming::Unknown {
            // Generate reference file path from section name
            let ref_file = format!(
                "references/{}.md",
                section.name.to_lowercase().replace(' ', "-")
            );

            // Use trigger pattern if available, otherwise generic condition
            let condition = section
                .trigger_pattern
                .clone()
                .or_else(|| Some("needed at runtime".to_string()));

            breadcrumbs.push(InlineBreadcrumb {
                section_name: section.name.clone(),
                reference_file: ref_file,
                condition,
            });
        }
    }

    breadcrumbs
}

/// Generate back-link headers with core reference
///
/// Format: <!-- Core flow: see SKILL.md section Y -->
///
/// Creates a back-link header for each reference file pointing back to the
/// corresponding section in the core SKILL.md file.
fn generate_back_link_headers(detection: &RoutingDetectionResult) -> HashMap<String, String> {
    let mut headers = HashMap::new();

    // Generate headers for subcommand reference files
    for subcommand in &detection.subcommands {
        let ref_file = format!("references/{}.md", subcommand.to_lowercase());
        let header = format!(
            "<!-- Core flow: see SKILL.md section {} -->",
            subcommand
        );
        headers.insert(ref_file, header);
    }

    // Generate headers for timing section reference files
    for section in &detection.timing_sections {
        let ref_file = format!(
            "references/{}.md",
            section.name.to_lowercase().replace(' ', "-")
        );
        let header = format!(
            "<!-- Core flow: see SKILL.md section {} -->",
            section.name
        );
        headers.insert(ref_file, header);
    }

    headers
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TimingSection;

    #[test]
    fn test_generate_routing_table_format() {
        let detection = RoutingDetectionResult {
            subcommands: vec!["scout".to_string(), "wave".to_string()],
            agent_types: vec![],
            timing_sections: vec![],
            recommended_style: RoutingStyle::Table,
        };

        let table = generate_routing_table(&detection, "saw");

        assert!(table.contains("| Subcommand | Reference file |"));
        assert!(table.contains("| /saw scout | references/scout.md |"));
        assert!(table.contains("| /saw wave | references/wave.md |"));
    }

    #[test]
    fn test_generate_inline_breadcrumbs_for_runtime_sections() {
        let detection = RoutingDetectionResult {
            subcommands: vec![],
            agent_types: vec![],
            timing_sections: vec![
                TimingSection {
                    name: "Error Handling".to_string(),
                    timing: SectionTiming::Runtime,
                    trigger_pattern: Some("when error occurs".to_string()),
                },
                TimingSection {
                    name: "Setup Instructions".to_string(),
                    timing: SectionTiming::Invocation,
                    trigger_pattern: None,
                },
            ],
            recommended_style: RoutingStyle::Inline,
        };

        let breadcrumbs = generate_inline_breadcrumbs(&detection);

        // Should only generate breadcrumb for Runtime section
        assert_eq!(breadcrumbs.len(), 1);
        assert_eq!(breadcrumbs[0].section_name, "Error Handling");
        assert_eq!(breadcrumbs[0].reference_file, "references/error-handling.md");
        assert_eq!(breadcrumbs[0].condition, Some("when error occurs".to_string()));
    }

    #[test]
    fn test_generate_back_link_headers_with_core_reference() {
        let detection = RoutingDetectionResult {
            subcommands: vec!["scout".to_string()],
            agent_types: vec![],
            timing_sections: vec![TimingSection {
                name: "Error Handling".to_string(),
                timing: SectionTiming::Runtime,
                trigger_pattern: None,
            }],
            recommended_style: RoutingStyle::Inline,
        };

        let headers = generate_back_link_headers(&detection);

        assert_eq!(headers.len(), 2);
        assert_eq!(
            headers.get("references/scout.md"),
            Some(&"<!-- Core flow: see SKILL.md section scout -->".to_string())
        );
        assert_eq!(
            headers.get("references/error-handling.md"),
            Some(&"<!-- Core flow: see SKILL.md section Error Handling -->".to_string())
        );
    }

    #[test]
    fn test_generate_routing_none_style_returns_empty() {
        let detection = RoutingDetectionResult {
            subcommands: vec!["scout".to_string(), "wave".to_string()],
            agent_types: vec![],
            timing_sections: vec![],
            recommended_style: RoutingStyle::Table,
        };

        let output = generate_routing(&detection, RoutingStyle::None, "saw").unwrap();

        assert!(output.routing_table.is_none());
        assert!(output.inline_breadcrumbs.is_empty());
        // Back-link headers are still generated even with None style
        assert!(!output.back_link_headers.is_empty());
    }

    #[test]
    fn test_routing_table_uses_skill_name_in_triggers() {
        let detection = RoutingDetectionResult {
            subcommands: vec!["test".to_string()],
            agent_types: vec![],
            timing_sections: vec![],
            recommended_style: RoutingStyle::Table,
        };

        let output = generate_routing(&detection, RoutingStyle::Table, "myskill").unwrap();

        let table = output.routing_table.unwrap();
        assert!(table.contains("/myskill test"));
    }
}
