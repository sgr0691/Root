# Root v0.1.2

> A Nix-backed trust demo for safe installs, history, and rollback.

Root installs developer CLI tools through Nix, records what changed, and lets you
undo it — without needing to learn Nix.

[![CI](https://github.com/sgr0691/Root/actions/workflows/ci.yml/badge.svg)](https://github.com/sgr0691/Root/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/sgr0691/Root/main/scripts/install.sh | sh
```

## Quick Start (v0.1.2)

```bash
root doctor
root install ffmpeg
cat ~/.root/root.lock     # real /nix/store/ paths, no "latest"
root history              # snapshots + events
root verify ffmpeg        # checks binary from Root-managed profile
root rollback             # uses locked state, not moving nixpkgs
```

## Core Commands (v0.1.2)

| Command | Description |
|---------|-------------|
| `root doctor` | Check that Root and Nix are ready |
| `root install ffmpeg` | Install a package via Nix with deterministic lock |
| `root history` | Show snapshot summaries and event ledger |
| `root rollback` | Roll back to the last snapshot using locked state |
| `root verify <pkg>` | Verify installed package binaries are functional |

Supported packages in v0.1.2: `ffmpeg`, `poppler`.

## What v0.1.2 Changed

v0.1.2 is a **correctness release**. It replaces the v0.1.1 placeholder lock
format with real Nix metadata:

- **Real Nix store paths** — `nix build --print-out-paths` captures actual
  `/nix/store/...` paths instead of invented hashes.
- **Real package versions** — `nix eval` reads the real `name` and `version`
  from the Nix derivation; `"latest"` is never written.
- **Locked nixpkgs** — `nix flake metadata --json` pins the exact nixpkgs
  revision, nar hash, and flake reference.
- **Snapshot v2** — Snapshots store the full lock state so rollback uses locked
  installables, not name-only resolution.
- **Rollback by locked state** — Restores using the saved installable
  (`github:NixOS/nixpkgs/<rev>#<attr>`) rather than resolving `nixpkgs#<pkg>`.
- **Legacy detection** — v1 locks with `"latest"` or placeholder paths are
  detected and flagged by `root doctor`.

## How It Works

Root manages an isolated Nix profile at `~/.root/profiles/default` — it never
touches your default Nix or Homebrew profiles.

Every `root install` and `root rollback` creates a snapshot first. Snapshots
contain the full deterministic lock state. The event ledger at
`~/.root/events.jsonl` records every operation. Verification checks binaries
from the Root-managed profile, not from PATH.

## Limitations (v0.1.2)

- **Limited package set.** Only `ffmpeg` and `poppler` are on the allowlist.
  Installing any other package is rejected with a clear message.
- **Rollback applies only to Root-managed packages.** Root cannot undo changes
  made by Homebrew, manual installs, or other tools.
- **Nix must be installed.** Root manages a Nix profile but does not bundle Nix.
- **Stale lockfiles.** If Root crashes during a mutation, delete
  `~/.root/root.lockfile` manually to recover.
- **macOS only.** Apple Silicon and Intel are supported. Linux is detected but
  not officially supported. Windows is not available.

## Experimental Commands

The CLI includes additional commands that are **not part of the v0.1.2 public
surface**. They may change, break, or be removed without notice:

| Command | Status |
|---------|--------|
| `root init` | Experimental |
| `root plan install <pkg>` | Experimental |
| `root remove <pkg>` | Experimental |
| `root list` | Experimental |
| `root lock` | Experimental |
| `root sync` | Experimental — does not operate on v2 locks |
| `root import brew` | Experimental |
| `--json` on all commands | Experimental |

These exist for development and early testing. Do not rely on them for
production use.

## Roadmap

- **v0.2** — More golden packages (ripgrep, jq, poppler, imagemagick)
- **v0.3** — Agent skill packs for Codex, Claude, and Cursor
- **v0.4** — Linux support (platform detection already in place)
- **v0.5** — Permissions, sandboxes, project environments, menu bar app
- **Future** — Desktop app, team sync, cloud features

See [Docs](Docs/) for the full plan.

## Safety

- Snapshots before every mutation
- Rollback by locked state — not by package name
- Nix profile isolation — no global PATH pollution
- Structured event ledger — every change is recorded
- Post-install and post-rollback profile verification
- All Nix operations target `~/.root/profiles/default`, not the user profile

## Development

```bash
cargo build
cargo test --all
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

## License

Apache 2.0
