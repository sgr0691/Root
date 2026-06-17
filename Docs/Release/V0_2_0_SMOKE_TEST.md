# Root v0.2.0 Smoke Test

Manual release validation for the Roadmap Phase 1–6 command surface. Run the
full automated CI sequence first, then execute the relevant real Nix and Docker
checks on a disposable Root directory.

## Automated Gates

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo build
target/debug/root --version
```

Expected: every command succeeds and the binary reports `root 0.2.0`.

## Isolated Test State

```bash
export ROOT_DIR="$(mktemp -d)/root"
target/debug/root init
```

Do not point `ROOT_DIR` at an existing Root installation during release smoke
testing.

## Phase 1: Search and Update

```bash
target/debug/root search rg
target/debug/root search rg --json
target/debug/root install ripgrep
target/debug/root update ripgrep
```

Expected: `rg` resolves to `ripgrep`; install and update preserve deterministic
v2 lock metadata and record snapshots/history.

## Phase 2: Sync and Restore

```bash
cp "$ROOT_DIR/root.lock" /tmp/root-v0.2.0-shared.lock
target/debug/root sync
target/debug/root restore --lock /tmp/root-v0.2.0-shared.lock
```

Expected: both commands reconcile only the Root-managed profile and report a
snapshot ID.

## Phase 3: Execution Runtime

Add this task to `$ROOT_DIR/Rootfile`:

```toml
[tasks]
smoke = "printf 'root-run-ok\\n'"
```

Then run:

```bash
target/debug/root run smoke
target/debug/root run -- printf root-command-ok
target/debug/root run smoke --json
```

Expected: commands execute with the Root profile first on `PATH`; JSON includes
exit code, duration, captured stdout/stderr policy, and success.

## Phase 4: Policies

Create `/tmp/root-v0.2.0-policy.toml`:

```toml
version = 1

[packages]
remove = "deny"

[execution]
run = "allow"

[sandboxes]
create = "allow"
run = "allow"
destroy = "allow"
```

```bash
target/debug/root policy apply /tmp/root-v0.2.0-policy.toml
target/debug/root permissions --json
target/debug/root remove ripgrep
```

Expected: permissions output reflects the file and remove exits with policy
denial code `9` without creating a mutation snapshot.

## Phase 5: Docker Sandboxes

```bash
docker info
target/debug/root sandbox create smoke --image ubuntu:latest
target/debug/root sandbox list --json
target/debug/root sandbox run root-sandbox-smoke -- printf sandbox-ok
target/debug/root sandbox destroy root-sandbox-smoke
```

Expected: create, run, list, and destroy succeed. If Docker is unavailable,
Root must return a capability error and must not claim secure isolation.

## Phase 6: Machine Status

```bash
target/debug/root status
target/debug/root status --json
target/debug/root history
```

Expected: status includes a stable machine ID and reports Rootfile, lockfile,
and Root-profile counts. History includes execution, policy, sandbox, update,
and restore events generated during this checklist.

## Cleanup

```bash
docker rm -f root-sandbox-smoke 2>/dev/null || true
rm -f /tmp/root-v0.2.0-shared.lock /tmp/root-v0.2.0-policy.toml
rm -rf "$(dirname "$ROOT_DIR")"
unset ROOT_DIR
```
