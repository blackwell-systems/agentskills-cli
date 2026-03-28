# Progressive Disclosure Example: Release Management Skill

This directory contains a comprehensive example demonstrating the progressive disclosure capabilities of `agentskills-cli`. The example is a realistic, fully-featured release management skill that exercises most/all features of the tool.

## What This Example Demonstrates

### 1. **Large, Realistic Skill (~650 lines)**
The `release-management-skill.md` file is a complete release orchestration workflow covering:
- Version control and semantic versioning
- Multi-environment deployments (staging, production)
- Multiple deployment strategies (rolling, blue-green, canary)
- Comprehensive monitoring and alerting
- Failure diagnostics and rollback procedures
- Security and compliance considerations

This is **not a toy example** — it represents a real-world skill that would benefit significantly from progressive disclosure.

### 2. **Multiple Trigger Patterns (Invocation-time)**
The skill has several subcommands that trigger different reference material:
- `/release-management prepare` — Version bumping procedures
- `/release-management build` — Build artifact generation
- `/release-management test` — Test suite execution
- `/release-management deploy` — Deployment orchestration
- `/release-management monitor` — Post-deployment monitoring
- `/release-management rollback` — Emergency rollback procedures

Each subcommand could have its detailed procedures extracted to separate reference files.

### 3. **Runtime-Triggered References**
The skill includes conditional sections loaded only when specific failures occur:
- **Build failures** → `references/build-failure-diagnosis.md`
- **Test failures** → `references/test-failure-diagnosis.md`
- **Deployment failures** → `references/deployment-failure-diagnosis.md`
- **Post-deployment issues** → `references/post-deployment-diagnosis.md`

These are loaded **during execution** when the failure condition is detected, not at invocation time.

### 4. **Complex Structure for Timing Analysis**
The skill mixes:
- **Core workflow** (always needed) — Lines 1-300
- **Invocation-specific content** (subcommand help) — Various sections
- **Runtime diagnostic content** (failure handling) — Lines 300-650

This tests the semantic analyzer's ability to classify when each section should be loaded.

### 5. **Rich Metadata for Pattern Detection**
The frontmatter includes:
- `argument-hint` with multiple subcommands and flags
- Explicit environment variables (staging/production)
- Multiple operation modes

The tool should detect these patterns and generate appropriate triggers.

## Running the Example

### Prerequisites

1. **Build agentskills-cli:**
```bash
cd /path/to/agentskills-cli
cargo build --release
```

2. **Set up semantic analysis provider (optional but recommended):**

The tool supports multiple providers for intelligent section classification:

```bash
# Option 1: Anthropic API
export ANTHROPIC_API_KEY="sk-ant-..."

# Option 2: Claude CLI (Max plan users)
# Just ensure 'claude' is on your PATH

# Option 3: OpenAI API
export OPENAI_API_KEY="sk-..."

# Option 4: Gemini API
export GOOGLE_API_KEY="..."

# Without a provider, the tool uses mechanical splitting (still useful but less intelligent)
```

### Running the Lint Command

**Validate the skill against Agent Skills spec:**

```bash
agentskills lint release-management-skill.md
```

**Expected output:**
```
⚠ Warning: SKILL.md exceeds 200-line recommendation (650 lines)
⚠ Warning: Consider running 'agentskills decompose' to apply progressive disclosure
✓ Base spec validation passed
```

**JSON output (useful for CI/CD):**
```bash
agentskills lint release-management-skill.md --json | jq
```

### Running the Decompose Command

**Step 1: Dry-run (preview changes)**

```bash
agentskills decompose release-management-skill.md --dry-run
```

This shows:
- Which sections will be extracted to references
- What the core SKILL.md will look like after extraction
- Which trigger patterns were detected
- Estimated line reduction

**Step 2: Interactive mode (with semantic analysis)**

```bash
agentskills decompose release-management-skill.md --interactive
```

This mode:
1. Analyzes each section with the LLM
2. Classifies timing (invocation vs runtime)
3. Shows reasoning for each extraction decision
4. Asks for confirmation before applying changes

