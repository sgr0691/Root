# Sandbox Audit — Root v0.2.1

**Date:** 2026-06-23  
**Scope:** Full sandbox subsystem across `root-sandbox`, `root-core`, `root-cli`, and `policy`  
**Auditor:** Automated codebase analysis  
**Version:** 0.2.1 (workspace Cargo.toml)

---

## Executive Summary

- **Sandbox provider:** Docker-only (`RealSandboxProvider` in `crates/root-sandbox/src/lib.rs`)
- **CLI commands exposed:** `root sandbox create`, `root sandbox run`, `root sandbox list`, `root sandbox destroy`
- **Sandbox trait** (`SandboxProvider`): 5 methods — `check_availability`, `create`, `run_command`, `list`, `destroy`
- **Policy integration:** 3 `PolicyAction` variants for sandbox (`SandboxCreate`, `SandboxRun`, `SandboxDestroy`)
- **Event recording:** All sandbox operations record a `RootEventType::Sandbox` event
- **Total gaps identified: 12** (see Gap Summary below)

---

## Gap Summary

| # | Gap | Severity | Area | Category |
|---|-----|----------|------|----------|
| 1 | No sandbox lifecycle model — no state machine, no state tracking | Critical | Core | Lifecycle |
| 2 | No timeout enforcement for running commands | High | sandbox-provider | Resource Mgmt |
| 3 | `destroy` only verifies name prefix, not ownership metadata | Medium | sandbox-provider | Security |
| 4 | No orphan cleanup on partial failure — stale containers leak | High | sandbox-provider | Resource Mgmt |
| 5 | `list` does not populate `created_at` (empty string) | Low | sandbox-provider | Data Quality |
| 6 | No periodic or TTL-based expiry of sandboxes | High | System | Resource Mgmt |
| 7 | `run_command` uses raw `id` with no ownership check | Medium | sandbox-provider | Security |
| 8 | Event recording swallows errors (unused `let _`) | Low | Core | Reliability |
| 9 | No `started_at` / `duration_ms` in sandbox events | Medium | Core | Observability |
| 10 | No sandbox-Nix ecosystem unification | Low | Design | Architecture |
| 11 | No sandbox `exec` interactive mode / TTY support | Low | CLI | UX |
| 12 | `image` parameter accepts arbitrary images — no digest pinning | Medium | sandbox-provider | Security |

---

## 1. Architecture Overview

### Files Involved

| File | Role |
|------|------|
| `crates/root-sandbox/src/lib.rs` | `SandboxProvider` trait + `RealSandboxProvider` (Docker) + `MockSandboxProvider` (tests) |
| `crates/root-core/src/lib.rs` (lines 2255–2284, 2846–2967) | Report structs + sandbox orchestration functions |
| `crates/root-core/src/policy.rs` (lines 73–90, 167–170) | `SandboxPolicy` struct + `PolicyAction::Sandbox*` |
| `crates/root-core/src/events.rs` | `RootEventType::Sandbox` + event recording |
| `crates/root-cli/src/main.rs` (lines 114–168, 310–317, 987–1055) | CLI subcommand parsing, provider instantiation, dispatch |

### Dependency Graph

```
root-cli (bin)
  ├── root-core        (sandbox_create, sandbox_run, sandbox_list, sandbox_destroy)
  │     ├── root-sandbox   (SandboxProvider trait)
  │     └── root-lockfile  (init_root_dir)
  └── root-sandbox     (RealSandboxProvider)
```

---

## 2. Operation-by-Operation Audit

### 2.1 `sandbox create`

**CLI:** `root sandbox create [NAME] [--image IMAGE]`  
**Rust function:** `sandbox_create()` at `crates/root-core/src/lib.rs:2846`  
**Provider method:** `RealSandboxProvider::create()` at `crates/root-sandbox/src/lib.rs:79`

#### Expected Behavior
1. Initializes `~/.root` directory via `init_root_dir()`.
2. Enforces policy (`SandboxCreate`) — denied => error exit.
3. Checks Docker availability via `docker info`.
4. Calls provider `create(name, image)`.
5. Provider: sanitizes name as `root-sandbox-{name}`, force-removes any existing container with that name (`docker rm -f`), runs `docker run -d --name <name> <image> sleep infinity`, inspects container ID.
6. Records a `Sandbox` / `Completed` event.
7. Returns `SandboxCreateReport`.

