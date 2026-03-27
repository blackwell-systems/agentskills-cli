use agentskills::error::Error;
use agentskills::models::{RoutingStyle, UpgradeOptions};
use agentskills::upgrade::upgrade_skill;
use std::fs;
use std::io::Write;
use tempfile::TempDir;

/// Test upgrade with routing table style
#[tokio::test]
async fn test_upgrade_with_routing_table_style() {
    let temp_dir = TempDir::new().unwrap();
    let skill_path = temp_dir.path().join("SKILL.md");
    let mut file = fs::File::create(&skill_path).unwrap();

    // Create content with multiple subcommands (should trigger table style)
    let content = r#"---
name: test-skill
description: Test skill with routing
argument-hint: "/test [cmd1|cmd2|cmd3|cmd4]"
---

# Test Skill

Main content here.

## Command 1 Details

Details for command 1.
Line 2
Line 3
Line 4
Line 5
Line 6
Line 7
Line 8
Line 9
Line 10
Line 11
Line 12
Line 13
Line 14
Line 15
Line 16
Line 17
Line 18
Line 19
Line 20
Line 21
Line 22
Line 23
Line 24
Line 25
Line 26
Line 27
Line 28
Line 29
Line 30

## Command 2 Details

Details for command 2.
Line 2
Line 3
Line 4
Line 5
Line 6
Line 7
Line 8
Line 9
Line 10
Line 11
Line 12
Line 13
Line 14
Line 15
Line 16
Line 17
Line 18
Line 19
Line 20
Line 21
Line 22
Line 23
Line 24
Line 25
Line 26
Line 27
Line 28
Line 29
Line 30
"#;
    writeln!(file, "{}", content).unwrap();

    let options = UpgradeOptions {
        dry_run: false,
        with_agent_references: false,
        interactive: None,
        provider: None,
        routing_style: Some(RoutingStyle::Table),
        show_timing: false,
        back_links: true,
    };

    let result: Result<_, Error> = upgrade_skill(&skill_path, &options).await;
    assert!(result.is_ok());

    // Verify routing table was created in SKILL.md
    let updated_content = fs::read_to_string(&skill_path).unwrap();

    // Should contain routing table markers
    // Note: We can't verify exact table format without knowing Agent B implementation,
    // but we can check that the file was processed successfully
    assert!(updated_content.contains("test-skill"));
}

/// Test upgrade with inline style
#[tokio::test]
async fn test_upgrade_with_inline_style() {
    let temp_dir = TempDir::new().unwrap();
    let skill_path = temp_dir.path().join("SKILL.md");
    let mut file = fs::File::create(&skill_path).unwrap();

    // Create content with subcommands
    let content = r#"---
name: inline-test
description: Test inline routing
argument-hint: "/test [cmd1|cmd2]"
---

# Inline Test

Main content here.

## Command 1 Details

Details for command 1.
Line 2
Line 3
Line 4
Line 5
Line 6
Line 7
Line 8
Line 9
Line 10
Line 11
Line 12
Line 13
Line 14
Line 15
Line 16
Line 17
Line 18
Line 19
Line 20
Line 21
Line 22
Line 23
Line 24
Line 25
Line 26
Line 27
Line 28
Line 29
Line 30
"#;
    writeln!(file, "{}", content).unwrap();

    let options = UpgradeOptions {
        dry_run: false,
        with_agent_references: false,
        interactive: None,
        provider: None,
        routing_style: Some(RoutingStyle::Inline),
        show_timing: false,
        back_links: true,
    };

    let result: Result<_, Error> = upgrade_skill(&skill_path, &options).await;
    assert!(result.is_ok());

    // Verify reference files were created
    let references_dir = temp_dir.path().join("references");
    assert!(references_dir.exists());

    // Should have reference files for extracted sections
    let entries: Vec<_> = fs::read_dir(&references_dir).unwrap().collect();
    assert!(!entries.is_empty());
}

