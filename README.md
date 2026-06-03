# Root v0.1.1

> A Nix-backed trust demo for safe installs, history, and rollback.

Root installs developer CLI tools through Nix, records what changed, and lets you
undo it — without needing to learn Nix.

[![CI](https://github.com/sgr0691/Root/actions/workflows/ci.yml/badge.svg)](https://github.com/sgr0691/Root/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/sgr0691/Root/main/scripts/install.sh | sh
```

## Quick Start (v0.1.1 — trust demo)

```bash
root doctor
root install ffmpeg
root history
root rollback
root history
```

## Core Commands (v0.1)

| Command | Description |
|---------|-------------|
| `root doctor` | Check that Root and Nix are ready |
| `root install ffmpeg` | Install `ffmpeg` via Nix with snapshot |
| `root history` | Show the event ledger |
| `root rollback` | Undo the last Root-managed change |

Only `ffmpeg` is officially supported in v0.1.

## How It Works

Root manages an isolated Nix profile at `~/.root/profiles/default` — it never
touches your default Nix or Homebrew profiles.

Every `root install` and `root rollback` creates a snapshot first. The event
ledger at `~/.root/events.jsonl` records every operation.

## Limitations (v0.1)

- **Only `ffmpeg` is supported.** Installing any other package is rejected with a
  clear message.
- **Rollback applies only to Root-managed packages.** Root cannot undo changes
  made by Homebrew, manual installs, or other tools.
- **Version tracking uses `"latest"`.** Root does not yet resolve and lock exact
  Nix derivation versions.
- **Stale lockfiles.** If Root crashes during a mutation, delete `~/.root/root.lockfile`
  manually to recover.
- **macOS only.** Apple Silicon and Intel are supported. Linux is detected but not
  officially supported. Windows is not available.

## Experimental Commands

The CLI includes additional commands that are **not part of the v0.1 public
surface**. They may change, break, or be removed without notice:

| Command | Status |
|---------|--------|
| `root init` | Experimental |
| `root plan install <pkg>` | Experimental |
| `root remove <pkg>` | Experimental |
| `root list` | Experimental |
| `root verify <pkg>` | Experimental |
| `root lock` / `root sync` | Experimental |
| `root import brew` | Experimental |
| `--json` on all commands | Experimental |

These exist for development and early testing. Do not rely on them for
production use.

## Roadmap

- **v0.2** — More golden packages (ripgrep, jq, poppler, imagemagick)
- **v0.3** — Agent skill packs for Codex, Claude, and Cursor
- **v0.4** — Full-lockfile determinism, version pinning
- **v0.5** — Linux support (platform detection already in place)
- **Future** — Permissions, sandboxes, project environments, menu bar app

See [Docs](Docs/) for the full plan.

## Safety

- Snapshots before every mutation
- Rollback is available after every install
- Nix profile isolation — no global PATH pollution
- Structured event ledger — every change is recorded
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