#### Failure Modes
| Condition | Behavior | Exit Code | Gap? |
|-----------|----------|-----------|------|
| Docker not on PATH | `SandboxError::NotAvailable` → pretty-printed error advising Docker install | 1 | No |
| Docker daemon not running | `docker info` returns false → same as above | 1 | No |
| Policy denies `SandboxCreate` | `enforce_policy` returns `Err` → `Policy denied` → exit code 9 | 9 | No |
| `docker rm -f` on pre-existing fails | Error silently ignored | N/A | **Gap 8** |
| `docker run` fails (bad image, OOM, etc.) | `SandboxError::Generic` from run_docker | 1 | No |
| Image not found on Docker Hub | `docker run` fails → generic error "Unable to find image" | 1 | No |
| No `sleep infinity` available in image | `docker run` fails — container exits immediately | 1 | No |
| `docker run` succeeds but `docker inspect` fails | Container leaked — Root cannot manage it | 1 | **Gap 4** |

#### Cleanup Behavior
- **On success:** None needed — container is running.
- **On failure:** The `docker rm -f` pre-clean only runs if an existing container with the same `root-sandbox-{name}` exists. If `docker run` succeeds but `docker inspect` fails, the container is **orphaned** — no rollback/deletion occurs. **(Gap 4)**
- The pre-existing container removal is silent (`let _ = Self::run_docker(...)`) — errors are swallowed.

#### Current Gaps
- **Gap 4:** No cleanup on partial failure. If `docker run -d` creates the container but the subsequent `docker inspect` fails, the container is leaked.
- **Gap 2:** No timeout on `docker run`. A slow image pull blocks indefinitely.
- **Gap 12:** `image` defaults to `ubuntu:latest` (floating tag) — no digest pinning, no validation.

---

### 2.2 `sandbox run`

**CLI:** `root sandbox run <ID> [-- <COMMAND>...]`  
**Rust function:** `sandbox_run()` at `crates/root-core/src/lib.rs:2892`  
**Provider method:** `RealSandboxProvider::run_command()` at `crates/root-sandbox/src/lib.rs:108`

#### Expected Behavior
1. Initializes `~/.root` directory.
2. Enforces policy (`SandboxRun`).
3. Calls `provider.run_command(id, command)` which executes `docker exec <id> <command...>`.
4. Captures stdout, stderr, exit code.
5. Records event with status `Completed` (exit 0) or `Failed` (nonzero).
6. Returns `SandboxRunReport`.
7. CLI: if `!report.success`, exits with `report.exit_code.max(1)`.

#### Failure Modes
| Condition | Behavior | Exit Code | Gap? |
|-----------|----------|-----------|------|
| Policy denies | Same as create — exit 9 | 9 | No |
| Sandbox ID not found | `docker exec` fails → `SandboxError::Generic` → exit 1 | 1 | No |
| Command not found inside container | `docker exec` returns exit 127 | propagated | No |
| Container stopped/paused | `docker exec` fails → generic error | 1 | No |
| Arbitrary container ID passed | `docker exec` still runs — no ownership check | 1 | **Gap 7** |
| Command hangs (infinite loop) | Process blocks indefinitely | N/A | **Gap 2** |

#### Cleanup Behavior
- None. The command's side effects inside the container are intentional; no cleanup is expected.

#### Current Gaps
- **Gap 2 (repeated):** `docker exec` has no timeout. Hanging commands block the Root CLI forever.
- **Gap 7 (new):** `run_command` passes the `id` directly to `docker exec` with no verification that the container is Root-owned (unlike `destroy` which checks `root-sandbox-` prefix). An attacker or user error can execute commands in *any* Docker container.
- **Gap 9:** Events record only `message` with exit code text; no `started_at`, `finished_at`, or `duration_ms` fields are populated.

---

### 2.3 `sandbox list`

**CLI:** `root sandbox list`  
**Rust function:** `sandbox_list()` at `crates/root-core/src/lib.rs:2934`  
**Provider method:** `RealSandboxProvider::list()` at `crates/root-sandbox/src/lib.rs:124`