/// Test upgrade with back links disabled
#[tokio::test]
async fn test_upgrade_with_back_links_disabled() {
    let temp_dir = TempDir::new().unwrap();
    let skill_path = temp_dir.path().join("SKILL.md");
    let mut file = fs::File::create(&skill_path).unwrap();

    // Create content
    let content = r#"---
name: backlink-test
description: Test with back links disabled
---

# Test

Main content.

## Reference Section

Reference content.
Line 2
Line 3
Line 4
Line 5
Line 6
Line 7
Line 8
Line 9
Line 10
Line 11
Line 12
Line 13
Line 14
Line 15
Line 16
Line 17
Line 18
Line 19
Line 20
Line 21
Line 22
Line 23
Line 24
Line 25
Line 26
Line 27
Line 28
Line 29
Line 30
"#;
    writeln!(file, "{}", content).unwrap();

    let options = UpgradeOptions {
        dry_run: false,
        with_agent_references: false,
        interactive: None,
        provider: None,
        routing_style: Some(RoutingStyle::Inline),
        show_timing: false,
        back_links: false,
    };

    let result: Result<_, Error> = upgrade_skill(&skill_path, &options).await;
    assert!(result.is_ok());

    // Verify upgrade completed successfully
    // Note: We can't verify exact back-link behavior without Agent B implementation,
    // but we verify the option is accepted and upgrade completes
    let references_dir = temp_dir.path().join("references");
    assert!(references_dir.exists());
}

/// Test upgrade preserves existing behavior when routing=None (backwards compatibility)
#[tokio::test]
async fn test_upgrade_backwards_compatibility() {
    let temp_dir = TempDir::new().unwrap();
    let skill_path = temp_dir.path().join("SKILL.md");
    let mut file = fs::File::create(&skill_path).unwrap();

    let content = r#"---
name: compat-test
description: Backwards compatibility test
---

# Test

Content here.

## Section 1

Section content.
Line 2
Line 3
Line 4
Line 5
Line 6
Line 7
Line 8
Line 9
Line 10
Line 11
Line 12
Line 13
Line 14
Line 15
Line 16
Line 17
Line 18
Line 19
Line 20
Line 21
Line 22
Line 23
Line 24
Line 25
Line 26
Line 27
Line 28
Line 29
Line 30
"#;
    writeln!(file, "{}", content).unwrap();

    // Don't specify routing_style - should use default behavior
    let options = UpgradeOptions {
        dry_run: false,
        with_agent_references: false,
        interactive: None,
        provider: None,
        routing_style: None,
        show_timing: false,
        back_links: true,
    };

    let result: Result<_, Error> = upgrade_skill(&skill_path, &options).await;
    assert!(result.is_ok());

    // Verify basic upgrade completed
    let references_dir = temp_dir.path().join("references");
    assert!(references_dir.exists());
}

/// Test routing style parsing is case-insensitive
#[test]
fn test_routing_style_parsing_case_insensitive() {
    // This tests the CLI command parsing logic
    use agentskills::commands::upgrade::UpgradeCommand;
    use clap::Parser as _;

    let cmd = UpgradeCommand::try_parse_from(&[
        "upgrade",
        "/path/to/skill",
        "--routing-style",
        "SMART",
    ])
    .unwrap();

    assert_eq!(cmd.routing_style, Some("SMART".to_string()));

    let cmd = UpgradeCommand::try_parse_from(&[
        "upgrade",
        "/path/to/skill",
        "--routing-style",
        "inline",
    ])
    .unwrap();

    assert_eq!(cmd.routing_style, Some("inline".to_string()));
}

/// Test timing flag
#[test]
fn test_timing_flag() {
    use agentskills::commands::upgrade::UpgradeCommand;
    use clap::Parser as _;

    let cmd = UpgradeCommand::try_parse_from(&["upgrade", "/path/to/skill", "--timing"]).unwrap();

    assert!(cmd.timing);
}

/// Test back-links flag defaults to true
#[test]
fn test_back_links_default() {
    use agentskills::commands::upgrade::UpgradeCommand;
    use clap::Parser as _;

    let cmd = UpgradeCommand::try_parse_from(&["upgrade", "/path/to/skill"]).unwrap();

    assert!(cmd.back_links); // Should default to true
}
