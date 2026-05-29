# Root PRD

## Product Definition

Root is a deterministic machine environment manager for developers and AI coding agents.

It lets developers install tools safely, lock machine state, roll back changes, and give AI coding agents a trusted protocol for installing and verifying dependencies without mutating the user's machine unpredictably.

## One-Liner

Root is a safer Brew for developers and AI coding agents, powered by Nix with no Nix knowledge required.

## Product Thesis

Modern developer machines are mutable, fragile, drifting, and increasingly AI-hostile. AI coding agents frequently install packages, mutate PATHs, run global commands, or rely on machine state they cannot inspect reliably.

Root makes local development reproducible, reversible, inspectable, and agent-safe.

## Problem

Developers often install a single CLI tool and accidentally trigger unrelated dependency updates. Homebrew is beloved and useful, but its global mutable model can cause unexpected machine drift.

Common pain points:

- Installing one tool updates unrelated tools.
- Node, Ruby, Python, Postgres, and OpenSSL versions drift over time.
- Developers cannot easily recreate the exact state of a working laptop.
- Rollbacks are not first-class.
- AI coding agents can break local environments by installing tools directly.
- Teams still suffer from “works on my machine.”

## Target Users

### 1. AI-Native Developers

Developers using Codex, Claude Code, Cursor, OpenCode, VS Code agents, or terminal-based coding assistants.

Primary pain: AI agents mutate their machines unpredictably.

### 2. Full-Stack and Infra Developers

Developers with complex local setups involving Node, Python, Rust, Go, Postgres, Redis, Docker, cloud CLIs, FFmpeg, PDF tooling, and language servers.

Primary pain: dependency conflicts and machine drift.

### 3. Small Engineering Teams

Teams that want predictable onboarding and reproducible local tooling without forcing every engineer to learn Nix.

Primary pain: onboarding inconsistency and local environment support burden.

## MVP Goal

Root v0.1 should prove one thing:

> A developer can install CLI tools on macOS without mutating unrelated dependencies, and can roll back safely.

## MVP Scope

Root v0.1 supports:

- macOS Apple Silicon first
- CLI packages only
- Nix under the hood
- Rootfile and root.lock
- local snapshots
- rollback
- doctor/drift detection
- Brew import report
- JSON output for AI agents
- agent skill pack for Codex, Claude Code, Cursor, and generic agents

## Non-Goals for MVP

Root v0.1 does not support:

- GUI app replacement for Brew casks
- Linux or Windows
- cloud sync
- team dashboard
- private package registry
- enterprise policy controls
- full Nix language exposure
- replacing Docker/dev containers

## Core Commands

```bash
root init
root install <pkg>
root remove <pkg>
root list
root lock
root sync
root history
root rollback
root doctor
root plan install <pkg>
root verify <pkg>
root import brew
```

Every command that changes or inspects state should support:

```bash
--json
```

## Core User Promise

When a user runs:

```bash
root install poppler
```

Root should communicate:

```txt
Installed poppler.
Changed: poppler and required dependencies.
Unchanged: node, ruby, postgres, python.
Snapshot saved. Rollback available.
```

## Success Metrics

MVP success can be measured by:

- User can install a package through Root successfully.
- Rootfile and root.lock are generated.
- User can run `root sync` on a fresh machine/profile and reproduce the environment.
- User can rollback to a previous snapshot.
- `root doctor` can detect PATH shadowing and lockfile drift.
- Agent skill can instruct Codex/Claude/Cursor to use Root instead of Brew.
- JSON output is valid and machine-readable.

## Positioning

Root should be positioned as:

> The package manager AI agents are allowed to use.

Secondary positioning:

> Brew-simple installs. Nix-safe machines.

## MVP Demo Script

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
