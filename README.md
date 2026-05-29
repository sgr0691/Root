# Root

> The package manager AI agents are allowed to use.

A safer Brew for developers and AI coding agents, powered by Nix with no Nix knowledge required.

[![CI](https://github.com/sgr0691/Root/actions/workflows/ci.yml/badge.svg)](https://github.com/sgr0691/Root/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/sgr0691/Root/main/scripts/install.sh | sh
```

Or install via Homebrew:

```bash
brew tap sgr0691/root
brew install root
```

## Quick Start

1. `root init`
2. `root plan install poppler`
3. `root install poppler`
4. `root verify poppler`
5. `root history`
6. `root rollback --last`

## Core Commands

| Command | Description |
|---|---|
| `root init` | Initialize Root directory structure |
| `root plan install <pkg>` | Search for a package to install |
| `root install <pkg>` | Install a package |
| `root remove <pkg>` | Remove a package |
| `root list` | List managed packages |
| `root history` | Show snapshot history |
| `root rollback --last` | Rollback to the previous state |
| `root doctor` | Check system health and drift |
| `root verify <pkg>` | Verify an installed package's binaries are executable |
| `root import <source>` | Import packages from other package managers (e.g., brew) |
| `root lock` | Regenerate root.lock from current state |
| `root sync` | Reconcile Nix profile with root.lock |

## JSON Output

Every command supports `--json` for machine-readable output, perfect for AI agent integration and scripting.

```bash
root install poppler --json
root doctor --json
```

## Agent Skills

Pre-built agent skill packs are available in the `skills/` directory for:

- **Codex** — `skills/codex/AGENTS.md`
- **Claude** — `skills/claude/SKILL.md`
- **Cursor** — `skills/cursor/root.mdc`
- **Generic** — `skills/generic/ROOT_AGENT_PROTOCOL.md`

These skills teach AI agents to use Root instead of Brew or global installers, following the safe install protocol.

## Safety

- **Snapshots before every mutation** — Root automatically saves state before installing or removing packages.
- **Rollback available** — Revert to a previous snapshot with a single command.
- **Deterministic via Nix** — Exact versions are locked and reproducible.
- **No global PATH pollution** — Root manages its own profile without overwriting your shell configuration.

## Demo

1. Start with a Mac that has Node, Ruby, Python, and Postgres installed.
2. Run `root init`.
3. Run `root install poppler`.
4. Show Root's impact report.
5. Confirm unrelated tools did not change.
6. Run `root history`.
7. Run `root rollback --last`.
8. Show poppler removed and previous state restored.
9. Run Codex/Claude/Cursor with Root skill instructions.
10. Agent detects a missing tool and uses Root safely.

## Development

Root v0.1 is the MVP release. See [Core Docs](Docs/Core/) for the full plan.

### v0.1 MVP

| Feature | Status |
|---|---|
| `root init` / `root install` / `root remove` / `root list` | ✅ |
| `root lock` / `root sync` | ✅ |
| `root history` / `root rollback --last` | ✅ |
| `root doctor` / `root verify` | ✅ |
| `root plan install` | ✅ |
| `root import brew` | ✅ |
| `--json` all commands | ✅ |
| Snapshots before every mutation | ✅ |
| Agent skill packs (Claude, Codex, Cursor) | ✅ |
| Nix adapter with mock tests | ✅ |

### Post-MVP

- `root shell` — project environments
- Menu bar app — drift status, snapshot timeline, rollback
- Agent permissions — install approvals, allow/deny lists, audit log
