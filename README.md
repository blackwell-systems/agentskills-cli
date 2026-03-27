# agentskills-cli

CLI for validating and upgrading [Agent Skills](https://agentskills.io/specification): enforce spec compliance, convert large skills into progressive disclosure, and detect vendor extensions.

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

This CLI moves that work to build time. It analyzes, restructures, and generates routing ahead of execution, producing deterministic, reusable artifacts that never require the model to process the full skill again.

In practice: skills run at runtime. This tool shapes them before they ever run.

## When to use this

- You have an Agent Skill and want to validate it against the spec
- You have a large skill and want to split it into progressive disclosure
- You want to detect vendor extensions and understand portability
- You are building skills for distribution across multiple agent platforms

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

Transforms large skills into progressive disclosure with semantic analysis and conditional routing.

**What it does:**
1. **Pattern detection** - Extracts subcommands and agent types from frontmatter
2. **Semantic analysis** - Uses Claude API to classify section routing intent
3. **Smart splitting** - Moves detailed content to `references/` loaded on-demand
4. **Routing generation** - Creates `triggers` and `agent-references` frontmatter
5. **Script bundling** - Includes production inject-agent-context script

**Three-tier architecture:**
- **Tier 1:** Metadata (always loaded) - `name`, `description` for skill discovery
- **Tier 2:** Core instructions (loaded on invocation) - <200 lines, overview + routing
- **Tier 3:** Reference material (loaded on-demand) - detailed docs, examples, edge cases

```bash
# Preview changes (recommended first run)
agentskills upgrade ~/.claude/skills/my-skill --dry-run

# Interactive mode - confirm before applying
agentskills upgrade ~/.claude/skills/my-skill --interactive

# Automatic upgrade with agent-references support
agentskills upgrade ~/.claude/skills/my-skill --with-agent-references

# With semantic analysis (requires ANTHROPIC_API_KEY or claude CLI)
agentskills upgrade ~/.claude/skills/my-skill --interactive
```

**Before upgrade:**
```
my-skill/
└── SKILL.md (847 lines - everything in one file)
```

**After upgrade:**
```
my-skill/
├── SKILL.md (150 lines - core overview + routing)
├── references/
│   ├── api-docs.md (loaded when user asks about API)
│   ├── examples.md (loaded for --examples flag)
│   └── troubleshooting.md (loaded on error patterns)
└── scripts/
    └── inject-agent-context (conditional loading logic)
```

**Routing example (generated):**
```yaml
triggers:
  - match: "^/skill-name --examples"
    inject: references/examples.md
  - match: "error|failed|broken"
    inject: references/troubleshooting.md

agent-references:
  - file: references/api-docs.md
    when:
      agent_type: "scout"
```

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

### Semantic Analysis (with ANTHROPIC_API_KEY or Claude CLI)

Classifies each section's routing intent:

```markdown
## Scout Pre-flight Validation

Before launching a Scout agent, verify...
```

Analyzed as:
- `is_agent_specific: true` (mentions "Scout agent")
- `agent_type: "scout"`
- `is_command_specific: false`

Generates:
```yaml
agent-references:
  - file: references/scout-validation.md
    when:
      agent_type: "scout"
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