**Step 3: Full upgrade with all features**

```bash
agentskills decompose release-management-skill.md \
  --interactive \
  --routing-style smart \
  --timing \
  --back-links
```

**Flags explained:**
- `--interactive` — Show semantic analysis reasoning and preview before applying
- `--routing-style smart` — Generate intelligent routing (vs table/inline/none)
- `--timing` — Include timing annotations (invocation vs runtime) in breadcrumbs
- `--back-links` — Generate back-links in reference files to core SKILL.md
- `--provider anthropic-api` — Force specific provider (optional)

### Expected Results After Upgrade

**Directory structure:**
```
release-management-skill/
├── SKILL.md (150-200 lines - core workflow)
└── references/
    ├── build-failure-diagnosis.md (invocation-time)
    ├── test-failure-diagnosis.md (runtime)
    ├── deployment-failure-diagnosis.md (runtime)
    ├── post-deployment-diagnosis.md (runtime)
    ├── security-procedures.md (invocation-time)
    ├── monitoring-setup.md (invocation-time)
    └── ci-cd-configuration.md (invocation-time)
```

**Core SKILL.md (after extraction):**
- Overview and workflow description (always loaded)
- Routing logic for subcommands
- Breadcrumbs to reference files with timing annotations

**Example breadcrumb (runtime-triggered):**
```markdown
## Build Failures [See references/build-failure-diagnosis.md when build fails]

If the build fails during artifact compilation, read `${SKILL_DIR}/references/build-failure-diagnosis.md`
and follow the diagnostic procedures.
```

**Example breadcrumb (invocation-triggered):**
```markdown
## Security Procedures

For detailed security incident response procedures, see `${SKILL_DIR}/references/security-procedures.md`.
```

**Updated frontmatter (with triggers):**
```yaml
---
name: release-management
description: Comprehensive release management workflow...
argument-hint: "[prepare|build|test|deploy|monitor|rollback]..."
triggers:
  - pattern: "prepare"
    inject: references/version-management.md
  - pattern: "build"
    inject: references/build-procedures.md
  - pattern: "deploy"
    inject: references/deployment-strategies.md
allowed-tools: Bash, Read, Write, Grep, Glob
---
```

## Features Exercised

### Progressive Disclosure Features

| Feature | Exercised? | How |
|---------|-----------|-----|
| **Large skill splitting** | ✅ | 650 lines → ~170 core + 7-8 reference files |
| **Semantic section analysis** | ✅ | LLM classifies invocation vs runtime sections |
| **Invocation-time triggers** | ✅ | Subcommand-specific references (prepare, build, deploy) |
| **Runtime-time triggers** | ✅ | Failure diagnostic references (build-failure, test-failure) |
| **Trigger pattern detection** | ✅ | Detects subcommands from argument-hint |
| **Smart routing generation** | ✅ | Generates breadcrumbs with Read instructions |
| **Timing annotations** | ✅ | Labels sections as invocation/runtime |
| **Back-link generation** | ✅ | Reference files link back to core for shared logic |
| **Frontmatter preservation** | ✅ | Original metadata maintained |
| **Multi-provider support** | ✅ | Works with Anthropic, OpenAI, Gemini, Copilot |

### Validation Features

| Feature | Exercised? | How |
|---------|-----------|-----|
| **Base spec validation** | ✅ | Checks required fields (name, description) |
| **200-line core check** | ✅ | Warns about 650-line original file |
| **Frontmatter validation** | ✅ | Validates YAML structure |
| **Extension detection** | ✅ | Detects Claude Code specific fields |
| **JSON output** | ✅ | `--json` flag for CI/CD integration |

### Decompose Options

