# Progressive Disclosure Example Index

This directory contains a comprehensive example for the `agentskills-cli` tool, demonstrating progressive disclosure patterns for Agent Skills.

## File Overview

### 1. **release-management-skill.md** (483 lines)
The main example file — a complete, realistic release management skill.

**Purpose:** Demonstrate a skill that benefits from progressive disclosure.

**Characteristics:**
- Large enough to warrant splitting (483 lines)
- Complex workflow with multiple stages
- Mix of invocation-time and runtime content
- Multiple trigger patterns (6 subcommands)
- Realistic use case (not a toy example)

**Start here:** This is the input file for all tool operations.

### 2. **README.md** (650+ lines)
Complete documentation of the example.

**Contents:**
- What the example demonstrates
- Feature exercise matrix
- Running instructions (lint, upgrade)
- Expected results and directory structure
- Performance benchmarks
- Routing style comparisons
- CI/CD integration examples
- Troubleshooting guide

**Start here if:** You want to understand the example comprehensively.

### 3. **USAGE-EXAMPLE.md** (550+ lines)
Step-by-step walkthrough with actual command outputs.

**Contents:**
- Real terminal commands and outputs
- Before/after comparisons
- Performance measurements
- CI/CD integration snippets
- Interactive mode flow
- Troubleshooting scenarios

**Start here if:** You want to see exactly what happens when you run the tool.

### 4. **INVESTIGATION-REPORT.md** (500+ lines)
Technical investigation findings and analysis.

**Contents:**
- Tool location and setup
- Complete capability summary
- Feature exercise matrix
- Architecture insights
- Performance analysis
- Recommendations
- Known limitations

**Start here if:** You want technical details about the tool's implementation.

### 5. **INDEX.md** (this file)
Navigation guide for the example directory.

## Quick Start

### Absolute Beginner
1. Read **README.md** (sections: "What This Example Demonstrates" and "Running the Example")
2. Run the lint command from **USAGE-EXAMPLE.md** Step 1
3. Run the dry-run upgrade from **USAGE-EXAMPLE.md** Step 2
4. Review the output and understand what progressive disclosure does

### Skill Author (want to apply to your skills)
1. Read **README.md** → "Running the Example" section
2. Run the example commands on release-management-skill.md
3. Read **USAGE-EXAMPLE.md** → "Performance Comparison" section
4. Apply the pattern to your own skills

### Tool Developer (want to understand implementation)
1. Read **INVESTIGATION-REPORT.md** → "Tool Capabilities" section
2. Review "Architecture Insights" section
3. Check "Feature Exercise Matrix" to see coverage
4. Review "Recommendations" for improvement ideas

### CI/CD Engineer (want to integrate)
1. Read **USAGE-EXAMPLE.md** → "CI/CD Integration Example" section
2. Review **README.md** → "Testing the Upgrade" section
3. Copy the GitHub Actions workflow example
4. Adapt to your pipeline

## Learning Path

### Path 1: Hands-On (30 minutes)
1. Build agentskills-cli: `cd /path/to/agentskills-cli && cargo build --release`
2. Run lint: `agentskills lint release-management-skill.md`
3. Run dry-run: `agentskills decompose release-management-skill.md --dry-run`
4. Review output and understand 43% size reduction
5. Read **README.md** to understand why it matters

### Path 2: Comprehensive Understanding (90 minutes)
1. Read **INVESTIGATION-REPORT.md** (20 min)
2. Read **README.md** (25 min)
3. Run all commands from **USAGE-EXAMPLE.md** (30 min)
4. Compare routing styles (10 min)
5. Apply to your own skill (5 min)

### Path 3: Deep Dive (3+ hours)
1. Read all documentation files
2. Run interactive upgrade with semantic analysis
3. Test all routing styles (smart, table, inline, none)
4. Test multiple providers (Anthropic, OpenAI, Gemini)
5. Measure context savings in real usage
6. Integrate into your CI/CD pipeline

## Command Cheat Sheet

**Prerequisites:**
```bash
# Build the tool
cd /path/to/agentskills-cli
cargo build --release

# Set path for convenience
export PATH="/path/to/agentskills-cli/target/release:$PATH"

# Navigate to example
cd /path/to/scout-and-wave/examples/progressive-disclosure
```

**Basic validation:**
```bash
# Validate skill
agentskills lint release-management-skill.md

# JSON output
agentskills lint release-management-skill.md --json | jq
```

**Upgrade (preview):**
```bash
# Dry run (no files written)
agentskills decompose release-management-skill.md --dry-run

# With semantic analysis (requires API key)
export ANTHROPIC_API_KEY="sk-ant-..."
agentskills decompose release-management-skill.md --dry-run
```