#### Expected Behavior
1. Calls `provider.list()` which runs `docker ps -a --filter name=root-sandbox- --format '{{.ID}}\t{{.Names}}\t{{.Status}}\t{{.Image}}'`.
2. Parses tab-separated output into `Vec<SandboxInstance>`.
3. Returns `SandboxListReport`.

#### Failure Modes
| Condition | Behavior | Exit Code | Gap? |
|-----------|----------|-----------|------|
| Docker unavailable | Propagates error | 1 | No |
| No containers match filter | Returns empty list | 0 | No |
| Malformed `docker ps` output | Silently drops lines with <4 parts | 0 (partial) | **Gap 5** |

#### Cleanup Behavior
- Read-only operation. No cleanup needed.

#### Current Gaps
- **Gap 5:** `created_at` is always `String::new()` (empty string) because `docker ps --format` output does not include creation time in the template. The field is silently missing.
- **Gap 5b:** Lines with unexpected format are silently dropped. A corrupt Docker output could lead to an incomplete list.
- No policy enforcement for `list` — `sandbox_list()` does NOT call `enforce_policy`. This is arguably correct (list is informational), but inconsistent with the other three operations.

---

### 2.4 `sandbox destroy`

**CLI:** `root sandbox destroy <ID>`  
**Rust function:** `sandbox_destroy()` at `crates/root-core/src/lib.rs:2945`  
**Provider method:** `RealSandboxProvider::destroy()` at `crates/root-sandbox/src/lib.rs:160`

#### Expected Behavior
1. Initializes `~/.root` directory.
2. Enforces policy (`SandboxDestroy`).
3. Calls `provider.destroy(id)`.
4. Provider: inspects container (`docker inspect --format '{{.Name}}' <id>`), verifies name starts with `root-sandbox-`, then force-removes (`docker rm -f`).
5. Records `Completed` event.
6. Returns `SandboxDestroyReport`.

#### Failure Modes
| Condition | Behavior | Exit Code | Gap? |
|-----------|----------|-----------|------|
| Policy denies | Exit 9 | 9 | No |
| Container not found | `docker inspect` fails → `SandboxError::NotFound` → exit 1 | 1 | No |
| Container not Root-owned | Name does not start with `root-sandbox-` → `SandboxError::NotRootOwned` → exit 1 | 1 | No |
| `docker rm -f` fails (permission, etc.) | `SandboxError::Generic` | 1 | No |
| Container is from a different Root instance (same prefix) | Name check passes — destroys anyway | 0 | **Gap 3** |
| Container already removed | `docker inspect` fails → mapped to `NotFound` | 1 | Correct |

#### Cleanup Behavior
- Force-removes the container. This is the cleanup operation itself — it is the intended cleanup.
- No cleanup if `destroy` itself fails partway. If `docker inspect` succeeds but `docker rm -f` fails, the container is left in an unknown state but still running.

#### Current Gaps
- **Gap 3:** Ownership check is name-prefix based (`root-sandbox-`). This is a weak heuristic — any container prefixed with `root-sandbox-` from any source is considered Root-owned. There is no label-based ownership (e.g., Docker labels), no checksum, no signature.
- **Gap 3b:** The ownership check queries the container name via `docker inspect`, then strips the leading `/`, then checks the prefix. If Docker ever changes the output format of `{{.Name}}`, this check could silently pass or fail.
- If `destroy` is called on an already-destroyed container, the `docker inspect` fails with "No such object" which maps to `NotFound` — this is correct behavior.

---

## 3. Lifecycle Model

### Current State

**There is no lifecycle model.** The system has a flat set of 4 operations (create, run, list, destroy) with no state machine:

```
   ┌──────────┐
   │  absent  │
   └────┬─────┘
        │ create
        ▼
   ┌──────────┐
   │ running  │ ← (from docker ps output)  ←──┐
   └────┬─────┘                                │
        │ destroy                               │ docker stop (external)
        ▼                                       │
   ┌──────────┐   docker start (external) ──────┘
   │  absent  │
   └──────────┘

   No: paused, stopped, starting, error, expired states
   No: explicit state transitions managed by Root
   No: state persistence in events or lockfile
```

### Gap Analysis

