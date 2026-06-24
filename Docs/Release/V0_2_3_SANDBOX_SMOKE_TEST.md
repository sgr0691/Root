# Root v0.2.3 Sandbox Smoke Test

Manual release validation focused on the new sandbox system: create, run,
destroy lifecycle; named sandboxes; custom images; error paths; timeout;
event ledger; Docker-unavailable handling; and JSON output.

Run the full automated CI sequence first, then execute these checks with
Docker running and a disposable Root directory.

---

## Automated Gates

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo build
target/debug/root --version
```

**Expected:** every command succeeds and the binary reports `root 0.2.3`.

---

## Prerequisites

- macOS (Apple Silicon or Intel) or Linux
- Docker installed and running (`docker info` succeeds)
- Internet access (Docker Hub pull access for `ubuntu:latest` and `alpine:latest`)
- No existing `~/.root` directory (or back it up before these tests)
- `root` binary built from the v0.2.3 tag

---

## 1. Fresh Sandbox Create, Run, Destroy

Basic lifecycle: create a sandbox (default name), run a command inside it,
then destroy it.

```bash
root sandbox create
```

**Expected:**
- Sandbox created with name `root-sandbox-default`.
- Output shows created sandbox name, id, image (`ubuntu:latest`), and status (`running`).
- Exit code 0.

```bash
root sandbox run <id> -- echo hello
```

**Expected:**
- Command executes inside the container.
- `hello` printed to stdout.
- Exit code 0.

```bash
root sandbox destroy <id>
```

**Expected:**
- Container stopped and removed.
- Output: `Destroyed sandbox '<id>'.`
- Exit code 0.

```bash
root history --limit 5
```

**Expected:**
- Three sandbox events recorded: create, run, destroy.
- Each event has type `Sandbox`, a timestamp, and a status (`Completed`).
- Run event includes the command and exit code.

---

## 2. Named Sandbox with Image

Create a sandbox with an explicit name and a custom Docker image, run a
command, then destroy.

```bash
root sandbox create test-box --image alpine:latest
```

**Expected:**
- Sandbox created with name `root-sandbox-test-box`.
- Image is `alpine:latest`.
- Status is `running`.
- Exit code 0.

```bash
root sandbox run test-box -- echo hello from alpine
```

**Expected:**
- `hello from alpine` printed to stdout.
- Exit code 0.

```bash
root sandbox destroy test-box
```

**Expected:**
- Container removed.
- Output: `Destroyed sandbox 'test-box'.`
- Exit code 0.

---

## 3. Resource Limits

Create a sandbox with explicit memory and CPU limits.

```bash
root sandbox create --memory 1g --cpus 1
```

**Expected:**
- Sandbox created with the specified resource constraints.
- Docker `inspect` confirms `--memory` and `--cpus` are applied to the container.
- Status is `running`.
- Exit code 0.

**Cleanup:**

```bash
root sandbox destroy default
```

---

## 4. Timeout

Create a sandbox and run a command with a short timeout; the long-running
command should be killed after the timeout expires.

```bash
root sandbox create timeout-box
```

**Expected:**
- Sandbox created and running.
- Exit code 0.

```bash
root sandbox run timeout-box --timeout 5 -- sleep 30
```

**Expected:**
- Command is killed after approximately 5 seconds.
- Exit code 124 (standard timeout exit code).
- Output includes a timeout message (e.g., "Command timed out").
- Timeout event recorded in history with status `Failed` or `TimedOut`.

**Cleanup:**

```bash
root sandbox destroy timeout-box
```

**Expected:**
- Container removed.
- Exit code 0.

---

## 5. Invalid Command

Run a command that does not exist inside a valid sandbox.

```bash
root sandbox create
```

**Expected:**
- Default sandbox created.
- Exit code 0.

```bash
root sandbox run default -- nonexistent-command
```

**Expected:**
- Exit code non-zero (typically 127 -- command not found).
- Failure event recorded in history with status `Failed`.
- Sandbox is **not** destroyed (still listed in `root sandbox list`).

**Cleanup:**

```bash
root sandbox destroy default
```

**Expected:**
- Container removed.
- Exit code 0.

---

## 6. Destroy Non-Existent Sandbox

Attempt to destroy a sandbox that was never created.

```bash
root sandbox destroy nonexistent-sandbox
```

**Expected:**
- Clear error: sandbox not found.
- Error message mentions the id `nonexistent-sandbox`.
- Exit code non-zero (1).
- No Docker containers affected.

---

## 7. Repeated Destroy

Destroy a sandbox twice. The first call succeeds; the second fails because
the sandbox is already gone.

```bash
root sandbox create repeat-box
```

**Expected:**
- Sandbox created.
- Exit code 0.

```bash
root sandbox destroy repeat-box
```

**Expected:**
- First destroy succeeds.
- Output: `Destroyed sandbox 'repeat-box'.`
- Exit code 0.

```bash
root sandbox destroy repeat-box
```

**Expected:**
- Second destroy fails with `"not found"` or similar error.
- Error message clearly indicates the sandbox no longer exists.
- Exit code non-zero (1).

---

## 8. Run Destroyed Sandbox

Create a sandbox, destroy it, then attempt to run a command in it. The run
must fail with an invalid lifecycle transition error.

```bash
root sandbox create run-destroy
```

**Expected:**
- Sandbox created.
- Exit code 0.

```bash
root sandbox destroy run-destroy
```

**Expected:**
- Container removed.
- Exit code 0.

```bash
root sandbox run run-destroy -- echo hello
```

**Expected:**
- Clear error: sandbox not found or container not running.
- Error message explains the sandbox does not exist.
- Exit code non-zero (1).

---

## 9. List Sandboxes

Create multiple sandboxes, verify they appear in `list`, then destroy them
and verify the list is empty.

```bash
root sandbox create list-test-1
root sandbox create list-test-2
```

**Expected:**
- Both sandboxes created successfully.
- Exit code 0 for each.

```bash
root sandbox list
```

**Expected:**
- Lists 2 sandboxes: `root-sandbox-list-test-1` and `root-sandbox-list-test-2`.
- Each entry shows name, id, status (`running`), and image (`ubuntu:latest`).
- Exit code 0.

```bash
root sandbox destroy list-test-1
root sandbox destroy list-test-2
```

**Expected:**
- Both destroyed successfully.
- Exit code 0 for each.

```bash
root sandbox list
```

**Expected:**
- Message: `No Root-managed sandboxes.`
- Exit code 0.

---

## 10. Event Ledger for Sandbox Operations

Run a sequence of sandbox operations and verify they are all recorded in the
event ledger with correct timestamps and results.

```bash
# Run a representative sequence
root sandbox create ledger-test
root sandbox run ledger-test -- echo event-ledger-check
root sandbox destroy ledger-test

