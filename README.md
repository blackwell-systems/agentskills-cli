# agentskills-cli

CLI for validating and upgrading [Agent Skills](https://agentskills.io/specification) with control-flow awareness.

Converts large, monolithic skills into structured, portable programs by:
- Splitting content into progressive disclosure
- Classifying when sections are needed (invocation vs runtime)
- Converting implicit control flow into explicit, executable instructions

The official validator checks strict base-spec conformance. `agentskills-cli` is designed for real-world skills: it validates the base spec, detects vendor extensions without breaking on them, and helps migrate large or vendor-specific skills toward portable progressive disclosure.

## Core capabilities

Agent Skills defines a portable format for reusable AI agent capabilities.

This CLI is the enforcement and migration layer for that ecosystem. It validates skills against the spec and evolves existing skills toward that standard:

- Validate skills against the official spec
- Upgrade large skills into progressive disclosure (intelligent splitting with semantic routing)
- Detect vendor extensions and non-standard fields

## Why a CLI instead of a skill?

A skill can apply progressive disclosure at runtime. This CLI compiles it ahead of time.

Progressive disclosure is a structural concern: it determines what enters the model's context and when. Doing that work at runtime requires loading and reasoning about large skills inside the model itself.

This CLI moves that work to build time. It analyzes, restructures, and generates routing ahead of execution, producing deterministic, reusable artifacts that never require the model to process the full skill again. This avoids requiring the model to reason over the full skill at runtime.

In practice: skills run at runtime. This tool shapes them before they ever run.

## When to use this

- You have an Agent Skill and want to validate it against the spec
- You have a large skill and want to split it into progressive disclosure
- You want to detect vendor extensions and understand portability
- You are building skills for distribution across multiple agent platforms

## What this enables

**Runtime logic is preserved, but externalized into explicit instructions the model can execute.**

Before upgrade (283 lines):
```markdown
## Step 8 — Diagnose failures
If CI fails:
1. Fetch failed logs
2. Identify root cause
3. Propose fix
[30 lines of retry logic]
```

After upgrade (core: ~160 lines):
```markdown
## Step 8 — Diagnose failures [See references/diagnose-failures.md when CI fails]

Read `${SKILL_DIR}/references/diagnose-failures.md` and follow its instructions.
```

The sequential flow (Step 7 → 8 → 9) remains visible in the core skill. The runtime branch becomes an explicit Read instruction executed at the right moment.

## Features

### Lint - Spec Validation

Validates Agent Skills against the [base specification](https://agentskills.io/specification) and detects vendor-specific extensions.

**What it checks:**
- Required fields (`name`, `description`)
- Frontmatter structure and YAML validity
- Progressive disclosure rules (200-line core, references/ on-demand)
- Vendor extensions (`triggers`, `agent-references`, `model`, `version`)
- inject-agent-context script presence and format

Warns about non-standard fields without blocking on them. This is ecosystem-aware validation - vendor extensions serve real needs (conditional loading, model selection) and often become spec features. The tool helps you understand your conformance level and portability tradeoffs, not just pass/fail.

```bash
# Validate a skill
agentskills lint ~/.claude/skills/my-skill

# JSON output for CI/CD
agentskills lint ~/.claude/skills/my-skill --json
```

**Example output:**
```
✓ Base spec validation passed
⚠ Warning: Field 'triggers' is a vendor extension (not in Agent Skills spec)
⚠ Warning: SKILL.md exceeds 200-line recommendation (347 lines)
```

### Upgrade - Progressive Disclosure

Transforms large skills into progressive disclosure with semantic analysis and control-flow awareness.

**What it does:**
1. **Semantic analysis** - Classifies section routing intent AND timing (invocation vs runtime)
2. **Smart splitting** - Moves detailed content to `references/` loaded on-demand
3. **Control flow externalization** - Converts runtime branches into explicit Read instructions
4. **Frontmatter preservation** - Keeps existing `name`, `description`, and custom fields unchanged

**Three-tier architecture:**
- **Tier 1:** Metadata (always loaded) - `name`, `description` for skill discovery
- **Tier 2:** Core instructions (loaded on invocation) - <200 lines, overview + routing
- **Tier 3:** Reference material (loaded on-demand) - detailed docs, examples, edge cases

```bash
# Preview changes (recommended first run)
agentskills upgrade ~/.claude/skills/my-skill --dry-run

# Interactive mode - confirm before applying
agentskills upgrade ~/.claude/skills/my-skill --interactive

# With semantic analysis (requires ANTHROPIC_API_KEY or claude CLI)
agentskills upgrade ~/.claude/skills/my-skill
```

**Before upgrade:**
```
my-skill/
└── SKILL.md (847 lines - everything in one file)
```

**After upgrade:**
```
my-skill/
├── SKILL.md (150 lines - core with breadcrumbs)
└── references/
    ├── api-docs.md (invocation-time content)
    ├── examples.md (command-specific content)
    └── troubleshooting.md (runtime content with breadcrumb)
```

**Frontmatter unchanged:**
```yaml
---
name: my-skill
description: Original description preserved
# All existing fields remain exactly as they were
---
```

### Control Flow Awareness (Invocation vs Runtime)

Semantic analysis classifies WHEN each section is needed:

**Invocation-time sections** (triggered by user request):
- Subcommands: `--examples`, `/scout`, `--dry-run`
- Agent types: `wave-agent`, `scout`
- Explicit topics: "when user asks about X"

**Runtime sections** (triggered during execution):
- Failures: "if CI fails", "when test fails"
- Errors: "when error occurs", "on retry"
- Missing resources: "if artifact not found"
- State transitions: "after command completes"

**How they're handled:**

| Timing | Core SKILL.md | References | Loading |
|--------|---------------|------------|---------|
| **Invocation** | Removed entirely | Extracted | Platform-specific (you configure) |
| **Runtime** | Breadcrumb left | Extracted | Explicit Read instruction |

**Example - Release skill with CI failure handling:**

Before upgrade (283 lines):
```markdown
## Step 7 — Watch CI
[monitoring code]

## Step 8 — Diagnose failures
If CI fails:
1. Fetch failed logs
2. Identify root cause
3. Propose fix
[30 lines of retry logic]

## Step 9 — Verify release
[verification code]
```

After upgrade (core: ~160 lines):
```markdown
## Step 7 — Watch CI
[monitoring code]

## Step 8 — Diagnose failures [See references/diagnose-failures.md when CI fails]

Read `${SKILL_DIR}/references/diagnose-failures.md` and follow its instructions.

## Step 9 — Verify release
[verification code]
```

**Why breadcrumbs matter:**
- Preserve sequential flow (Step 7 → 8 → 9 still visible)
- Show WHAT to do (read references/diagnose-failures.md)
- Show WHEN to do it (when CI fails)
- LLM executes Read tool at the right moment

This is **control flow externalization** - implicit runtime branches become explicit, tool-callable instructions.

### Semantic Analysis Authentication

The `upgrade` command's semantic analysis supports multiple AgentSkills-compliant providers (checked in order):

1. **Anthropic API** - Set `ANTHROPIC_API_KEY` environment variable
2. **Claude CLI** - Have `claude` command available on PATH (Max plan users)
3. **OpenAI API** - Set `OPENAI_API_KEY` environment variable
4. **Gemini API** - Set `GOOGLE_API_KEY` environment variable
5. **Gemini CLI** - Have `gemini` command available on PATH
6. **GitHub Copilot CLI** - Have `copilot` command available on PATH (Copilot subscription)

Without any provider, the tool falls back to mechanical splitting (section headers only).

**Override provider selection:**
```bash
# Force a specific provider instead of auto-detection
agentskills upgrade <skill-path> --provider openai-api
agentskills upgrade <skill-path> --provider copilot-cli

# Valid provider names: anthropic-api, claude-cli, openai-api, gemini-api, gemini-cli, copilot-cli
```

## Installation

### From source

```bash
git clone https://github.com/blackwell-systems/agentskills-cli.git
cd agentskills-cli
cargo install --path .
```

### Requirements

- Rust 1.75+
- (Optional) LLM provider for semantic analysis: Anthropic (API/CLI), OpenAI (API), Google Gemini (API/CLI), or GitHub Copilot (CLI)

## Bundled Skills

The `skills/` directory contains interactive guides:

- **progressive-disclosure-guide**: Step-by-step wizard for manual skill conversion

Install to `~/.claude/skills/` and invoke with `/progressive-disclosure-guide <path-to-skill>`.

See [skills/README.md](skills/README.md) for details.

## How Progressive Disclosure Works

### Pattern Detection

Extracts routing patterns from your skill's frontmatter:

```yaml
# Input (SKILL.md frontmatter)
argument-hint: "[scout|wave|status] <feature>"
allowed-tools: Agent(subagent_type=scout), Agent(subagent_type=wave-agent)
```

Detected patterns:
- Subcommands: `scout`, `wave`, `status`
- Agent types: `scout`, `wave-agent`

### How Semantic Analysis Works

Classifies each section's routing intent AND timing:

**Example 1: Agent-specific section**
```markdown
## Scout Pre-flight Validation

Before launching a Scout agent, verify...
```

Analyzed as:
- `is_agent_specific: true` (mentions "Scout agent")
- `agent_type: "scout"`
- `trigger_timing: "invocation"` (loaded when scout agent launches)

Result:
- Section removed from core entirely
- Extracted to `references/scout-validation.md`
- No frontmatter generated (you wire up loading based on your platform)

**Example 2: Runtime-triggered section**
```markdown
## CI Failure Diagnosis

If CI fails during release, follow these steps...
```

Analyzed as:
- `is_conditional: true`
- `condition_pattern: "CI fails"`
- `trigger_timing: "runtime"` (loaded when failure occurs during execution)

Generates breadcrumb in core:
```markdown
## CI Failure Diagnosis — [See references/ci-failure-diagnosis.md when CI fails]

Read `${SKILL_DIR}/references/ci-failure-diagnosis.md` and follow its instructions.
```

### Mechanical Fallback (no API key)

Without semantic analysis, splits by section headers:
- Top-level sections stay in core
- Subsections >50 lines move to references/
- Cross-references added manually

## Validation Details

### Vendor Extensions

The tool recognizes common vendor-specific fields and validates their format:

| Field | Validation | Spec Status |
|-------|-----------|-------------|
| `triggers` | Array of strings, non-empty | Vendor extension (Claude Code) |
| `agent-references` | Array of objects with `file`, optional `when` | Vendor extension |
| `model` | Non-empty string | Vendor extension (Claude Code) |
| `version` | Semver format | Vendor extension (suggest `metadata.version`) |

**Why warn instead of error?** These extensions serve real needs (conditional loading, model selection) not yet in the base spec. The tool helps you make informed decisions about portability vs. functionality.

### Progressive Disclosure Rules

Based on empirical context economy research:

- **200-line core limit** - Keeps skill loading fast, forces focus on essential content
- **References loaded on-demand** - Detailed docs only when needed (subcommand, agent type, error pattern)
- **inject-agent-context script** - Required for conditional loading; must have shebang and be executable

## Development

```bash
# Build
cargo build

# Test
cargo test

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt
```

### Architecture

```
src/
├── commands/       # CLI command handlers (lint, upgrade)
├── models.rs       # Skill metadata, routing graph, config
├── validation/     # Spec validators (base, extensions, progressive disclosure)
└── upgrade/        # Progressive disclosure modules
    ├── analyzer.rs          # Bloat detection
    ├── pattern_detector.rs  # Frontmatter extraction
    ├── semantic_analyzer.rs # Claude API integration
    ├── routing_graph.rs     # Trigger pattern generation
    ├── frontmatter_gen.rs   # YAML output
    ├── splitter.rs          # Content splitting logic
    └── generator.rs         # Final assembly
```

## Examples

### Validate before upgrading

```bash
agentskills lint ~/.claude/skills/my-skill --json | jq '.warnings'
```

### Upgrade with preview

```bash
# See what would change
agentskills upgrade ~/.claude/skills/my-skill --dry-run

# Apply if it looks good
agentskills upgrade ~/.claude/skills/my-skill --interactive
```

### CI/CD integration

```bash
# Fail build if skill is invalid
agentskills lint skills/production-skill || exit 1

# Check for vendor extensions in pre-commit
agentskills lint skills/new-skill --json | \
  jq -e '.warnings | length == 0' || \
  echo "Warning: skill uses vendor extensions"
```

## Related Projects

- [Agent Skills Specification](https://agentskills.io/specification) - Official spec

## License

MIT