| Missing Concept | Impact |
|-----------------|--------|
| No state enum | `SandboxInstance.status` is a raw `String` from Docker (e.g., "Up 2 hours", "Exited (0)") — not normalized beyond the "running" / "Up" check in `list()`. |
| No sandbox registry | Sandboxes are not tracked in `~/.root/` — no lockfile entry, no fingerprint. Recovery after `docker system prune` is impossible. |
| No expiry / TTL | Sandboxes run forever until explicitly destroyed. Long-lived sandboxes accumulate resources. |
| No lifecycle hooks | No pre-create, post-create, pre-destroy, post-destroy hooks. |

**Gap 1 (Critical):** The entire lifecycle is delegated to Docker with no Root-level abstraction. Root cannot answer "what sandboxes exist?" without querying Docker. If Docker metadata is lost (prune, reset, reinstall), Root has zero awareness of its sandboxes.

---

## 4. Event Recording Quality

### What is recorded

All sandbox operations call `events::record_event()` with:
- `event_type`: `RootEventType::Sandbox`
- `status`: `Completed` or `Failed`
- `command`: `"root sandbox create {name}"`, `"root sandbox run {id}"`, `"root sandbox destroy {id}"`
- `message`: Descriptive text
- All other fields: `None`

### What is NOT recorded

| Field | Create | Run | Destroy | Gap? |
|-------|--------|-----|---------|------|
| `package` | None (N/A) | None | None | — |
| `snapshot_id` | None | None | None | — |
| `task_name` | None | None | None | #9 |
| `exit_code` | None | None | None | #9 |
| `started_at` | None | None | None | #9 |
| `finished_at` | None | None | None | #9 |
| `duration_ms` | None | None | None | #9 |
| `policy_decision` | None | None | None | #9 |

**Gap 9 (Medium):** Sandbox events are low-fidelity compared to execution events (`record_execution_event`), which capture `exit_code`, `started_at`, `finished_at`, `duration_ms`, and `task_name`. Sandbox events use the generic `record_event` function and miss all timing and exit information.

**Gap 8 (Low):** All `record_event` calls use `let _ = ...`, which silently ignores write failures (disk full, permission denied, etc.). If event recording fails, the sandbox operation still reports success to the user — a silent data loss.

---

## 5. Error Handling Quality

### Error Type Hierarchy

```
SandboxError (root-sandbox)
  ├── NotAvailable(String)    — Docker not on PATH / daemon not running
  ├── NotFound(String)        — Container ID not found
  ├── NotRootOwned(String)    — Container name doesn't start with root-sandbox-
  └── Generic(String)         — Any other Docker failure
```

### Mapping to CLI exit codes

The `exit_code_for_error()` function in `root-cli/src/main.rs` (line 194) maps errors:
- `SandboxError` types are NOT explicitly matched — they fall through to the generic `1` exit code.
- Only `NixError` variants and string-pattern matching on error messages produce specific exit codes (3, 4, 5, 6, 7, 8, 9).
- Sandbox-specific error codes (e.g., "Docker not available" → 7, "sandbox not found" → 3) are not defined for sandbox errors.

### Assessment

| Strength/Weakness | Detail |
|-------------------|--------|
| Good error categorization | `NotAvailable`, `NotFound`, `NotRootOwned`, `Generic` cover expected cases |
| Good provider error wrapping | `sandbox_create/run/destroy` wrap `SandboxError` in `anyhow::Error` with context |
| No sandbox-specific exit codes | All sandbox failures map to exit 1 (generic). No way to distinguish "Docker unavailable" from "container not found" in scripts. |
| Inconsistent `destroy` error on missing container | Missing container → `docker inspect` fails. The `map_err` at line 163 maps all inspect failures to `NotFound` — even when the actual error is something else like a daemon error. |
| `run_command` doesn't map `docker exec` failures correctly | If Docker returns a non-zero exit, `Command::new().output()` still succeeds (the process ran). Only if `Command::new()` itself fails (e.g., Docker binary missing) is an error returned. Non-zero exits from the executed command are reported in `SandboxExecResult.exit_code`, not as provider errors. |

---

## 6. Docker Dependency Assumptions

### Hard Dependency