# Now inspect the ledger
root history
```

**Expected (human-readable):**
- Three sandbox events listed: create, run, destroy.
- Each event has:
  - Type: `Sandbox`
  - Timestamp (ISO 8601 or RFC 3339)
  - Status: `Completed` (or `Failed` for error cases in other tests)
  - A detail message (e.g., "Created sandbox 'root-sandbox-ledger-test' (id: ...)")

```bash
root history --json --limit 10
```

**Expected (JSON):**
- Valid JSON array (or object wrapping an array).
- Each entry has `event_type`, `timestamp`, `status`, `details`.
- Sandbox events are interleaved with any other operation events in
  chronological order.
- No missing or duplicate events.

---

## 11. Docker Unavailable

Simulate the condition where Docker is not running or not on PATH, then
verify the error message is clear and actionable.

**Setup:** Temporarily disable Docker (e.g., quit Docker Desktop, or rename
the `docker` binary, or set `PATH` to exclude it):

```bash
# Option A: quit Docker Desktop and verify
# Option B: run in a subshell with modified PATH
PATH=/usr/bin:/bin:/usr/sbin:/sbin root sandbox create
```

**Expected (when Docker is unavailable):**
- Clear error message: "No sandbox provider is available."
- Explanation that Root requires Docker to create sandboxes.
- Suggestion to install Docker Desktop and link to https://docker.com.
- Suggestion to verify with `docker info`.
- No panic, no crash.
- Exit code 1.

```bash
PATH=/usr/bin:/bin:/usr/sbin:/sbin root sandbox list
```

**Expected:**
- Error indicating Docker is unavailable.
- Exit code non-zero.

**Cleanup:** Restart Docker or restore PATH. Verify:

```bash
docker info
```

**Expected:** Docker is available and `docker info` succeeds.

---

## 12. JSON Output

Add `--json` to every sandbox command and verify structured output.

```bash
root sandbox create --json
```

**Expected (JSON):**
```json
{
  "success": true,
  "id": "...",
  "name": "root-sandbox-default",
  "image": "ubuntu:latest",
  "status": "running",
  "created_at": "..."
}
```

```bash
# Note the id from the create output, then:
root sandbox run <id> -- echo hello --json
```

**Expected (JSON):**
```json
{
  "success": true,
  "sandbox_id": "...",
  "command": "echo hello",
  "exit_code": 0,
  "stdout": "hello\n",
  "stderr": ""
}
```

```bash
root sandbox list --json
```

**Expected (JSON):**
```json
{
  "success": true,
  "sandboxes": [
    {
      "id": "...",
      "name": "root-sandbox-default",
      "status": "running",
      "created_at": "...",
      "image": "ubuntu:latest"
    }
  ]
}
```

```bash
root sandbox destroy <id> --json
```

**Expected (JSON):**
```json
{
  "success": true,
  "id": "<id>"
}
```

**Error JSON** (test on a known failure, e.g. destroy a nonexistent sandbox):

```bash
root sandbox destroy nonexistent --json
```

**Expected (JSON):**
```json
{
  "success": false,
  "message": "Sandbox 'nonexistent' not found"
}
```

- Exit code 1.

```bash
root history --json --limit 5
```

**Expected (JSON):**
- Valid JSON array or object.
- Includes sandbox events from the operations above.
- Each event has `event_type`, `timestamp`, `status`, `details`.

---

## Validation Checklist

| Test Path                                          | Status | Notes |
|----------------------------------------------------|--------|-------|
| 1. Fresh sandbox create, run, destroy              | ☐      | |
| 2. Named sandbox with `--image alpine:latest`      | ☐      | |
| 3. Resource limits (`--memory`, `--cpus`)          | ☐      | |
| 4. Timeout (`--timeout 5` on long-running command) | ☐      | |
| 5. Invalid command (nonexistent binary)            | ☐      | |
| 6. Destroy non-existent sandbox                    | ☐      | |
| 7. Repeated destroy (first succeeds, second fails) | ☐      | |
| 8. Run destroyed sandbox (invalid lifecycle)       | ☐      | |
| 9. List sandboxes (create two, list, destroy)      | ☐      | |
| 10. Event ledger for sandbox operations            | ☐      | |
| 11. Docker unavailable (clear error message)       | ☐      | |
| 12. JSON output (create, run, list, destroy)       | ☐      | |
| No panics or crashes on any error path             | ☐      | |
| All sandbox containers use `root-sandbox-` prefix  | ☐      | |
| History correctly records sandbox events           | ☐      | |

---

## Cleanup

```bash
# Remove any remaining sandbox containers
root sandbox list --json | python3 -c "
import json, subprocess, sys
data = json.load(sys.stdin)
for sb in data.get('sandboxes', []):
    subprocess.run(['root', 'sandbox', 'destroy', sb['name'].replace('root-sandbox-', '')])
"

# Or remove by force directly via Docker (if root CLI is unavailable)
docker rm -f $(docker ps -aq --filter name=root-sandbox-) 2>/dev/null || true
```

If you used a disposable Root directory via `ROOT_DIR`:

```bash
rm -rf "$ROOT_DIR"
unset ROOT_DIR
```
