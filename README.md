# Root v0.1.3

> A curated package manager for developer CLI tools, backed by Nix.

Root installs developer CLI tools through Nix, records what changed, and lets you
undo it — without needing to learn Nix.

[![CI](https://github.com/sgr0691/Root/actions/workflows/ci.yml/badge.svg)](https://github.com/sgr0691/Root/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/sgr0691/Root/main/scripts/install.sh | sh
```

## Try Root in 60 seconds

```bash
# 1. Browse the curated catalog
root catalog

# 2. Preview what install would do
root plan install ripgrep

# 3. Install
root install ripgrep

# 4. See what happened
root history

# 5. Verify the binary works
root verify ripgrep

# 6. Undo the install
root rollback --last
```

That's it. Every install is recorded, every binary is verified, and every change
can be undone — all without learning Nix.

## Core Commands

| Command | Description |
|---------|-------------|
| `root catalog` | Browse the curated package catalog |
| `root doctor` | Check that Root and Nix are ready |
| `root install ripgrep` | Install a package via Nix with deterministic lock |
| `root plan install ripgrep` | Preview what an install would do (no changes made) |
| `root history` | Show snapshot summaries and event ledger |
| `root verify ripgrep` | Verify installed package binaries are functional |
| `root rollback --last` | Roll back to the last snapshot using locked state |

## Supported Packages

Root curates a catalog of 24 developer CLI tools across four categories:

### media
| Package | Description | Nix attribute | Binaries | Verify |
|---------|-------------|---------------|----------|--------|
| ffmpeg | Video/audio processing | nixpkgs#ffmpeg | ffmpeg | ffmpeg -version |
| imagemagick | Image manipulation | nixpkgs#imagemagick | magick, convert | magick --version |
| poppler | PDF utilities | nixpkgs#poppler | pdftotext, pdfinfo | pdftotext -v, pdfinfo -v |

### search
| Package | Description | Nix attribute | Binaries | Verify |
|---------|-------------|---------------|----------|--------|
| ripgrep | Fast recursive search | nixpkgs#ripgrep | rg | rg --version |
| fd | Fast file finder | nixpkgs#fd | fd | fd --version |
| fzf | Fuzzy file finder | nixpkgs#fzf | fzf | fzf --version |

### dev
| Package | Description | Nix attribute | Binaries | Verify |
|---------|-------------|---------------|----------|--------|
| bat | File viewer with syntax highlighting | nixpkgs#bat | bat | bat --version |
| bun | JavaScript runtime and bundler | nixpkgs#bun | bun | bun --version |
| eza | Modern ls replacement | nixpkgs#eza | eza | eza --version |
| gh | GitHub CLI | nixpkgs#gh | gh | gh --version |
| git-lfs | Git large file storage | nixpkgs#git-lfs | git-lfs | git-lfs --version |
| gnumake | Build automation | nixpkgs#gnumake | make | make --version |
| httpie | HTTP client | nixpkgs#httpie | http | http --version |
| jq | JSON processor | nixpkgs#jq | jq | jq --version |
| just | Command runner | nixpkgs#just | just | just --version |
| nodejs | JavaScript runtime | nixpkgs#nodejs | node, npm | node --version |
| openssl | Cryptography toolkit | nixpkgs#openssl | openssl | openssl version |
| pkg-config | Package configuration | nixpkgs#pkg-config | pkg-config | pkg-config --version |
| python3 | Python interpreter | nixpkgs#python3 | python3 | python3 --version |
| sqlite | SQL database engine | nixpkgs#sqlite | sqlite3 | sqlite3 --version |
| tree | Directory tree viewer | nixpkgs#tree | tree | tree --version |
| uv | Python package manager | nixpkgs#uv | uv | uv --version |

### net
| Package | Description | Nix attribute | Binaries | Verify |
|---------|-------------|---------------|----------|--------|
| curl | URL transfer tool | nixpkgs#curl | curl | curl --version |
| wget | URL downloader | nixpkgs#wget | wget | wget --version |

Each package's metadata (Nix attribute, expected binaries, and verification
commands) is defined in the `PackageSpec` catalog inside `root-core`. New
packages are easy to add without changing the install or lock logic.

Run `root catalog` to see the full list at any time.

## Why curated packages first?

Root uses a curated allowlist for safety:

1. **Predictable behavior.** Every supported package has well-known Nix
   attribute names, binary names, and verification commands. No surprises.
2. **Deterministic installs.** The package catalog provides the metadata
   needed for fully deterministic v2 lockfiles (correct binary names, proper
   store path tracking).
3. **Error prevention.** Unsupported packages are rejected before any Nix
   call — no waiting for a failed Nix build or wrong attribute name.
4. **Testable surface.** The curated set is easy to test end-to-end. Every
   package is validated for unique names, valid attributes, and at least one
   verification command.

Arbitrary `root install <anything>` support is planned for a future release.
Until then, unsupported packages get a clear error message with the full catalog.

## Example flow

```bash
# List what's available
root catalog

# Preview before installing
root plan install ripgrep

# Install
root install ripgrep

# Verify the binary works from the Root-managed profile
root verify ripgrep

# See all operations
root history

# Undo the last change
root rollback --last
```

## What v0.1.3 Changed

v0.1.3 is the **Curated Package Catalog** release:

- **Expanded catalog** — From 4 to 24 curated packages across `media`,
  `search`, `dev`, and `net` categories. Includes fd, bat, eza, fzf,
  git-lfs, gh, httpie, just, tree, sqlite, imagemagick, wget, curl,
  gnumake, pkg-config, openssl, python3, nodejs, bun, and uv.
- **`root catalog` command** — Lists all supported packages grouped by
  category.
- **Rich `PackageSpec` metadata** — Each package defines aliases, Nix
  attributes, expected binaries, per-binary verification commands,
  category, and description. The catalog is easy to extend.
- **Better unsupported-package errors** — Rejection messages now show
  categorized package lists so users can discover alternatives.
- **Full verification coverage** — Every supported package has at least
  one verification command. `root verify <pkg>` checks the Root-managed
  profile path, not the user's global PATH.

## How It Works

Root manages an isolated Nix profile at `~/.root/profiles/default` — it never
touches your default Nix or Homebrew profiles.

Every `root install` and `root rollback` creates a snapshot first. Snapshots
contain the full deterministic lock state. The event ledger at
`~/.root/events.jsonl` records every operation. Verification checks binaries
from the Root-managed profile, not from PATH.

## Limitations (v0.1.3)

- **Curated catalog only.** Arbitrary `root install <anything>` is not yet
  supported. Unsupported packages are rejected with a clear categorized
  message.
- **Rollback applies only to Root-managed packages.** Root cannot undo
  changes made by Homebrew, manual installs, or other tools.
- **Nix must be installed.** Root manages a Nix profile but does not
  bundle Nix.
- **Stale lockfiles.** If Root crashes during a mutation, delete
  `~/.root/root.lockfile` manually to recover.
- **macOS only.** Apple Silicon and Intel are supported. Linux is detected
  but not officially supported. Windows is not available.

## Experimental Commands

The CLI includes additional commands that are **not part of the v0.1.3 public
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

- **v0.2** — Arbitrary package install support, expanded platform support
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
