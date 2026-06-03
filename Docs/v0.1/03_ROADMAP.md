# Root Roadmap — Believable v0.1 First

## Roadmap Principle

Root should not expand until the core trust loop works.

The core trust loop:

```text
Doctor
↓
Install
↓
Verify
↓
History
↓
Rollback
```

## v0.1 — Believable Ship

### Goal

Make one package install feel safer than Brew.

### Commands

```bash
root doctor
root install ffmpeg
root history
root rollback
```

### Demo

```bash
root doctor
root install ffmpeg
ffmpeg -version
root history
root rollback
root history
```

### Why This Matters

This proves the product thesis without building the full product.

If the v0.1 demo does not feel compelling, the later roadmap will not matter.

## v0.1.1 — Polish Release

### Goal

Make v0.1 less fragile after early feedback.

### Features

- Better error messages.
- Better Nix install detection.
- Better rollback messaging.
- Better README.
- Known limitations section.
- Support debug logs.

## v0.1.2 — More Golden Packages

### Goal

Validate Root beyond ffmpeg.

Candidate allowlist:

```text
ffmpeg
ripgrep
jq
poppler
imagemagick
nodejs
python3
```

Do not jump to arbitrary package installation yet.

## v0.2 — Root Lockfile

### Goal

Make managed machine state explicit.

Files:

```text
Rootfile
root.lock
```

Commands:

```bash
root lock
root sync
```

## v0.3 — Agent-Friendly Output

### Goal

Prepare Root for AI coding agents.

Features:

```bash
root doctor --json
root install ffmpeg --json
root history --json
root rollback --json
```

Add:

```text
skills/
├── AGENTS.md
├── claude/SKILL.md
├── codex/ROOT_CODEX_GUIDE.md
└── cursor/root.mdc
```

## v0.4 — Agent Runtime

### Goal

Make Root the safe machine mutation layer for coding agents.

Focus:

- Claude Code
- Codex
- Cursor
- OpenCode
- Amp

Agent rule:

> Use Root for system-level installs. Do not use Brew directly unless the user explicitly asks.

## v0.5 — Impact Analysis

### Goal

Show what will change before execution.

Command:

```bash
root plan install ffmpeg
```

Output:

```text
Will change:
  + ffmpeg

Will not change:
  node
  ruby
  python
  postgres

Rollback:
  available
```

## v0.6 — Permissions

### Goal

Require user approval before agent-initiated machine mutations.

Permission modes:

- approve once
- always allow
- deny

## v0.7 — Sandboxes

### Goal

Test installs before touching the real machine.

Potential providers:

- Docker
- macOS lightweight isolation where possible
- Cloudflare Sandbox support later
- Firecracker later

## v0.8 — Desktop

### Goal

Visualize the data Root already has.

Do not build Desktop before Root has meaningful history, snapshots, agent activity, and rollback data.

Desktop sections:

- Machine health
- Root history
- Snapshots
- Rollbacks
- Agent actions
- Pending approvals
