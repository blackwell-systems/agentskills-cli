use crate::models::RoutingGraph;

/// Generates triggers YAML from routing graph
///
/// Iterates over graph nodes and emits YAML entries for each node with trigger_patterns.
/// Format follows progressive disclosure specification:
/// ```yaml
/// triggers:
///   - match: "<pattern>"
///     inject: references/<reference_file>
/// ```
///
/// Returns YAML string with proper indentation (2 spaces).
pub fn generate_triggers(graph: &RoutingGraph) -> String {
    let mut entries = Vec::new();

    for node in &graph.nodes {
        if node.trigger_patterns.is_empty() {
            continue;
        }

        for pattern in &node.trigger_patterns {
            let entry = format!(
                "  - match: \"{}\"\n    inject: references/{}",
                pattern, node.reference_file
            );
            entries.push(entry);
        }
    }

    if entries.is_empty() {
        return String::new();
    }

    format!("triggers:\n{}\n", entries.join("\n"))
}

/// Generates agent-references YAML from routing graph
///
/// Iterates over graph nodes and emits YAML entries for each node with agent_types.
/// Format follows progressive disclosure specification:
/// ```yaml
/// agent-references:
///   - agent-type: <type>
///     inject: references/<file>.md
///     when: "<condition_pattern>"  # optional
/// ```
///
/// Returns YAML string with proper indentation (2 spaces).
pub fn generate_agent_references(graph: &RoutingGraph) -> String {
    let mut entries = Vec::new();

    for node in &graph.nodes {
        if node.agent_types.is_empty() {
            continue;
        }

        for agent_type in &node.agent_types {
            let mut entry = format!(
                "  - agent-type: {}\n    inject: references/{}",
                agent_type, node.reference_file
            );

            if let Some(condition) = &node.condition_pattern {
                entry.push_str(&format!("\n    when: \"{}\"", condition));
            }

            entries.push(entry);
        }
    }

    if entries.is_empty() {
        return String::new();
    }

    format!("agent-references:\n{}\n", entries.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_triggers_single_node() {
        let graph = RoutingGraph {
            nodes: vec![RoutingNode {
                reference_file: "details.md".to_string(),
                trigger_patterns: vec!["/test".to_string()],
                agent_types: vec![],
                condition_pattern: None,
            }],
        };

        let result = generate_triggers(&graph);
        assert!(result.contains("triggers:"));
        assert!(result.contains("- match: \"/test\""));
        assert!(result.contains("inject: references/details.md"));
    }

    #[test]
    fn test_generate_triggers_multiple_nodes() {
        let graph = RoutingGraph {
            nodes: vec![
                RoutingNode {
                    reference_file: "details.md".to_string(),
                    trigger_patterns: vec!["/test".to_string()],
                    agent_types: vec![],
                    condition_pattern: None,
                },
                RoutingNode {
                    reference_file: "examples.md".to_string(),
                    trigger_patterns: vec!["/example".to_string(), "example:".to_string()],
                    agent_types: vec![],
                    condition_pattern: None,
                },
            ],
        };

        let result = generate_triggers(&graph);
        assert!(result.contains("triggers:"));
        assert!(result.contains("- match: \"/test\""));
        assert!(result.contains("inject: references/details.md"));
        assert!(result.contains("- match: \"/example\""));
        assert!(result.contains("- match: \"example:\""));
        assert!(result.contains("inject: references/examples.md"));
    }

    #[test]
    fn test_generate_triggers_empty_patterns() {
        let graph = RoutingGraph {
            nodes: vec![RoutingNode {
                reference_file: "details.md".to_string(),
                trigger_patterns: vec![],
                agent_types: vec![],
                condition_pattern: None,
            }],
        };

        let result = generate_triggers(&graph);
        assert_eq!(result, "");
    }

    #[test]
    fn test_generate_agent_references_single_node() {
        let graph = RoutingGraph {
            nodes: vec![RoutingNode {
                reference_file: "wave-details.md".to_string(),
                trigger_patterns: vec![],
                agent_types: vec!["wave-agent".to_string()],
                condition_pattern: None,
            }],
        };

        let result = generate_agent_references(&graph);
        assert!(result.contains("agent-references:"));
        assert!(result.contains("- agent-type: wave-agent"));
        assert!(result.contains("inject: references/wave-details.md"));
        assert!(!result.contains("when:"));
    }

    #[test]
    fn test_generate_agent_references_with_when() {
        let graph = RoutingGraph {
            nodes: vec![RoutingNode {
                reference_file: "program-flow.md".to_string(),
                trigger_patterns: vec![],
                agent_types: vec!["scout".to_string()],
                condition_pattern: Some("--program".to_string()),
            }],
        };

        let result = generate_agent_references(&graph);
        assert!(result.contains("agent-references:"));
        assert!(result.contains("- agent-type: scout"));
        assert!(result.contains("inject: references/program-flow.md"));
        assert!(result.contains("when: \"--program\""));
    }

    #[test]
    fn test_generate_agent_references_multiple_agents() {
        let graph = RoutingGraph {
            nodes: vec![RoutingNode {
                reference_file: "shared-details.md".to_string(),
                trigger_patterns: vec![],
                agent_types: vec!["scout".to_string(), "wave-agent".to_string()],
                condition_pattern: None,
            }],
        };

        let result = generate_agent_references(&graph);
        assert!(result.contains("agent-references:"));
        assert!(result.contains("- agent-type: scout"));
        assert!(result.contains("- agent-type: wave-agent"));
        // Both should point to the same reference file
        assert_eq!(result.matches("inject: references/shared-details.md").count(), 2);
    }

    #[test]
    fn test_generate_agent_references_empty_agent_types() {
        let graph = RoutingGraph {
            nodes: vec![RoutingNode {
                reference_file: "details.md".to_string(),
                trigger_patterns: vec![],
                agent_types: vec![],
                condition_pattern: None,
            }],
        };

        let result = generate_agent_references(&graph);
        assert_eq!(result, "");
    }

    #[test]
    fn test_generate_triggers_multiple_patterns_per_node() {
        let graph = RoutingGraph {
            nodes: vec![RoutingNode {
                reference_file: "amend-flow.md".to_string(),
                trigger_patterns: vec![
                    "^/saw amend".to_string(),
                    "--add-wave".to_string(),
                    "--redirect-agent".to_string(),
                ],
                agent_types: vec![],
                condition_pattern: None,
            }],
        };

        let result = generate_triggers(&graph);
        assert!(result.contains("triggers:"));
        assert!(result.contains("- match: \"^/saw amend\""));
        assert!(result.contains("- match: \"--add-wave\""));
        assert!(result.contains("- match: \"--redirect-agent\""));
        // All should point to the same reference file
        assert_eq!(result.matches("inject: references/amend-flow.md").count(), 3);
    }

    #[test]
    fn test_generate_agent_references_with_regex_condition() {
        let graph = RoutingGraph {
            nodes: vec![RoutingNode {
                reference_file: "program-contracts.md".to_string(),
                trigger_patterns: vec![],
                agent_types: vec!["wave-agent".to_string()],
                condition_pattern: Some("frozen_contracts_hash|frozen: true".to_string()),
            }],
        };

        let result = generate_agent_references(&graph);
        assert!(result.contains("agent-references:"));
        assert!(result.contains("- agent-type: wave-agent"));
        assert!(result.contains("inject: references/program-contracts.md"));
        assert!(result.contains("when: \"frozen_contracts_hash|frozen: true\""));
    }
}