**Upgrade (apply):**
```bash
# Interactive mode (recommended first time)
agentskills decompose release-management-skill.md --interactive

# All features
agentskills decompose release-management-skill.md \
  --routing-style smart \
  --timing \
  --back-links

# Different routing styles
agentskills decompose release-management-skill.md --routing-style table
agentskills decompose release-management-skill.md --routing-style inline
agentskills decompose release-management-skill.md --routing-style none
```

**Verify results:**
```bash
# Check structure
tree release-management-skill/

# Check line counts
wc -l release-management-skill/SKILL.md
wc -l release-management-skill/references/*.md

# Check generated files
cat release-management-skill/scripts/inject-context
head -20 release-management-skill/references/failure-diagnostics.md
```

## Key Concepts

### Progressive Disclosure
**Problem:** Large skills (500+ lines) waste context by loading content that's never used.

**Solution:** Split into core (always loaded) + references (loaded on-demand).

**Result:** 36-43% context reduction in typical scenarios.

### Invocation vs Runtime
**Invocation-time:** Loaded when skill is first called (subcommand help, config reference).

**Runtime:** Loaded during execution when conditions occur (failure diagnostics, troubleshooting).

**Classification:** Done by semantic analysis (LLM) or heuristics (mechanical).

### Routing Styles
**Smart:** Context-aware breadcrumbs with explicit Read instructions (recommended).

**Table:** Simple routing table at top of SKILL.md (visible index).

**Inline:** Direct Read commands inline (minimalist).

**None:** Extract references but no loading logic (manual wiring).

## Performance Summary

### Context Savings
- **Successful release:** 43% reduction (483 → 274 lines)
- **Build failure:** 36% reduction (483 → 310 lines)
- **Average:** 36-43% depending on failure rate

### Build Performance
- **Clean build:** ~5-6 seconds (release mode)
- **Lint command:** <100ms per skill
- **Upgrade (mechanical):** <200ms per 500-line skill
- **Upgrade (semantic):** 2-5 seconds per section (LLM latency)

## Feature Coverage

| Feature Category | Coverage |
|-----------------|----------|
| **Validation** | 100% (base spec + extensions) |
| **Semantic Analysis** | 100% (6 providers + mechanical fallback) |
| **Timing Classification** | 100% (invocation + runtime) |
| **Routing Generation** | 100% (4 styles) |
| **Output Formats** | 100% (terminal + JSON) |
| **Interactive Mode** | 100% (reasoning + preview) |
| **CI/CD Integration** | 100% (exit codes + JSON) |

## Common Questions

### "Why is this example so long?"
Because it's **realistic**. A toy example (100 lines) doesn't demonstrate the value of progressive disclosure. A real-world skill (483 lines) shows the actual problem and solution.

### "Do I need an API key?"
No. The tool works without any LLM provider (mechanical splitting). But semantic analysis (with a provider) gives better results.

### "Will this work with my skill?"
Probably yes. The tool is designed for the Agent Skills spec, which is platform-agnostic. It validates base spec compliance and warns about extensions.

### "What if I disagree with the semantic classification?"
Use `--interactive` mode and say "n" when prompted to skip extraction for that section.

### "Can I undo an upgrade?"
Yes, with `git checkout` if you committed before upgrading. Or just delete the generated `release-management-skill/` directory and start over.

### "How do I integrate into my workflow?"
See **USAGE-EXAMPLE.md** → "CI/CD Integration Example" for GitHub Actions and pre-commit hook examples.

## Related Documentation

### In scout-and-wave repo:
- `/docs/skills-progressive-disclosure.md` — Progressive disclosure protocol
- `/implementations/claude-code/prompts/saw-skill.md` — Real-world PD example

### External:
- [Agent Skills Specification](https://agentskills.io/specification)
- [agentskills-cli README](https://github.com/blackwell-systems/agentskills-cli)

## Support

For questions or issues:
1. Check **README.md** → "Troubleshooting" section
2. Review **USAGE-EXAMPLE.md** → "Troubleshooting" section
3. Read **INVESTIGATION-REPORT.md** → "Known Limitations" section
4. Open an issue in the agentskills-cli repo

## File Summary

| File | Lines | Purpose | Start Here If... |
|------|-------|---------|------------------|
| **release-management-skill.md** | 483 | Example input | You want to test the tool |
| **README.md** | 650+ | Complete docs | You want comprehensive understanding |
| **USAGE-EXAMPLE.md** | 550+ | Real outputs | You want to see actual results |
| **INVESTIGATION-REPORT.md** | 500+ | Technical analysis | You want implementation details |
| **INDEX.md** | 300+ | Navigation | You want to navigate the example |

**Total documentation:** ~2,500 lines covering every aspect of the tool and example.

---

**Last Updated:** 2026-03-27
**Example Version:** 1.0
**Tool Version:** agentskills v0.1.0
