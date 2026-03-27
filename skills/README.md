# Bundled Skills

This directory contains skills that ship with agentskills-cli. These skills help you work with Agent Skills and progressive disclosure patterns.

## Available Skills

### progressive-disclosure-guide

Interactive guide for converting large skills to progressive disclosure format.

**Install:**
```bash
# Symlink to your personal skills directory
ln -s "$(pwd)/skills/progressive-disclosure-guide" ~/.claude/skills/progressive-disclosure-guide
```

**Usage:**
```bash
/progressive-disclosure-guide ~/.claude/skills/my-large-skill
```

The skill will:
1. Assess your current skill structure
2. Analyze which sections should stay in core vs move to references/
3. Show a dry-run preview of changes
4. Execute the upgrade with your confirmation
5. Validate the result
6. Explain next steps for routing setup

**When to use:**
- Your SKILL.md is >500 lines and takes too long to load
- You want better context economy (load details only when needed)
- You're converting an existing skill to use subcommands
- You're learning progressive disclosure patterns

**Requirements:**
- agentskills CLI tool must be installed and on PATH
- Claude Code (uses `context: fork` for subagent execution)

## Installation

To make these skills available in Claude Code:

```bash
# Install all bundled skills
for skill in skills/*/; do
  skill_name=$(basename "$skill")
  ln -sf "$(pwd)/$skill" ~/.claude/skills/"$skill_name"
done
```

Or install individually using the instructions in each skill's section above.

## Contributing

Have an idea for a bundled skill? Open an issue or PR at:
https://github.com/blackwell-systems/agentskills-cli
