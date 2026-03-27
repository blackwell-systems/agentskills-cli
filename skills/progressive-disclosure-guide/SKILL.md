---
name: progressive-disclosure-guide
description: Interactive guide for converting skills to progressive disclosure pattern. Use when you have a skill >500 lines and want to split it into core + references for better context economy.
context: fork
agent: general-purpose
allowed-tools: Bash, Read, Write
argument-hint: "<path-to-skill-directory>"
---

# Progressive Disclosure Guide

You are helping a user convert their skill to use progressive disclosure — a pattern that keeps the core SKILL.md under 200 lines while making detailed reference material available on-demand.

## What is Progressive Disclosure?

Progressive disclosure is a three-tier information architecture for skills:

**Tier 1: Metadata (always loaded)**
- `name` and `description` fields from frontmatter
- Used for skill discovery and dispatch decisions

**Tier 2: Core instructions (loaded on invocation)**
- Main SKILL.md content
- Target: <200 lines for fast loading
- Contains: overview, routing logic, common workflows

**Tier 3: Reference material (loaded on-demand)**
- Detailed docs in `references/` directory
- Loaded only when specific subcommands/patterns trigger
- Examples: API specs, extensive examples, edge case handling

## Your Task

Guide the user through converting their skill step-by-step:

### Step 1: Validate Prerequisites

Check if `agentskills` CLI is available:
```bash
command -v agentskills
```

If not found, tell the user:
> "The agentskills CLI tool is required. Install from: https://github.com/blackwell-systems/agentskills-cli"

Then stop and wait for them to install it.

### Step 2: Assess Current State

Get the skill path from `$ARGUMENTS`. If no path provided, ask the user for it.

Run lint to check current state:
```bash
agentskills lint $SKILL_PATH
```

**If the skill is already compliant** (under 200 lines):
- Congratulate them!
- Explain they could still benefit from references/ for future growth
- Ask if they want to proceed anyway

**If the skill has issues**:
- Summarize the validation results
- Explain what progressive disclosure will fix

### Step 3: Analyze Structure

Read the skill file:
```bash
Read $SKILL_PATH/SKILL.md
```

Analyze the content and identify:
1. **Core sections** (must stay in SKILL.md):
   - Overview/introduction
   - Routing logic (if present)
   - Common workflows
   - Cross-references to other sections

2. **Reference candidates** (can move to references/):
   - Detailed API documentation
   - Extensive examples
   - Edge case handling
   - Background/theory
   - Troubleshooting guides
   - Long tables or data

Present your analysis to the user with specific line ranges and reasoning.

### Step 4: Preview Changes

Run upgrade in dry-run mode:
```bash
agentskills upgrade --dry-run $SKILL_PATH
```

Explain what the tool will do:
- Which content stays in SKILL.md
- Which files will be created in references/
- How the routing works (if applicable)

**Important:** Explain the trade-offs:
- ✅ Faster skill loading (smaller core)
- ✅ Better context economy (load details only when needed)
- ⚠️ More files to maintain
- ⚠️ Need to set up routing/injection mechanism

### Step 5: Get Confirmation

Ask the user: "Shall I proceed with the upgrade?"

If they say no or want to customize:
- Offer to show them how to do manual splits
- Explain which sections they should prioritize moving
- Point them to the upgrade command documentation

If they say yes, proceed to Step 6.

### Step 6: Execute Upgrade

Run the actual upgrade:
```bash
agentskills upgrade $SKILL_PATH
```

Report what was created:
- New references/ directory
- New reference files
- Updated SKILL.md
- Any routing scripts (if applicable)

### Step 7: Validate Result

Run lint again to confirm:
```bash
agentskills lint $SKILL_PATH
```

The skill should now pass validation with <200 lines in SKILL.md.

### Step 8: Explain Next Steps

Tell the user what they need to do to complete the migration:

1. **Review the split**: Read the updated SKILL.md and references/ to ensure the split makes sense
2. **Update references**: Add cross-references in SKILL.md to guide when to load each reference file
3. **Set up routing** (optional): If using subcommands, implement the routing mechanism:
   - Add `triggers` to frontmatter (Claude Code extension)
   - Use `!`command`` injection to load references conditionally
   - Or use hooks (PreToolUse) for automatic injection
4. **Test**: Invoke the skill and verify references load correctly
5. **Commit**: Save the changes to version control

### Step 9: Offer Advanced Patterns

Ask if they want to learn about advanced progressive disclosure patterns:

**Pattern A: Subcommand routing**
```yaml
triggers:
  - match: "^/skill-name subcmd1"
    inject: references/subcmd1.md
  - match: "^/skill-name subcmd2"
    inject: references/subcmd2.md
```

**Pattern B: Dynamic injection via shell**
```
Use backtick injection: !`bash scripts/inject-context.sh "$ARGUMENTS"`
(Place this in SKILL.md to dynamically load references based on arguments)
```

**Pattern C: Agent-specific references**
Use `agent-references` frontmatter to inject context when spawning subagents.

Offer to help them implement any of these patterns.

## Important Guidelines

- **Be educational**: Explain WHY you're suggesting each split, not just what to move
- **Ask before acting**: Always get confirmation before running the actual upgrade
- **Preserve user intent**: If they want to keep something in core, respect that
- **Show the benefits**: Explain how progressive disclosure improves context economy
- **Offer alternatives**: Manual splitting, partial upgrades, or keeping it monolithic

## Example Flow

```
User: /progressive-disclosure-guide ~/.claude/skills/my-big-skill

You: Let me check if the agentskills CLI is available...
     ✓ Found agentskills v0.1.0

     Now analyzing your skill...

     📊 Current state:
     - SKILL.md: 847 lines
     - No references/ directory
     - ⚠️ Warning: SKILL.md exceeds 200-line recommendation

     I've read your skill content. Here's what I found:

     **Core sections (should stay in SKILL.md):**
     - Lines 1-50: Overview and invocation modes
     - Lines 200-250: Main workflow logic

     **Reference candidates (can move to references/):**
     - Lines 51-199: Detailed API documentation → references/api-docs.md
     - Lines 251-600: Extensive examples → references/examples.md
     - Lines 601-847: Troubleshooting guide → references/troubleshooting.md

     This would reduce SKILL.md from 847 → ~150 lines while preserving all content.

     Shall I run a dry-run preview to show exactly what would change?
```
