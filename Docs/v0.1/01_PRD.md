# Root v0.1 PRD

## Product Name

Root

## Release

v0.1 — Believable Ship

## Goal

Ship the smallest credible Root release that proves deterministic, reversible machine changes.

## Problem

Installing one tool on a developer machine often causes unrelated system changes.

With Homebrew, a simple install can update dependencies, runtimes, or libraries the user did not intend to touch.

Developers want the convenience of Brew and the reproducibility of Nix, but without learning Nix.

## Product Hypothesis

If Root can make one install feel safer, clearer, and reversible, developers will understand the product immediately.

## Scope

Root v0.1 supports four commands:

```bash
root install ffmpeg
root history
root rollback
root doctor
```

`ffmpeg` is the first supported golden package because:

- It is familiar to developers.
- It has real dependencies.
- It is easy to verify with `ffmpeg -version`.
- It makes a good demo.
- It is commonly installed through Brew.

## User Stories

### Install

As a developer, I want to install ffmpeg without learning Nix so that I can use a common CLI tool safely.

Acceptance criteria:

- `root install ffmpeg` installs ffmpeg through Nix.
- Root creates a snapshot before install.
- Root records an install event.
- Root verifies `ffmpeg` is available after install.
- Root prints a clear human-readable summary.

### History

As a developer, I want to see what Root has changed so that I can understand my machine state.

Acceptance criteria:

- `root history` shows Root-managed events.
- Events include timestamp, command, package, status, and snapshot id.
- History is readable without JSON parsing.
- Empty history has a helpful empty state.

### Rollback

As a developer, I want to undo the last Root-managed change so that I can recover from a bad install.

Acceptance criteria:

- `root rollback` rolls back to the previous Root snapshot.
- Root records the rollback as a history event.
- Root clearly explains what was reverted.
- If no rollback is available, Root prints a helpful message.

### Doctor

As a developer, I want to check whether Root is ready so that I can trust it before installing tools.

Acceptance criteria:

- `root doctor` checks Nix availability.
- `root doctor` checks Root data directory.
- `root doctor` checks Root profile/path configuration.
- `root doctor` checks whether history storage is writable.
- `root doctor` prints next-step instructions for failed checks.

## Out of Scope

- Installing arbitrary packages beyond the explicitly supported package allowlist.
- Supporting casks or GUI apps.
- Replacing all Homebrew usage.
- AI agent runtime.
- Permissions.
- Sandboxes.
- Machine sync.
- Menu bar app.
- Cloud features.
- Team features.
- Linux and Windows support.

## Platform

v0.1 target:

- macOS
- Apple Silicon first
- Nix installed or installable
- Zsh shell environment

## UX Requirements

Root must never expose raw Nix complexity unless in debug mode.

Bad:

```text
attribute 'aarch64-darwin' missing
```

Good:

```text
Root could not find a compatible ffmpeg package for your Mac.
Run `root doctor` for details.
```

Root should always communicate:

- The action being performed.
- The snapshot created.
- The verification result.
- The rollback availability.

## CLI Output Principle

Every command should feel calm, short, and trustworthy.

Do not over-explain.
Do not dump raw logs.
Do not pretend more safety than exists.

## Example Install Output

```text
Root is planning this install:

Package:
  ffmpeg

Before:
  snapshot root-2026-06-03-1422 created

Installing with Nix...
Verifying ffmpeg...

Done.

Changed:
  + ffmpeg

Verified:
  ffmpeg -version succeeded

Rollback:
  root rollback
```

## Example History Output

```text
Root history

2026-06-03 14:22
  install ffmpeg
  status: verified
  snapshot: root-2026-06-03-1422

2026-06-03 14:30
  rollback
  status: completed
  restored: root-2026-06-03-1422
```

## Example Doctor Output

```text
Root doctor

✓ Nix installed
✓ Root data directory writable
✓ History store writable
✓ Root profile available
✓ ffmpeg support available

Root is ready.
```

## Risks

### Risk: v0.1 is too narrow

Mitigation:
Position it as a believable first release, not the full product.

### Risk: Nix setup complexity derails onboarding

Mitigation:
`root doctor` must clearly detect and explain setup problems.

### Risk: rollback is not truly system-wide

Mitigation:
Be honest. Rollback only applies to Root-managed state.

### Risk: users expect arbitrary package install

Mitigation:
v0.1 can support an allowlist and clearly say more packages are coming.

## Launch Criteria

Root v0.1 can be launched publicly when:

- `root doctor` works.
- `root install ffmpeg` works on a clean test Mac.
- `root history` records install and rollback events.
- `root rollback` restores the previous managed state.
- README includes accurate limitations.
- Demo video shows the full trust loop.
