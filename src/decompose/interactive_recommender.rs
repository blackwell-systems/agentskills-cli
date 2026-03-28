use crate::error::Error;
use crate::models::{RoutingDetectionResult, RoutingStyle, SectionTiming, DecomposeOptions};
use std::io::{self, Write};

/// Displays routing detection results and recommendations, gets user confirmation
/// before applying changes. Returns user's selected routing style or None if cancelled.
///
/// # Arguments
/// * `detection` - Routing detection results from detect_routing_patterns()
/// * `options` - Upgrade options (skip prompt if routing_style already set)
///
/// # Returns
/// * `Ok(Some(style))` - User selected or pre-configured routing style
/// * `Ok(None)` - User cancelled (don't apply routing)
/// * `Err(...)` - I/O error reading user input
///
/// # Constraints
/// - Display timing only if options.show_timing is true
/// - Skip prompt if options.routing_style is already set
/// - Use eprintln! for preview output (stderr)
pub fn show_interactive_preview(
    detection: &RoutingDetectionResult,
    options: &DecomposeOptions,
) -> Result<Option<RoutingStyle>, Error> {
    // Display detection summary
    eprintln!("\n--- Routing Analysis ---");
    eprintln!(
        "Detected {} subcommands: {:?}",
        detection.subcommands.len(),
        detection.subcommands
    );
    eprintln!(
        "Detected {} agent types: {:?}",
        detection.agent_types.len(),
        detection.agent_types
    );

    // Display timing sections if requested
    if options.show_timing {
        eprintln!("\nTiming sections:");
        for section in &detection.timing_sections {
            let timing_label = match section.timing {
                SectionTiming::Invocation => "invocation",
                SectionTiming::Runtime => "runtime",
                SectionTiming::Unknown => "unknown",
            };
            eprintln!("  - {} [{}]", section.name, timing_label);
        }
    }

    eprintln!(
        "\nRecommended routing style: {:?}",
        detection.recommended_style
    );

    // If user specified style via flag, skip prompt
    if let Some(ref style) = options.routing_style {
        eprintln!("\nUsing pre-configured routing style: {:?}", style);
        return Ok(Some(style.clone()));
    }

    // Prompt user for confirmation or custom selection
    eprint!("\nApply recommended routing? [y/N/custom]: ");
    io::stderr().flush().map_err(|e| {
        Error::ValidationError(format!("Failed to flush stderr: {}", e))
    })?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| {
        Error::ValidationError(format!("Failed to read user input: {}", e))
    })?;

    let choice = input.trim().to_lowercase();

    match choice.as_str() {
        "y" | "yes" => {
            eprintln!("Applying recommended routing style: {:?}", detection.recommended_style);
            Ok(Some(detection.recommended_style.clone()))
        }
        "n" | "no" | "" => {
            eprintln!("Routing generation cancelled.");
            Ok(None)
        }
        "custom" => {
            // Prompt for custom style selection
            eprintln!("\nSelect routing style:");
            eprintln!("  1. Smart (auto-select based on subcommand count)");
            eprintln!("  2. Inline (inline breadcrumbs)");
            eprintln!("  3. Table (routing table)");
            eprintln!("  4. None (no routing)");
            eprint!("Enter choice [1-4]: ");
            io::stderr().flush().map_err(|e| {
                Error::ValidationError(format!("Failed to flush stderr: {}", e))
            })?;

            let mut custom_input = String::new();
            io::stdin().read_line(&mut custom_input).map_err(|e| {
                Error::ValidationError(format!("Failed to read custom input: {}", e))
            })?;

            let custom_choice = custom_input.trim();
            let selected_style = match custom_choice {
                "1" => RoutingStyle::Smart,
                "2" => RoutingStyle::Inline,
                "3" => RoutingStyle::Table,
                "4" => RoutingStyle::None,
                _ => {
                    eprintln!("Invalid choice. Cancelling.");
                    return Ok(None);
                }
            };

            eprintln!("Selected routing style: {:?}", selected_style);
            Ok(Some(selected_style))
        }
        _ => {
            eprintln!("Invalid input. Cancelling.");
            Ok(None)
        }
    }
}

