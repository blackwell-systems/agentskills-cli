use crate::models::{RoutingGraph, RoutingNode};
use crate::upgrade::semantic_analyzer::{SectionIntent, TriggerTiming};

/// Build routing graph from pattern detector output and semantic analysis results
///
/// Takes:
/// - `subcommands`: Vec<String> from PatternDetector::extract_subcommands
/// - `agent_types`: Vec<String> from PatternDetector::extract_agent_types
/// - `sections`: Vec<(ref_file, section_header, intent)> from semantic analysis
///
/// Returns RoutingGraph with nodes array. Each node maps one reference file
/// to its routing metadata (triggers, agent_types, condition_pattern).
pub fn build(
    _subcommands: &[String],
    _agent_types: &[String],
    sections: &[(String, String, SectionIntent)],
) -> RoutingGraph {
    let mut nodes = Vec::new();

    for (ref_file, _section_header, intent) in sections {
        // Skip runtime-triggered sections - they're handled by breadcrumbs in core
        if matches!(intent.trigger_timing, Some(TriggerTiming::Runtime)) {
            continue;
        }

        let trigger_patterns = if intent.is_command_specific {
            vec![format!("^/saw {}", intent.command.as_ref().unwrap())]
        } else {
            vec![]
        };

        let agent_types_for_node = if intent.is_agent_specific {
            vec![intent.agent_type.as_ref().unwrap().clone()]
        } else {
            vec![]
        };

        let condition_pattern = if intent.is_conditional {
            intent.condition_pattern.clone()
        } else {
            None
        };

        nodes.push(RoutingNode {
            reference_file: ref_file.clone(),
            trigger_patterns,
            agent_types: agent_types_for_node,
            condition_pattern,
        });
    }

    RoutingGraph { nodes }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_command_specific_node() {
        let sections = vec![(
            "references/scout-details.md".to_string(),
            "Scout Details".to_string(),
            SectionIntent {
                is_command_specific: true,
                command: Some("scout".to_string()),
                is_agent_specific: false,
                agent_type: None,
                is_conditional: false,
                condition_pattern: None,
                trigger_timing: None,
                reasoning: "Section describes scout command usage".to_string(),
            },
        )];

        let graph = build(&[], &[], &sections);

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].reference_file, "references/scout-details.md");
        assert_eq!(graph.nodes[0].trigger_patterns, vec!["^/saw scout"]);
        assert!(graph.nodes[0].agent_types.is_empty());
        assert!(graph.nodes[0].condition_pattern.is_none());
    }

    #[test]
    fn test_build_agent_specific_node() {
        let sections = vec![(
            "references/wave-agent-procedures.md".to_string(),
            "Wave Agent Procedures".to_string(),
            SectionIntent {
                is_command_specific: false,
                command: None,
                is_agent_specific: true,
                agent_type: Some("wave-agent".to_string()),
                is_conditional: false,
                condition_pattern: None,
                trigger_timing: None,
                reasoning: "Section contains procedures for wave agents".to_string(),
            },
        )];

        let graph = build(&[], &[], &sections);

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(
            graph.nodes[0].reference_file,
            "references/wave-agent-procedures.md"
        );
        assert!(graph.nodes[0].trigger_patterns.is_empty());
        assert_eq!(graph.nodes[0].agent_types, vec!["wave-agent"]);
        assert!(graph.nodes[0].condition_pattern.is_none());
    }

    #[test]
    fn test_build_conditional_node() {
        let sections = vec![(
            "references/advanced-features.md".to_string(),
            "Advanced Features".to_string(),
            SectionIntent {
                is_command_specific: false,
                command: None,
                is_agent_specific: false,
                agent_type: None,
                is_conditional: true,
                condition_pattern: Some("--advanced".to_string()),
                trigger_timing: None,
                reasoning: "Section only relevant when advanced flag is used".to_string(),
            },
        )];

        let graph = build(&[], &[], &sections);

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(
            graph.nodes[0].reference_file,
            "references/advanced-features.md"
        );
        assert!(graph.nodes[0].trigger_patterns.is_empty());
        assert!(graph.nodes[0].agent_types.is_empty());
        assert_eq!(
            graph.nodes[0].condition_pattern,
            Some("--advanced".to_string())
        );
    }

    #[test]
    fn test_build_always_loaded_node() {
        let sections = vec![(
            "references/core-concepts.md".to_string(),
            "Core Concepts".to_string(),
            SectionIntent {
                is_command_specific: false,
                command: None,
                is_agent_specific: false,
                agent_type: None,
                is_conditional: false,
                condition_pattern: None,
                trigger_timing: None,
                reasoning: "Section contains general concepts for all uses".to_string(),
            },
        )];

        let graph = build(&[], &[], &sections);

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].reference_file, "references/core-concepts.md");
        assert!(graph.nodes[0].trigger_patterns.is_empty());
        assert!(graph.nodes[0].agent_types.is_empty());
        assert!(graph.nodes[0].condition_pattern.is_none());
    }
}