| Option | What It Does | Example |
|--------|-------------|---------|
| `--dry-run` | Preview changes without writing files | See what would be extracted |
| `--interactive` | Show reasoning, ask for confirmation | Review LLM analysis before applying |
| `--routing-style smart` | Generate intelligent routing logic | Context-aware breadcrumbs |
| `--routing-style table` | Generate routing table | Simple lookup table |
| `--routing-style inline` | Inline Read instructions | Direct Read commands |
| `--routing-style none` | No routing (manual wiring) | You handle loading |
| `--timing` | Add timing annotations to breadcrumbs | Show invocation vs runtime |
| `--back-links` | Generate back-links in references | Link back to core SKILL.md |
| `--provider X` | Force specific semantic analysis provider | anthropic-api, claude-cli, etc. |

## Testing the Upgrade

### 1. Verify Line Count Reduction

**Before:**
```bash
wc -l release-management-skill.md
# Expected: ~650 lines
```

**After upgrade:**
```bash
wc -l release-management-skill/SKILL.md
# Expected: 150-200 lines (70-75% reduction)

wc -l release-management-skill/references/*.md
# Expected: ~450-500 lines total across reference files
```

### 2. Verify Semantic Classification

Check that the tool correctly identified:

**Invocation-time sections** (removed from core, triggered by subcommand):
- Version management procedures
- Build artifact generation
- Deployment strategies
- Monitoring setup
- Security procedures

**Runtime sections** (breadcrumb left in core, triggered by failures):
- Build failure diagnostics
- Test failure diagnostics
- Deployment failure diagnostics
- Post-deployment issue diagnosis

### 3. Verify Routing Generation

**Check core SKILL.md contains:**
- Breadcrumbs with explicit Read instructions
- Timing annotations (if `--timing` flag used)
- Routing logic at top (if `--routing-style smart` used)

**Check reference files contain:**
- Dedup markers: `<!-- injected: references/filename.md -->`
- Back-links to core SKILL.md (if `--back-links` flag used)
- Complete extracted content (no truncation)

### 4. Verify Trigger Generation

**Check frontmatter for triggers:**
```bash
head -n 20 release-management-skill/SKILL.md | grep -A 10 "triggers:"
```

Should see patterns matching subcommands:
```yaml
triggers:
  - pattern: "prepare"
    inject: references/version-management.md
  - pattern: "build"
    inject: references/build-procedures.md
  # ... etc
```

## Comparing Routing Styles

Try different routing styles to see how they affect the output:

### Smart Routing (Recommended)
```bash
agentskills decompose release-management-skill.md --routing-style smart --dry-run
```

**Output:** Context-aware breadcrumbs with explicit Read instructions:
```markdown
## Build Failures [runtime-triggered]

If the build fails during artifact compilation, read:
`${SKILL_DIR}/references/build-failure-diagnosis.md`
```

### Table Routing
```bash
agentskills decompose release-management-skill.md --routing-style table --dry-run
```

**Output:** Simple routing table at top of SKILL.md:
```markdown
## Reference Routing

| Trigger | Reference File |
|---------|----------------|
| Build fails | references/build-failure-diagnosis.md |
| Test fails | references/test-failure-diagnosis.md |
```

### Inline Routing
```bash
agentskills decompose release-management-skill.md --routing-style inline --dry-run
```

**Output:** Direct Read commands inline:
```markdown
## Build Failures

Read `${SKILL_DIR}/references/build-failure-diagnosis.md` for diagnostic procedures.
```

### No Routing
```bash
agentskills decompose release-management-skill.md --routing-style none --dry-run
```

**Output:** References extracted but no automatic loading logic (you wire it up manually).

## Performance Benchmarks

**Without progressive disclosure:**
- Context size: ~650 lines (always loaded)
- Wasted context: ~450 lines on successful releases (no failures = no diagnostics needed)
- Context efficiency: ~30% (only 200 lines actually used)

**With progressive disclosure:**
- Core context: ~170 lines (always loaded)
- On-demand references: 0-450 lines (loaded only when needed)
- Context efficiency: 85-100% (core + only relevant references)

**Example scenario: Successful production release**
- Without PD: 650 lines loaded (100% waste on diagnostics not needed)
- With PD: 170 lines loaded (0 references needed for success path)
- **Savings: 74% context reduction**