The `RealSandboxProvider` has a hard dependency on the `docker` CLI being on `PATH`. There is:
- No fallback to Podman, containerd, or any other OCI runtime.
- No embedded Docker SDK (uses CLI subprocess).
- No graceful degradation — if Docker is absent, sandbox commands are completely non-functional.

### Assumptions Made

| Assumption | Risk |
|------------|------|
| `docker` binary is on PATH | If Docker Desktop is installed but CLI is not symlinked, `Command::new("docker")` fails. |
| `docker info` is sufficient availability check | Docker daemon could be running but resource-exhausted. |
| `docker exec` always attaches stdout/stderr correctly | Pseudo-TTY vs non-TTY differences (CLI does NOT use `-t` or `-i`). |
| `docker ps -a --filter name=root-sandbox-` returns all managed containers | If a user renames a container, it escapes management. |
| Tab-separated output parsing is stable | Docker output format changes could silently break parsing. |
| `docker inspect --format '{{.Name}}'` returns name starting with `/` | Brittle string manipulation (`trim_start_matches('/')`). |

### Test Isolation

- `MockSandboxProvider` in `crates/root-sandbox/src/lib.rs:173` provides full in-memory testing without Docker.
- The `SandboxProvider` trait enables swapping implementations, but the CLI always instantiates `RealSandboxProvider` (line 317 of `main.rs`).
- There are **no unit tests for `sandbox_create`, `sandbox_run`, `sandbox_list`, or `sandbox_destroy` in `root-core`** — only the `MockSandboxProvider` unit tests exist in `root-sandbox`. The orchestration layer (policy enforcement + event recording + provider call) in `root-core` is untested.

---

## 7. Resource Management Issues

### Identified Issues

| Issue | Details | Severity |
|-------|---------|----------|
| **Orphaned containers** | If `docker run` succeeds but `docker inspect` fails, the container is leaked with no cleanup. (Gap 4) | High |
| **No TTL / expiry** | Sandboxes run `sleep infinity` and persist until `destroy` is called. No automatic garbage collection. (Gap 6) | High |
| **No resource limits** | `docker run` is called without `--memory`, `--cpus`, `--pids-limit`, or any resource constraint. A sandbox can consume all host resources. (Gap 2) | Medium |
| **No network isolation controls** | Sandbox has full network access by default. `ResourcePolicy` exists but is not enforced at the Docker level — only at the policy-evaluation level. | Medium |
| **No storage limits** | No `--storage-opt` limits. A sandbox can fill the host disk. | Medium |
| **Accumulation of stopped containers** | `root sandbox list` shows all Root-named containers (including stopped/exited ones). No pruning of exited containers. (Gap 6) | Low |
| **Event log growth** | `events.jsonl` grows unboundedly. No log rotation or retention policy for sandbox events. | Low |

---

## 8. Security Analysis

| Concern | Detail | Severity |
|---------|--------|----------|
| **Weak ownership guard** | `destroy` only checks name prefix (`root-sandbox-`). No labels, no UUID, no signature. (Gap 3) | Medium |
| **No ownership check on `run_command`** | `run_command` accepts any Docker container ID/name — no `root-sandbox-` prefix check. (Gap 7) | Medium |
| **Floating image tags** | Default `ubuntu:latest` updates silently. No digest pinning. (Gap 12) | Medium |
| **No sandbox user isolation** | `docker exec` runs as root inside the container by default. Commands have full container root access. | Low |
| **No capability dropping** | Container runs with Docker's default capabilities, not a hardened subset. | Low |
| **Policy bypass surface** | `SandboxPolicy` controls access at the Root CLI level, but Docker CLI is still directly available — a user can bypass Root policy by running `docker exec` directly. | Low (by design — Root is not a security boundary) |

---

## 9. Detailed Gap Records

### Gap 1: No Lifecycle Model (Critical)
- **File:** System-wide (no dedicated lifecycle code exists)
- **Description:** There is no formal state machine for sandboxes. `SandboxInstance.status` is a raw `String` from Docker output (parsed only for "running" vs everything else). Root cannot track transitions between states, cannot detect sandbox crashes, and has no persistence layer for sandbox metadata outside Docker.
- **Recommendation:** Introduce a `SandboxState` enum (`Created`, `Running`, `Stopped`, `Error`, `Destroyed`), persist sandbox metadata to `~/.root/sandboxes.json` or similar, and query Docker only as an implementation detail of the provider.