/// Formats detection summary as a human-readable string
///
/// Used by tests and can be used by CLI for non-interactive display
pub fn format_detection_summary(detection: &RoutingDetectionResult) -> String {
    let mut summary = String::new();
    summary.push_str(&format!("Subcommands: {}\n", detection.subcommands.len()));
    summary.push_str(&format!("Agent types: {}\n", detection.agent_types.len()));
    summary.push_str(&format!("Timing sections: {}\n", detection.timing_sections.len()));
    summary.push_str(&format!("Recommended style: {:?}\n", detection.recommended_style));
    summary
}

/// Formats recommendation rationale based on detection results
///
/// Explains why a particular routing style was recommended
pub fn format_recommendation_rationale(detection: &RoutingDetectionResult) -> String {
    let subcommand_count = detection.subcommands.len();

    if subcommand_count >= 4 {
        format!(
            "Table routing recommended: {} subcommands detected (threshold: 4+)",
            subcommand_count
        )
    } else if subcommand_count > 0 {
        format!(
            "Inline routing recommended: {} subcommands detected (threshold: <4)",
            subcommand_count
        )
    } else {
        "No subcommands detected, inline routing recommended as default".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TimingSection;

    #[test]
    fn test_format_detection_summary() {
        let detection = RoutingDetectionResult {
            subcommands: vec!["cmd1".to_string(), "cmd2".to_string()],
            agent_types: vec!["scout".to_string(), "wave".to_string()],
            timing_sections: vec![
                TimingSection {
                    name: "Setup".to_string(),
                    timing: SectionTiming::Invocation,
                    trigger_pattern: None,
                },
            ],
            recommended_style: RoutingStyle::Inline,
        };

        let summary = format_detection_summary(&detection);

        assert!(summary.contains("Subcommands: 2"));
        assert!(summary.contains("Agent types: 2"));
        assert!(summary.contains("Timing sections: 1"));
        assert!(summary.contains("Recommended style: Inline"));
    }

    #[test]
    fn test_skip_prompt_when_routing_style_specified() {
        let detection = RoutingDetectionResult {
            subcommands: vec!["cmd1".to_string()],
            agent_types: vec![],
            timing_sections: vec![],
            recommended_style: RoutingStyle::Inline,
        };

        let options = DecomposeOptions {
            routing_style: Some(RoutingStyle::Table),
            ..Default::default()
        };

        let result = show_interactive_preview(&detection, &options).unwrap();

        // Should return the pre-configured style without prompting
        assert_eq!(result, Some(RoutingStyle::Table));
    }

    #[test]
    fn test_format_recommendation_rationale() {
        // Test table recommendation (4+ subcommands)
        let detection_table = RoutingDetectionResult {
            subcommands: vec![
                "cmd1".to_string(),
                "cmd2".to_string(),
                "cmd3".to_string(),
                "cmd4".to_string(),
            ],
            agent_types: vec![],
            timing_sections: vec![],
            recommended_style: RoutingStyle::Table,
        };

        let rationale_table = format_recommendation_rationale(&detection_table);
        assert!(rationale_table.contains("Table routing recommended"));
        assert!(rationale_table.contains("4 subcommands"));
        assert!(rationale_table.contains("threshold: 4+"));

        // Test inline recommendation (<4 subcommands)
        let detection_inline = RoutingDetectionResult {
            subcommands: vec!["cmd1".to_string(), "cmd2".to_string()],
            agent_types: vec![],
            timing_sections: vec![],
            recommended_style: RoutingStyle::Inline,
        };

        let rationale_inline = format_recommendation_rationale(&detection_inline);
        assert!(rationale_inline.contains("Inline routing recommended"));
        assert!(rationale_inline.contains("2 subcommands"));
        assert!(rationale_inline.contains("threshold: <4"));

        // Test no subcommands
        let detection_none = RoutingDetectionResult {
            subcommands: vec![],
            agent_types: vec![],
            timing_sections: vec![],
            recommended_style: RoutingStyle::Inline,
        };

        let rationale_none = format_recommendation_rationale(&detection_none);
        assert!(rationale_none.contains("No subcommands detected"));
        assert!(rationale_none.contains("inline routing recommended as default"));
    }
}
