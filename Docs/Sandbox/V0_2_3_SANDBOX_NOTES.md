# Root v0.2.3 Sandbox Notes

Reference document for Root's Docker-backed sandbox subsystem: lifecycle,
cleanup guarantees, resource limits, timeout handling, failure recovery,
Docker requirements, event recording, validation, and error messages.

---

## Table of Contents

1. [Lifecycle Model](#1-lifecycle-model)
2. [Cleanup Guarantees](#2-cleanup-guarantees)
3. [Resource Limits](#3-resource-limits)
4. [Timeout Behavior](#4-timeout-behavior)
5. [Failure Recovery](#5-failure-recovery)
6. [Docker Requirements](#6-docker-requirements)
7. [Event Recording](#7-event-recording)
8. [Validation](#8-validation)
9. [Error Messages](#9-error-messages)
10. [Implementation Reference](#10-implementation-reference)

---

## 1. Lifecycle Model

Sandboxes follow a strict state machine defined by the `SandboxState` enum.
Each state transition is validated before execution. Invalid transitions are
rejected with a `LifecycleViolation` error.

### States

| State       | Meaning                                                    |
|-------------|------------------------------------------------------------|
| `Created`   | Sandbox instance has been created but no command has run.  |
| `Running`   | A command is currently executing (or has been executed).   |
| `Completed` | The sandbox ran to completion (container exited normally). |
| `Failed`    | The sandbox command failed (non-zero exit or error).       |
| `Destroyed` | The sandbox has been destroyed and is no longer usable.    |

### Valid Transitions

```
Created ──► Running
Created ──► Completed
Created ──► Failed
Created ──► Destroyed

Running ──► Completed
Running ──► Failed
Running ──► Running    (self-transition is allowed)

Completed ──► Destroyed

Failed ──► Destroyed
```

### Invalid Transitions (Examples)

| Attempted Transition          | Error                                                  |
|-------------------------------|--------------------------------------------------------|
| Destroyed → Running           | `LifecycleViolation("Invalid state transition: Destroyed -> Running")` |
| Destroyed → Failed            | `LifecycleViolation("Invalid state transition: Destroyed -> Failed")` |
| Destroyed → Completed         | `LifecycleViolation("Invalid state transition: Destroyed -> Completed")` |
| Destroyed → Destroyed (repeat)| `LifecycleViolation("Sandbox '...' is already destroyed")` (Real provider) or `NotFound` (Mock provider) |

> **Note:** The `can_transition_to` method in `SandboxState` explicitly returns
> `false` for all transitions departing from `Destroyed` (except self-transition
> for `Destroyed` itself in the real provider, but destroy is idempotent at the
> API level).

### Enforcement Points

- **`MockSandboxProvider`** — `validate_transition` is called before every state
  change in `run_command` and `destroy`. Returns `LifecycleViolation` for
  invalid transitions.
- **`RealSandboxProvider`** — `run_command` checks the in-memory state map and
  rejects commands on destroyed sandboxes. `destroy` checks for already-destroyed
  state and returns a `LifecycleViolation` if the container is already gone.
- **CLI (`root sandbox run`)** — passes through the provider's error to the user.

---

## 2. Cleanup Guarantees

Root guarantees that cleanup is attempted in every scenario where a sandbox
can leave an orphaned container. The cleanup mechanisms differ between the
mock provider (which tracks state in-memory) and the real provider (which
manages Docker containers).

### On Destroy (Explicit)

- **Real provider**: `docker rm -f <id>` is called. If the container is not
  Root-owned (name does not start with `root-sandbox-`), the operation is
  rejected with `NotRootOwned`. If the container does not exist, returns
  `NotFound`.
- **Mock provider**: The sandbox is removed from the in-memory list and its
  state is set to `Destroyed`. A cleanup counter is incremented.
- **Both**: State is set to `Destroyed` regardless of cleanup outcome.

### On Failed Run

- **Real provider**: When a run command returns a non-zero exit code (and
  the exit code is not 124/timeout), `provider.destroy(id)` is called from
  `root-core::sandbox_run`.
- **Mock provider**: Destroy is called, which sets state to `Destroyed` and
  removes the sandbox from the list.
- A cleanup event is recorded in the event ledger.

### On Timeout

- **Behavior is identical to failed run**: `provider.destroy(id)` is called
  from `root-core::sandbox_run` when `result.exit_code == 124`.
- A cleanup event with the message "Cleanup attempted after timeout for sandbox
  '...'" is recorded.

### On Validation Failure (Post-Create)

- If a sandbox is created but post-create validation (exists + reachable) fails,
  `provider.destroy(&instance.id)` is called before returning the error.
- The user receives an error explaining that the sandbox was destroyed due to
  validation failure.

### On Stale Detection

- **`root sandbox list`** queries `docker ps -a --filter name=root-sandbox-` and
  reports all matching containers, including stopped ones. This allows users to
  discover containers that were left behind (e.g., due to a crash).
- The mock provider tracks all created sandboxes in an in-memory list that can
  be inspected via `list()`.

### Cleanup Guarantee Summary

| Scenario                                    | Cleanup Attempted | Recorded in Events |
|---------------------------------------------|-------------------|--------------------|
| Explicit `root sandbox destroy <id>`        | Yes               | Yes                |
| Run command fails (non-zero exit)           | Yes               | Yes (cleanup + run) |
| Run command times out (exit 124)            | Yes               | Yes (cleanup + timeout) |
| Post-create validation fails                | Yes               | No (not persisted) |
| Provider unavailable during destroy         | Depends*          | Yes (failure event) |
| Root crashes mid-operation                  | No (manual)       | No                 |

> \* The real provider attempts `docker rm -f <id>` as a final fallback even
> after a failed destroy attempt.

---

## 3. Resource Limits

Sandbox containers can be created with configurable memory and CPU limits.
These are passed through to Docker's `--memory` and `--cpus` flags.

### Default Values

| Resource | Default | Flag               |
|----------|---------|--------------------|
| Memory   | `2g`    | `--memory <value>` |
| CPUs     | `2.0`   | `--cpus <value>`   |

### CLI Usage

```bash
# Create with custom limits
root sandbox create my-sandbox --memory 4g --cpus 4.0

# Create with defaults
root sandbox create my-sandbox

# Create with partial overrides
root sandbox create my-sandbox --memory 512m
root sandbox create my-sandbox --cpus 1
```

### How Limits Are Applied

1. The `SandboxProvider::create()` trait method accepts `memory: Option<&str>`
   and `cpus: Option<&str>`.
2. `RealSandboxProvider::create()` passes these to Docker:
   ```
   docker run -d --name root-sandbox-<name> --memory <mem> --cpus <cpu> <image> sleep infinity
   ```
3. If `memory` is `None`, the default `"2g"` is used. If `cpus` is `None`, the
   default `"2.0"` is used.
4. The `SandboxInstance` struct stores `memory` and `cpus` as `Option<String>`
   and serializes them into JSON output (`root sandbox list --json`).

### Docker Resource Flag Reference

| Flag        | Accepts               | Example        |
|-------------|-----------------------|----------------|
| `--memory`  | Number + unit (b, k, m, g) | `512m`, `2g` |
| `--cpus`    | Decimal number        | `1.0`, `2.5`  |

### Resource Limit Errors

If Docker rejects the resource limits (e.g., too much memory requested, invalid
format, or OOM occurs), the error is normalized into `ResourceLimitExceeded`:

```
Error: Resource limit exceeded: <docker error message>
```

---

## 4. Timeout Behavior

Every sandbox run command has a configurable timeout that controls how long the
command is allowed to execute before being killed.

### Default

- **Default timeout**: 300 seconds (5 minutes)
- Applied when no `--timeout` flag is provided.

### How It Works

1. The `root sandbox run` command accepts `--timeout <seconds>`.
2. In `RealSandboxProvider::run_command()`:
   - If `timeout > 0`, the Docker exec wraps the command with `timeout <N>`:
     ```
     docker exec <id> timeout <N> <command...>
     ```
   - If `timeout` is `None`, the default `300` is used.
   - If `timeout` is `Some(0)`, no timeout is applied (command runs without
     Docker's `timeout` wrapper).
3. Docker's `timeout` utility sends SIGTERM after the specified seconds, then
   SIGKILL if the process does not stop.
4. The process exit code is captured. If the exit code is `124` (the standard
   timeout exit code), the sandbox run report marks `timed_out: true`.

### What Happens on Timeout

1. Command is killed by Docker's `timeout` utility.
2. Exit code 124 is returned.
3. `root-core::sandbox_run` detects `exit_code == 124`.
4. A cleanup is triggered: `provider.destroy(id)` is called.
5. Two events are recorded:
   - **Timeout event**: status `Timeout` with message "Command timed out in
     sandbox '...' after <N>ms"
   - **Cleanup event**: status `Completed` with message "Cleanup attempted after
     timeout for sandbox '...'"
6. The `SandboxRunReport` includes:
   - `timed_out: true`
   - `cleanup_attempted: true`
   - `exit_code: 124`
   - `stderr` includes "Command timed out after <N> seconds"

### Output

```bash
$ root sandbox run my-sandbox --timeout 5 -- sleep 30
Error: Command timed out in sandbox 'my-sandbox' after 5003ms
```

### Mock Provider Simulation

The mock provider has a `simulate_timeout` flag. When set to `true`,
`run_command` returns `SandboxExecResult` with `exit_code: 124` and
`stderr: "Command timed out"`.

---

## 5. Failure Recovery

### When a Sandbox Is Stuck

A sandbox may become stuck or unusable in the following scenarios:

| Scenario                               | Symptoms                                   |
|----------------------------------------|--------------------------------------------|
| Docker daemon crash                    | `docker exec` fails; sandbox unreachable   |
| Container entered bad state            | `docker exec` hangs or returns errors      |
| Root crashed mid-operation             | Container may be orphaned                  |
| Resource exhaustion (OOM)              | Container killed by Docker                 |

### Recovery Steps

1. **Detect stale sandboxes**:
   ```bash
   root sandbox list
   ```
   This shows all Root-managed containers (both running and stopped).

2. **Inspect a specific sandbox**:
   ```bash
   docker inspect root-sandbox-<name>
   ```

3. **Destroy a stuck sandbox**:
   ```bash
   root sandbox destroy <id-or-name>
   ```
   If the CLI fails, use Docker directly:
   ```bash
   docker rm -f root-sandbox-<name>
   ```

4. **Force cleanup all Root sandboxes**:
   ```bash
   docker rm -f $(docker ps -aq --filter name=root-sandbox-) 2>/dev/null || true
   ```

### Detecting Stale Sandboxes

- `root sandbox list --json` outputs all known sandboxes with their state.
  Sandboxes in `Completed`, `Failed`, or unknown states that are no longer
  tracked by Root's in-memory state may be stale.
- The real provider lists all Docker containers matching the `root-sandbox-`
  prefix, so any orphaned container is discoverable.
- There is no background daemon or periodic stale-checker. Detection is
  on-demand via `list`.

### Preventing Stale Sandboxes

- All failure paths in `root-core::sandbox_run` (non-zero exit, timeout)
  trigger automatic cleanup (destroy).
- Post-create validation failure destroys the container immediately.
- Destroy always attempts `docker rm -f` — even on error, an additional
  `docker rm -f` is attempted.

---

## 6. Docker Requirements

Root uses Docker to create and manage sandbox containers. It shells out to the
`docker` CLI binary.

### Requirements

| Requirement        | Detail                                                    |
|--------------------|-----------------------------------------------------------|
| Docker CLI         | `docker` must be on `PATH`                                |
| Docker Daemon      | Must be running (`docker info` must succeed)              |
| Platform           | macOS (Apple Silicon + Intel) and Linux                   |
| Min. Docker CLI    | No specific minimum — any version with `run`, `exec`, `ps`, `rm`, `inspect` |
| Image              | Default `ubuntu:latest` (pulled from Docker Hub if absent)|

### How Availability Is Checked

`RealSandboxProvider::check_availability()` runs:
```bash
docker info
```
Returns `true` if the command exits with status 0, `false` otherwise.

### What Happens When Docker Is Unavailable

If `check_availability()` returns `false`, the CLI returns:

```
Error: No sandbox provider is available.

Root requires Docker to create sandboxes.
Install Docker Desktop from https://docker.com
Then verify with: docker info
```

Exit code: 1

### Platform Notes

- **macOS (Apple Silicon)**: Docker Desktop for Apple Silicon is required.
  Ensure Rosetta 2 is installed if running x86_64 images.
- **macOS (Intel)**: Docker Desktop for Mac (Intel) works with default settings.
- **Linux**: Docker Engine (CE) is sufficient. `docker` must be on `PATH` and
  the user must have permissions (typically via the `docker` group or `sudo`).

### Container Tagging Convention

All Root-managed containers are created with the naming convention:
```
root-sandbox-<name>
```

This prefix is used to:
- Filter containers in `root sandbox list` (`--filter name=root-sandbox-`)
- Validate ownership in `root sandbox destroy` (rejects non-`root-sandbox-` containers)
- Identify orphaned containers during manual cleanup

---

## 7. Event Recording

Every sandbox operation is recorded in the event ledger at
`~/.root/events.jsonl`. The event schema is defined in `root-core::events`.

### Event Fields

| Field              | Type     | Description                                    |
|--------------------|----------|------------------------------------------------|
| `event_type`       | String   | Always `"Sandbox"` for sandbox operations      |
| `status`           | String   | `Completed`, `Failed`, or `Timeout`           |
| `command`          | String   | CLI command that triggered the event           |
| `timestamp`        | String   | ISO 8601 / RFC 3339 timestamp                  |
| `sandbox_id`       | String   | The sandbox ID (container ID or mock ID)       |
| `exit_code`        | Integer  | Exit code of the sandbox command (run only)    |
| `details`          | String   | Human-readable description                     |
| `started_at`       | String   | ISO 8601 start time (run only)                 |
| `finished_at`      | String   | ISO 8601 finish time (run only)                |
| `duration_ms`      | Integer  | Duration in milliseconds (run only)            |

### Events Recorded Per Operation

| CLI Command                          | Event Type | Status     | Details                                |
|--------------------------------------|------------|------------|----------------------------------------|
| `root sandbox create`                | Sandbox    | Completed  | "Created sandbox 'root-sandbox-...' (id: ...)" |
| `root sandbox run` (success)        | Sandbox    | Completed  | "Executed in sandbox '...': exit code 0" |
| `root sandbox run` (failure)        | Sandbox    | Failed     | "Executed in sandbox '...': exit code <N>" |
| `root sandbox run` (timeout)        | Sandbox    | Timeout    | "Command timed out in sandbox '...' after <N>ms" |
| `root sandbox run` → cleanup        | Sandbox    | Completed  | "Cleanup attempted after timeout for sandbox '...'" |
| `root sandbox run` → cleanup (fail) | Sandbox    | Completed  | "Cleanup attempted after failed run for sandbox '...'" |
| `root sandbox destroy` (success)    | Sandbox    | Completed  | "Destroyed sandbox '...'" |
| `root sandbox destroy` (failure)    | Sandbox    | Failed     | "Sandbox destroy failed: <error>" |
| `root sandbox destroy` (post-check warning) | Sandbox | Failed | "Sandbox '...' may still exist after destroy attempt" |

### Viewing Events

```bash
# View all events (human-readable)
root history

# View last N events
root history --limit 10

# View events as JSON
root history --json
```

Events are appended to `events.jsonl` in append-only mode. Each line is a
JSON object. Malformed lines are gracefully skipped during read (see v0.2.1).

### Event Flow for a Typical Sandbox Lifecycle

```
create ─► event(Completed, "Created sandbox 'root-sandbox-demo' (id: abc123)")
run    ─► event(Completed, "Executed in sandbox 'abc123': exit code 0")
destroy ─► event(Completed, "Destroyed sandbox 'abc123'")
```

### Event Flow for a Timeout

```
create ─► event(Completed, "Created sandbox 'root-sandbox-demo' (id: abc123)")
run    ─► event(Timeout, "Command timed out in sandbox 'abc123' after 5003ms")
       ─► event(Completed, "Cleanup attempted after timeout for sandbox 'abc123'")
```

---

## 8. Validation

Root performs validation at two points in the sandbox lifecycle: immediately
after creation and immediately after destruction.

### Post-Create Validation

After `provider.create()` returns successfully, `root-core::sandbox_create`
runs two checks:

1. **`provider.check_exists(id)`** — Verifies the container exists in Docker.
   - Real provider: runs `docker inspect --format '{{.Id}}' <id>`
   - Returns `true` if the inspect succeeds.
2. **`provider.check_reachable(id)`** — Verifies the container is reachable.
   - Real provider: runs `docker exec <id> echo reachable`
   - Returns `true` if the exec succeeds.

If either check fails, the sandbox is immediately destroyed and an error is
returned:

```
Error: Sandbox '<name>' was created but post-create validation failed
(exists: false, reachable: true). The sandbox has been destroyed.
```

### Post-Destroy Validation

After `provider.destroy()` completes (success or failure),
`root-core::sandbox_destroy` runs:

1. **`provider.check_exists(id)`** — Checks if the container still exists.
   - If `true`: a warning event is recorded ("Sandbox '...' may still exist
     after destroy attempt") and merged into the error message if destroy
     also failed.

### Validation Method Signatures

```rust
fn check_exists(&self, id: &str) -> Result<bool, SandboxError>;
fn check_reachable(&self, id: &str) -> Result<bool, SandboxError>;
```

### Mock Provider Validation Behavior

- `check_exists`: Returns `true` if the sandbox is tracked and not destroyed.
- `check_reachable`: Returns `true` if the sandbox is in `Running` or `Created`
  state.

---

## 9. Error Messages

All sandbox errors are defined in the `SandboxError` enum and normalized into
user-friendly messages before reaching the CLI.

### Error Catalog

| Error Variant            | Trigger                                     | User-Facing Message Pattern               | Exit Code |
|--------------------------|---------------------------------------------|-------------------------------------------|-----------|
| `NotAvailable`           | Docker not on PATH / daemon not running     | "No sandbox provider is available. ..."   | 1         |
| `NotFound`               | Sandbox ID does not exist                   | "Sandbox '<id>' not found"                | 3         |
| `NotRootOwned`           | Destroy attempted on non-Root container     | "Container '<id>' is not a Root-managed sandbox" | 1 |
| `DockerUnavailable`      | `docker` binary not found                   | "Docker is not available on PATH. Install Docker Desktop or the Docker CLI." | 7 |
| `ImagePullFailed`        | Docker image cannot be pulled/found         | "Failed to pull/start image '<image>': <details>" | 1 |
| `ContainerStartupFailed` | Container fails to start                    | "Container failed to start: <details>"    | 1 |
| `TimeoutExceeded`        | Command exceeded timeout                    | "Sandbox command timed out after <N> seconds" | 1 |
| `ResourceLimitExceeded`  | OOM or invalid resource spec                | "Resource limit exceeded: <details>"      | 1 |
| `PermissionDenied`       | Docker permission error                     | "Permission denied: <details>"            | 1 |
| `CleanupFailed`          | Docker `rm -f` fails                        | "Cleanup failed: <details>"               | 1 |
| `LifecycleViolation`     | Invalid state transition attempted          | "Invalid state transition: <current> -> <target> for sandbox '<id>'" | 6 |
| `Generic`                | Unclassified Docker error                   | "Sandbox operation failed: <details>"     | 1 |

### Error Normalization Flow

Docker stderr is normalized by `normalize_docker_error()` in
`RealSandboxProvider`:

```
Docker stderr
    │
    ▼
normalize_docker_error(stderr)
    │
    ├── "permission denied" or "permission_denied" ──► PermissionDenied
    ├── "image" + ("pull" or "not found")            ──► ImagePullFailed
    ├── "oom" or "memory" or "cpuset"                 ──► ResourceLimitExceeded
    └── otherwise                                      ──► Generic
```

Additional normalization in `RealSandboxProvider::create()` converts
`Generic` errors containing "image" or "pull" into `ImagePullFailed`,
and errors containing "cannot start" or "startup" into
`ContainerStartupFailed`.

### Error Examples

```bash
# Docker not available
$ root sandbox create
Error: No sandbox provider is available.

Root requires Docker to create sandboxes.
Install Docker Desktop from https://docker.com
Then verify with: docker info

# Sandbox not found
$ root sandbox destroy nonexistent
Error: Sandbox destroy failed: Sandbox 'nonexistent' not found

# Invalid lifecycle (run destroyed sandbox)
$ root sandbox run destroyed-sb -- echo hi
Error: Sandbox exec failed: Invalid state transition: Destroyed -> Running for sandbox 'abc123'

# Timeout
$ root sandbox run my-sb --timeout 3 -- sleep 30
Error: Command timed out in sandbox 'my-sb' after 3002ms

# Not a Root-managed container
$ root sandbox destroy my-own-container
Error: Sandbox destroy failed: Container 'my-own-container' is not a Root-managed sandbox
```

---

## 10. Implementation Reference

### Key Files

| File | Role |
|------|------|
| `crates/root-sandbox/src/lib.rs` | `SandboxState`, `SandboxError`, `SandboxInstance`, `SandboxProvider` trait, `RealSandboxProvider`, `MockSandboxProvider`, unit tests |
| `crates/root-core/src/lib.rs` (lines 2857–3067) | `sandbox_create`, `sandbox_run`, `sandbox_list`, `sandbox_destroy` orchestration |
| `crates/root-core/src/events.rs` | Event recording for sandbox operations |
| `crates/root-core/src/policy.rs` | Policy enforcement for sandbox actions |
| `crates/root-cli/src/main.rs` (lines 996–1081) | CLI `sandbox` subcommand parsing and dispatch |

### Trait Reference

```rust
pub trait SandboxProvider {
    fn check_availability(&self) -> Result<bool, SandboxError>;
    fn create(&self, name: &str, image: Option<&str>,
              memory: Option<&str>, cpus: Option<&str>)
              -> Result<SandboxInstance, SandboxError>;
    fn run_command(&self, id: &str, command: &[&str],
                   timeout_secs: Option<u64>)
                   -> Result<SandboxExecResult, SandboxError>;
    fn list(&self) -> Result<Vec<SandboxInstance>, SandboxError>;
    fn destroy(&self, id: &str) -> Result<(), SandboxError>;
    fn check_exists(&self, id: &str) -> Result<bool, SandboxError>;
    fn check_reachable(&self, id: &str) -> Result<bool, SandboxError>;
}
```

### Implementations

- **`RealSandboxProvider`**: Shells out to the `docker` CLI. Maintains an
  in-memory `HashMap<String, SandboxState>` to track lifecycle state.
- **`MockSandboxProvider`**: In-memory provider used for unit tests.
  Supports simulation flags (`simulate_timeout`, `simulate_cleanup_failure`,
  `simulate_destroy_unavailable`).

### Related Documents

- `Docs/Release/V0_2_3_SANDBOX_SMOKE_TEST.md` — Manual smoke tests for v0.2.3
  sandbox subsystem
- `Docs/Core/` — Full design specification chain (PRD, TECH_SPEC, ARCHITECTURE,
  UX_FLOWS, IMPLEMENTATION_PLAN)