### Gap 2: No Timeouts on Any Operation (High)
- **File:** `crates/root-sandbox/src/lib.rs` lines 49–62 (`run_docker`), line 112 (`run_command`)
- **Description:** `docker run`, `docker exec`, and all other Docker commands run with no timeout. A slow image pull, a hanging command, or a hung Docker daemon blocks Root indefinitely. There is no `std::process::Command` timeout mechanism.
- **Recommendation:** Add a configurable timeout (default e.g. 30s for exec, 120s for create) by spawning the command in a thread with a timeout or using a crate like `wait-timeout`.

### Gap 3: Weak Ownership Verification (Medium)
- **File:** `crates/root-sandbox/src/lib.rs` lines 162–167 (`destroy`)
- **Description:** Ownership check is a single string prefix match on `root-sandbox-`. Any container whose name starts with this prefix is considered Root-owned. There is no label-based verification, no cryptographic proof, no stored fingerprint.
- **Recommendation:** (a) Add Docker labels (`root-managed=true`, `root-version=0.2.1`, `root-sandbox-id=<uuid>`) at container creation time. (b) Verify labels on destroy. (c) Apply same check to `run_command`.

### Gap 4: Orphan Container Leak on Partial Create Failure (High)
- **File:** `crates/root-sandbox/src/lib.rs` lines 87–98 (`create`)
- **Description:** The `create` method runs `docker rm -f` (pre-clean), then `docker run -d`, then `docker inspect`. If `docker run` succeeds but `docker inspect` fails, the running container is leaked — it exists in Docker but Root never records its ID and cannot manage it.
- **Recommendation:** Use a scoped cleanup guard (a `Drop`-based guard struct) to remove the container if any post-creation step fails. If `create()` returns an error, the guard tears down the container.

### Gap 5: Missing `created_at` in `list` Output (Low)
- **File:** `crates/root-sandbox/src/lib.rs` lines 124–158
- **Description:** `list()` uses `docker ps --format '{{.ID}}\t{{.Names}}\t{{.Status}}\t{{.Image}}'`, which does not include a creation timestamp. `SandboxInstance.created_at` is always set to `String::new()`. This field is effectively useless.
- **Recommendation:** Add `{{.CreatedAt}}` to the format template and parse the timestamp. Alternatively, populate `created_at` from the provider's create return value (but `list()` doesn't have access to that — a registry/persistence layer would solve this).

### Gap 6: No Automatic Cleanup / Garbage Collection (High)
- **File:** System-wide
- **Description:** Sandboxes run `sleep infinity` and live until explicitly destroyed. There is no TTL mechanism, no periodic cleanup, no automatic pruning of exited containers, and no cleanup on Root uninstall. Long-running or abandoned sandboxes consume disk, memory, and process slots indefinitely.
- **Recommendation:** (a) Add a `--ttl` flag to `sandbox create` that auto-destroys after a duration. (b) Add a `root sandbox prune` command to remove stopped/exited containers. (c) Consider a background worker (optional) for periodic cleanup.

### Gap 7: `run_command` Skips Ownership Check (Medium)
- **File:** `crates/root-sandbox/src/lib.rs` lines 108–122
- **Description:** `run_command()` passes the `id` directly to `docker exec` with no verification that the container is Root-managed. Unlike `destroy()` which checks `root-sandbox-` prefix, `run_command()` executes in any container ID/name provided.
- **Recommendation:** Add the same `root-sandbox-` prefix check (or, better, label-based verification) to `run_command()` before executing. Return `NotRootOwned` if the container is not managed by Root.

### Gap 8: Event Recording Errors Are Silently Swallowed (Low)
- **File:** `crates/root-core/src/lib.rs` lines 2869, 2911, 2953
- **Description:** All sandbox event recordings use `let _ = record_event(...)`. If the events file cannot be written (disk full, permissions, filesystem error), the error is silently discarded. The user sees a successful operation even though audit data was lost.
- **Recommendation:** Log the recording failure (eprintln or warn) but do not fail the operation. At minimum, the user should be aware that event recording failed.

