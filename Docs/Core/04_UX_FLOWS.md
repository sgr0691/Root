# Root UX Flows

## UX Principle

Root should feel like a safer Brew, not like a Nix tutorial.

Users should always understand:

- what changed
- what did not change
- how to undo it
- whether the machine matches the lockfile

## First-Run Flow

### Command

```bash
root init
```

### Ideal Output

```txt
Root initialized.

✓ Root directory created at ~/.root
✓ Nix detected
✓ Root profile ready
✓ Snapshot system enabled

Next:
  root install poppler
  root import brew
```

If Nix is missing:

```txt
Nix is required for Root.

Root can install and configure Nix safely.
No packages will be installed yet.

Run:
  root init --install-nix
```

## Install Flow

### Command

```bash
root install poppler
```

### Ideal Output

```txt
Planning install...

Will add:
  poppler 24.08.0

Required dependencies:
  cairo
  fontconfig
  freetype

Will not change:
  node
  ruby
  python
  postgres

Snapshot created:
  snap_20260524_153000

Installing...
Verifying...

✓ pdftotext available
✓ pdfinfo available

Installed poppler.
Rollback available with:
  root rollback snap_20260524_153000
```

## Plan Flow

### Command

```bash
root plan install poppler
```

### Purpose

Show what Root would do without doing it.

### Ideal Output

```txt
Install plan for poppler

Would add:
  poppler 24.08.0

Would not change:
  node
  ruby
  python
  postgres

Would create snapshot before install.
No changes made.
```

## Doctor Flow

### Command

```bash
root doctor
```

### Healthy Output

```txt
Root health check

✓ Nix available
✓ Root profile active
✓ Machine matches root.lock
✓ No PATH conflicts detected
✓ 8 packages verified

Your machine is in sync.
```

### Drift Output

```txt
Root health check

Drift detected:
  node expected 22.11.0, found 20.10.0

PATH conflict:
  /opt/homebrew/bin/node shadows Root-managed node

Suggested fix:
  root sync
```

## Rollback Flow

### Command

```bash
root rollback --last
```

### Ideal Output

```txt
Rolling back to:
  snap_20260524_153000

Will remove:
  poppler

Will restore:
  previous root.lock

Rolling back...
Verifying...

Rollback complete.
```

## Brew Import Flow

### Command

```bash
root import brew
```

### Ideal Output

```txt
Found 84 Brew packages.

Safe CLI matches:
  ripgrep → ripgrep
  fd → fd
  poppler → poppler
  ffmpeg → ffmpeg

Unsupported in MVP:
  18 casks

Unmatched:
  7 packages

Generated candidate file:
  ~/.root/Rootfile.import

No packages were changed.
```

## AI Agent Flow

### Scenario

User asks an AI agent:

> Help me extract text from PDFs in this repo.

### Agent Behavior

The agent should:

1. Check if Root exists.
2. Run `root doctor --json`.
3. Check whether `pdftotext` exists.
4. Run `root plan install poppler --json` if missing.
5. Ask or proceed based on agent permissions.
6. Run `root install poppler --json`.
7. Run `root verify poppler --json`.
8. Continue the task.
9. If verification fails, run `root rollback --last --json`.

### Agent Summary

```txt
I installed poppler through Root instead of Brew.
Root created a snapshot first, verified pdftotext, and confirmed Node, Ruby, Python, and Postgres were unchanged.
```

## Future Menu Bar Flow

The menu bar app should show:

- Root status: Healthy / Drift Detected / Needs Attention
- Last snapshot
- Packages changed recently
- Rollback action
- PATH conflicts
- Agent activity log

Example states:

```txt
Root: Healthy
Last snapshot: 12 minutes ago
Managed packages: 18
No drift detected
```

```txt
Root: Drift Detected
Homebrew node shadows Root node
Suggested action: root sync
```