**Example scenario: Build failure during staging**
- Without PD: 650 lines loaded
- With PD: 170 (core) + 80 (build-failure-diagnosis.md) = 250 lines
- **Savings: 62% context reduction**

## Real-World Usage Pattern

This example mirrors a real production skill structure. Here's how it would be used:

### Initial Development
1. Write comprehensive skill with all details (~650 lines)
2. Get it working end-to-end
3. Test all failure modes and edge cases

### Optimization with Progressive Disclosure
1. Run `agentskills lint` to check current state
2. Run `agentskills decompose --interactive` to analyze sections
3. Review semantic classification (invocation vs runtime)
4. Apply upgrade and test that references load correctly
5. Deploy optimized skill (70% context reduction)

### Maintenance
1. Update reference files when failure procedures change
2. Keep core SKILL.md stable (rarely changes)
3. Add new reference files for new failure modes
4. Re-run `agentskills lint` periodically to catch bloat

## Advanced: Custom Provider Testing

Test the multi-provider fallback system:

```bash
# Force Anthropic API (best quality)
agentskills decompose release-management-skill.md --provider anthropic-api --dry-run

# Force Claude CLI (Max plan users)
agentskills decompose release-management-skill.md --provider claude-cli --dry-run

# Force OpenAI API
agentskills decompose release-management-skill.md --provider openai-api --dry-run

# Force mechanical splitting (no LLM)
# Unset all API keys and CLI tools first
agentskills decompose release-management-skill.md --dry-run
```

Compare the semantic classification quality across providers:
- Anthropic typically provides best classification (specialized for agent tasks)
- OpenAI/Gemini provide good general-purpose classification
- Mechanical splitting uses heuristics (section headers >50 lines)

## Troubleshooting

### "SKILL.md already under 200 lines"
If you've already run upgrade, restore the original:
```bash
git checkout release-management-skill.md
```

### "No semantic analyzer found"
Install a provider:
```bash
# Option 1: Set API key
export ANTHROPIC_API_KEY="sk-ant-..."

# Option 2: Install CLI tool
# For Claude CLI, ensure you're on Max plan and 'claude' is on PATH
```

### "Upgrade creates too many/few reference files"
Adjust splitting with flags:
```bash
# More aggressive splitting
agentskills decompose release-management-skill.md --routing-style smart

# Less aggressive (table only)
agentskills decompose release-management-skill.md --routing-style table

# Manual control
agentskills decompose release-management-skill.md --routing-style none --dry-run
# Then manually create reference files as needed
```

## Next Steps

After running this example:

1. **Apply to your own skills:**
   - Run `agentskills lint ~/.claude/skills/my-skill`
   - Run `agentskills decompose ~/.claude/skills/my-skill --interactive`

2. **Integrate into CI/CD:**
   ```bash
   # In your pre-commit hook or CI pipeline
   agentskills lint skills/production-skill --json | jq -e '.errors | length == 0'
   ```

3. **Establish baseline:**
   - Measure context usage before/after
   - Track which references are actually loaded in production
   - Optimize based on usage patterns

4. **Share your skills:**
   - Progressive disclosure makes skills more portable
   - Smaller core files are easier to review and maintain
   - Reference files can be community-contributed

## Further Reading

- [Agent Skills Specification](https://agentskills.io/specification)
- [Progressive Disclosure Patterns](../../../docs/skills-progressive-disclosure.md)
- [agentskills-cli README](https://github.com/blackwell-systems/agentskills-cli)

## Questions or Issues?

This example was created to stress-test `agentskills-cli` progressive disclosure features. If you find:
- Semantic classification errors (wrong invocation/runtime classification)
- Routing generation issues
- Performance problems with large skills
- Multi-provider compatibility issues

Please open an issue with:
1. The command you ran
2. Expected vs actual behavior
3. Provider used (if semantic analysis)
4. Output of `agentskills lint --json`