### Gap 9: Sandbox Events Lack Execution Metadata (Medium)
- **File:** `crates/root-core/src/lib.rs` lines 2869–2880, 2911–2922, 2953–2961
- **Description:** Sandbox events use the generic `record_event()` function which only captures `event_type`, `status`, `command`, `package`, `snapshot_id`, `restored_snapshot_id`, and `message`. Execution events via `record_execution_event()` additionally capture `task_name`, `exit_code`, `started_at`, `finished_at`, and `duration_ms`. Sandbox events lack all of these fields.
- **Recommendation:** Create a dedicated sandbox event recording path (analogous to `record_execution_event()`) that captures timing, exit code, and sandbox metadata.

### Gap 10: No Sandbox-Nix Ecosystem Unification (Low)
- **File:** System-wide architectural gap
- **Description:** Sandbox is a completely separate subsystem from the Nix-based package management. There is no way to install a Root-managed package *into* a sandbox. A user cannot run `root sandbox create build-env && root sandbox run build-env -- root install ffmpeg` because the `root` CLI inside the container is not available.
- **Recommendation:** This is a design choice for now, but it should be documented as a known limitation. Consider adding a `--nix` flag to sandbox create that pre-installs Nix inside the container, or a `sandbox provision` command.

### Gap 11: No Interactive / TTY Mode for `sandbox run` (Low)
- **File:** `crates/root-cli/src/main.rs` lines 1000–1029, `crates/root-sandbox/src/lib.rs` lines 108–122
- **Description:** `sandbox run` uses `docker exec` without `-it` flags. Commands that require a TTY (interactive shells, `top`, etc.) will fail or behave differently. There is no `--interactive` flag on the CLI.
- **Recommendation:** Add a `--interactive` / `-i` flag to `root sandbox run` that passes `-it` to `docker exec`.

### Gap 12: Unpinned / Floating Docker Image Tags (Medium)
- **File:** `crates/root-sandbox/src/lib.rs` line 80
- **Description:** The default image is `ubuntu:latest` (a floating tag that changes over time). The `--image` flag accepts any Docker image reference with no validation, no digest pinning, and no checksum verification. This means sandbox environments are non-deterministic — two `root sandbox create` calls at different times can produce different base systems.
- **Recommendation:** (a) Warn when a floating tag is used. (b) Optionally resolve the tag to a digest at creation time and store it. (c) Consider Root maintaining a pinned default image with a known digest.

---

## 10. Test Coverage Analysis

### Existing Tests (`crates/root-sandbox/src/lib.rs` lines 266–355)

| Test | What It Covers | What It Misses |
|------|----------------|----------------|
| `test_mock_availability` | Mock returns correct availability | No test for real provider |
| `test_mock_create_list_destroy` | Full create/list/destroy cycle | No failure injection |
| `test_mock_run_command` | Run command in mock | No error cases (bad command, missing container) |
| `test_mock_destroy_not_found` | Destroy non-existent sandbox | — |
| `test_mock_destroy_root_owned_container` | Destroy by name works | — |
| `test_mock_destroy_rejects_non_root_container` | Rejects non-Root-owned | — |
| `test_mock_destroy_by_id_root_owned` | Destroy by ID works | — |
| `test_mock_unavailable_errors` | All ops fail when unavailable | — |

### Missing Tests

| Test Needed | Location | Reason |
|-------------|----------|--------|
| `sandbox_create` policy enforcement | `root-core` | Orchestration untested |
| `sandbox_run` policy enforcement | `root-core` | Orchestration untested |
| `sandbox_destroy` policy enforcement | `root-core` | Orchestration untested |
| `sandbox_create` event recording | `root-core` | Event side effects untested |
| `sandbox_run` exit code & event propagation | `root-core` | Status mapping untested |
| `RealSandboxProvider` integration tests | `root-sandbox` | Real Docker not exercised in CI |
| `sandbox_list` with no containers | `root-core` | Edge case |
| Concurrent sandbox operations | `root-core` | Race conditions (no mutex on sandbox path) |

---

## 11. Recommendations by Priority

### Immediate (Next Release)

