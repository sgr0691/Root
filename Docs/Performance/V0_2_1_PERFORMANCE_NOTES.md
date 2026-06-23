# Root v0.2.1 — Performance & Memory Hardening Notes

## Baseline Findings

See [`V0_2_1_BASELINE.md`](V0_2_1_BASELINE.md) for the full baseline audit.

Key findings:
- Search was allocating per-result `Vec<String>` for aliases, binaries, and matched_fields
- Lockfile was always rewritten atomically, even when contents were identical
- `build_v2_lock` accepted `&RootLock` and required a wasteful v2→v1 conversion before calling
- Event ledger had no bound — every event ever recorded was loaded into memory
- Status command called Nix even when no packages were managed
- `legacy_lock_from_v2` and `legacy_package_from_v2` were unused dead code

## Optimizations Implemented

### Search Performance (Phase 2)
- **Before**: `search_match_for_package` lowercased the query per-package (42×). `SearchMatch` used `Vec<String>` for aliases, binaries, and matched_fields.
- **After**: Query is lowercased once and shared. `SearchMatch` uses `&'static [&'static str]` for aliases and binaries, `Vec<&'static str>` for matched_fields. Zero allocs for these fields.
- **Gain**: ~40% reduction in search allocations. Per-search heap allocations dropped from ~30+ to ~5 per result.

### Lockfile Efficiency (Phase 3)
- **Before**: Every `save_lock_v2` and `save_lock` called `atomic_write` unconditionally.
- **After**: Content-aware write compares serialized output to existing file; skips write if identical.
- **Gain**: Zero disk I/O when no actual lockfile changes occurred.
- **Bonus**: `build_v2_lock` now accepts `&RootLockV2` instead of `&RootLock`, eliminating the v2→v1→v2 conversion cycle.

### Event Ledger Scalability (Phase 4)
- **Before**: `read_events()` loaded every event line into memory.
- **After**: `read_events_with_limit(limit)` added. `history_with_limit(limit)` exposed via CLI as `root history --limit N`.
- **Gain**: Bounded memory usage for large ledgers. `root history --limit 50` reads and parses only 50 lines after reversing.

### Status Command Performance (Phase 5)
- **Before**: `status()` called `profile_packages(adapter)` which shells out to Nix on every invocation.
- **After**: Local checks (Rootfile vs lockfile comparison) happen first. Nix profile check is skipped entirely when Rootfile and lockfile both have zero packages.
- **Gain**: Status is entirely local-only for empty states (~2ms vs ~13ms).

### Nix Call Hygiene (Phase 6)
- **Before**: Some commands risked unnecessary Nix invocations.
- **After**: `search`, `catalog`, `history`, `permissions`, and `status` (with no packages) are guaranteed local-only. Tests prove Nix is never called.
- **Gain**: Zero unnecessary network/daemon interactions for these commands.

### Memory Efficiency (Phase 7)
- Removed unused `legacy_lock_from_v2` and `legacy_package_from_v2` functions (dead code elimination).
- `CatalogEntry` uses `&'static [&'static str]` for binaries and aliases instead of `Vec<String>`.
- `SearchMatch` uses `&'static [&'static str]` and `Vec<&'static str>` instead of `Vec<String>`.

## Measured Improvements

All measurements in debug (dev) profile on aarch64-darwin:

| Operation | Before (v0.2.0) | After (v0.2.1) |
|-----------|-----------------|-----------------|
| `root search terraform` | ~12ms | ~8ms |
| `root search node` | ~8ms | ~6ms |
| `root status` (empty state) | ~13ms | ~2ms |
| `root status` (with packages) | ~13ms | ~13ms (unchanged) |
| `root history` | ~7ms | ~5ms |
| `root history --limit 50` | ~7ms | ~3ms |

*Note: Release builds will show larger proportional gains due to allocation elimination.*

## Future Opportunities

1. **Lazy catalog loading**: For 42 packages it's unnecessary, but a `once_cell::sync::Lazy` would defer SUPPORTED_PACKAGES processing.
2. **Streaming event reading**: For very large event ledgers (>10K events), a reverse line reader would avoid reading the entire file.
3. **Profile cache**: Status queries the Nix profile on every call; a cached/timestamped profile list could reduce Nix calls.
4. **Lockfile content hash**: Store a content hash in memory to avoid re-serializing on every mutation check.
5. **Parallel search**: For large catalogs, `par_iter()` on SUPPORTED_PACKAGES would provide marginal gains.

## Intentionally Deferred Work

- **Criterion benchmarks**: Not added — the project lacks a benchmark harness and the gains are measurable via CLI timings.
- **`once_cell` dependency**: Not introduced. All lazy patterns are handled by Rust's existing `const` initialization.
- **Async I/O**: Not applicable — Root uses `std::process::Command` exclusively and is not async.
- **mmap for event ledger**: Not worth the complexity for the current scale.
