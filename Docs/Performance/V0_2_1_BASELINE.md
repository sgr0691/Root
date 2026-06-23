# Root v0.2.1 — Baseline Performance Audit

Date: 2026-06-22
Build: `cargo build` (debug/dev profile)

## Command Timings

| Command | Time (debug) | Notes |
|---------|-------------|-------|
| `root --version` | ~2ms | No config required |
| `root search terraform` | ~12ms | In-memory catalog scan |
| `root search node` | ~8ms | In-memory catalog scan |
| `root status` | ~13ms | Reads lockfile, calls Nix |
| `root history` | ~7ms | Reads events.jsonl, snapshots dir |
| `root verify` | N/A | Requires installed package |
| `root run --help` | ~3ms | CLI help generation |
| `root sandbox list` | N/A | Requires Docker |
| `cargo build` (incremental) | ~0.6s | Workspace rebuild |

## Bottlenecks Identified

1. **Search allocations**: `search_match_for_package` lowercases the query per-package (42 times). `SearchMatch` and `CatalogEntry` allocate `Vec<String>` for aliases, binaries, and `matched_fields` on every call.

2. **Lockfile rewrite**: `save_lock_v2` and `save_lock` always write via `atomic_write` — no content comparison. Every mutation command rewrites, even if contents are identical.

3. **build_v2_lock waste**: Converts v2→v1 (via `legacy_lock_from_v2`) just to pass a few fields to `build_v2_lock` which creates a new v2.

4. **Event ledger**: `read_events` loads every event line from `events.jsonl` into memory. No limit/cap. Scales O(n) with history size.

5. **Status command**: Calls `profile_packages(adapter)` which shells out to Nix even when there are zero packages to check.

6. **No caching**: Search index is rebuilt on every invocation (acceptable for 42 packages, but still unnecessary work).

## Improvement Opportunities

- Use `&'static [&'static str]` for alias/binary fields in `SearchMatch` and `CatalogEntry`
- Pre-compute lowercase query once, pass through call chain
- Content-aware write: check if serialized output matches existing file before writing
- Refactor `build_v2_lock` to take `&RootLockV2` directly
- Add `--limit` flag to `history` to cap event loading
- Skip Nix profile check in `status` when Rootfile/lockfile are empty
- Remove dead code (`legacy_lock_from_v2`, `legacy_package_from_v2`)