1. **[Gap 2]** Add timeouts to `run_docker()` and `run_command()` — prevents indefinite hangs.
2. **[Gap 7]** Add ownership check to `run_command()` — closes a security hole.
3. **[Gap 8]** Log event recording failures instead of swallowing — improves observability.

### Short-Term (Next Two Releases)

4. **[Gap 4]** Add cleanup guard to `create()` for partial failures — prevents resource leaks.
5. **[Gap 5]** Include `{{.CreatedAt}}` in `docker ps` format — fixes data quality.
6. **[Gap 9]** Add execution metadata fields to sandbox events — improves audit quality.
7. **[Gap 3]** Add Docker labels for ownership verification — strengthens security model.

### Medium-Term

8. **[Gap 1]** Design and implement a formal sandbox lifecycle with state persistence.
9. **[Gap 6]** Add TTL support, `prune` command, and automatic cleanup.
10. **[Gap 12]** Pin default image by digest; warn on floating tags.

### Long-Term

11. **[Gap 10]** Explore Nix-in-sandbox unification.
12. **[Gap 11]** Add interactive / TTY support for `sandbox run`.

---

## 12. Appendix: Code Reference Map

| Symbol | File | Line |
|--------|------|------|
| `SandboxProvider` trait | `crates/root-sandbox/src/lib.rs` | 34–40 |
| `RealSandboxProvider` struct | `crates/root-sandbox/src/lib.rs` | 42–43 |
| `RealSandboxProvider::create()` | `crates/root-sandbox/src/lib.rs` | 79–106 |
| `RealSandboxProvider::run_command()` | `crates/root-sandbox/src/lib.rs` | 108–122 |
| `RealSandboxProvider::list()` | `crates/root-sandbox/src/lib.rs` | 124–158 |
| `RealSandboxProvider::destroy()` | `crates/root-sandbox/src/lib.rs` | 160–170 |
| `RealSandboxProvider::run_docker()` | `crates/root-sandbox/src/lib.rs` | 49–62 |
| `MockSandboxProvider` struct | `crates/root-sandbox/src/lib.rs` | 173–176 |
| `SandboxError` enum | `crates/root-sandbox/src/lib.rs` | 6–16 |
| `SandboxInstance` struct | `crates/root-sandbox/src/lib.rs` | 18–25 |
| `SandboxExecResult` struct | `crates/root-sandbox/src/lib.rs` | 27–32 |
| `SandboxCreateReport` struct | `crates/root-core/src/lib.rs` | 2255–2262 |
| `SandboxRunReport` struct | `crates/root-core/src/lib.rs` | 2265–2272 |
| `SandboxListReport` struct | `crates/root-core/src/lib.rs` | 2274–2278 |
| `SandboxDestroyReport` struct | `crates/root-core/src/lib.rs` | 2281–2284 |
| `sandbox_create()` | `crates/root-core/src/lib.rs` | 2846–2890 |
| `sandbox_run()` | `crates/root-core/src/lib.rs` | 2892–2932 |
| `sandbox_list()` | `crates/root-core/src/lib.rs` | 2934–2943 |
| `sandbox_destroy()` | `crates/root-core/src/lib.rs` | 2945–2967 |
| `SandboxPolicy` struct | `crates/root-core/src/policy.rs` | 73–90 |
| `PolicyAction::SandboxCreate` | `crates/root-core/src/policy.rs` | 167 |
| `PolicyAction::SandboxRun` | `crates/root-core/src/policy.rs` | 168 |
| `PolicyAction::SandboxDestroy` | `crates/root-core/src/policy.rs` | 169 |
| `enforce_policy()` | `crates/root-core/src/lib.rs` | 830–855 |
| `SandboxSubcommands` enum | `crates/root-cli/src/main.rs` | 142–168 |
| `sandbox_provider` instantiation | `crates/root-cli/src/main.rs` | 317 |
| `handle_structured()` | `crates/root-cli/src/main.rs` | 282–306 |
| `exit_code_for_error()` | `crates/root-cli/src/main.rs` | 194–234 |
| `RootEventType::Sandbox` | `crates/root-core/src/events.rs` | 20 |
| `record_event()` | `crates/root-core/src/events.rs` | 213–233 |
| `record_execution_event()` | `crates/root-core/src/events.rs` | 170–191 |
