# agentskills-cli

CLI tool for validating and upgrading Agent Skills.

## Features

- **Lint**: Validate Agent Skills against base spec and extensions
- **Upgrade**: Convert skills to progressive disclosure pattern

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# Validate a skill
agentskills lint /path/to/skill

# Upgrade a skill (dry-run)
agentskills upgrade /path/to/skill --dry-run

# Upgrade with agent-references
agentskills upgrade /path/to/skill --with-agent-references
```

## Development

```bash
cargo build
cargo test
cargo clippy
```
