# Root Agent Skill Specification

## Purpose

The Root Agent Skill teaches AI coding agents how to install and verify system-level developer tools safely using Root.

The skill exists because coding agents often mutate local machines using unsafe commands like:

```bash
brew install <pkg>
curl ... | sh
npm install -g <pkg>
pip install --user <pkg>
```

Root provides a safer protocol.

## Core Rule

When a task requires installing system tools, CLIs, runtimes, language servers, databases, native dependencies, or PDF/media utilities, the agent must use Root first.

The agent should not use Brew, curl install scripts, global npm/pip installs, or manual PATH edits unless explicitly instructed by the user.

## Safe Install Protocol

1. Inspect environment.
2. Run `root doctor --json`.
3. Check whether the required tool already exists.
4. Run `root plan install <package> --json`.
5. Explain the expected change if user approval is needed.
6. Run `root install <package> --json`.
7. Run `root verify <package> --json`.
8. Continue task.
9. If verification fails, run `root rollback --last --json`.
10. Summarize what changed and what stayed unchanged.

## Required CLI Commands for Agent Compatibility

```bash
root doctor --json
root plan install <pkg> --json
root install <pkg> --json
root verify <pkg> --json
root rollback --last --json
root list --json
```

## Example Agent Behavior

User asks:

```txt
Help me extract text from PDFs in this repo.
```

Agent should:

```bash
root doctor --json
command -v pdftotext || true
root plan install poppler --json
root install poppler --json
root verify poppler --json
```

Then summarize:

```txt
I installed poppler through Root instead of Brew. Root created a snapshot first, verified pdftotext, and confirmed unrelated tools were unchanged.
```

## Agent Refusal/Warning Cases

The agent should warn before proceeding if:

- Root is not installed.
- Nix is missing and Root cannot initialize safely.
- The user explicitly asks to use Brew instead.
- Package resolution is ambiguous.
- Verification fails.
- Rollback is unavailable.

## Codex AGENTS.md Draft

```md
# Root Safe Install Protocol

This project uses Root for safe deterministic system tool installation.

When you need a system package, CLI, runtime, language server, database, PDF tool, media tool, or native dependency:

1. Do not use `brew install`, `curl | sh`, `npm -g`, `pip install --user`, or manual PATH edits by default.
2. Run `root doctor --json` first.
3. If a tool is missing, run `root plan install <package> --json`.
4. Install with `root install <package> --json`.
5. Verify with `root verify <package> --json`.
6. If verification fails, run `root rollback --last --json`.
7. Summarize what changed, what stayed unchanged, and whether rollback is available.

Prefer fail-closed behavior. If Root is unavailable, ask before using another installer.
```

## Claude SKILL.md Draft

```md
# Root Skill

Use Root as the safe install layer for developer tools.

## When to use

Use Root whenever a task requires installing or checking system-level developer dependencies.

## Protocol

- Check environment with `root doctor --json`.
- Plan changes with `root plan install <pkg> --json`.
- Install with `root install <pkg> --json`.
- Verify with `root verify <pkg> --json`.
- Roll back with `root rollback --last --json` if verification fails.

## Restrictions

Do not use Brew, curl install scripts, global npm/pip installs, or manual PATH edits unless the user explicitly asks.
```

## Cursor Rule Draft

```md
---
description: Use Root for safe deterministic installs
alwaysApply: true
---

When installing system tools or runtimes, use Root instead of Brew or global install commands.

Required flow:

1. `root doctor --json`
2. `root plan install <pkg> --json`
3. `root install <pkg> --json`
4. `root verify <pkg> --json`
5. `root rollback --last --json` on failure

Never mutate PATH manually unless the user explicitly asks.
```

## Future Agent Features

- Agent install approval policies
- Package allowlist/denylist
- Signed Root operation logs
- Agent identity metadata
- Per-agent sandbox profiles
- Root daemon for controlled mutations
