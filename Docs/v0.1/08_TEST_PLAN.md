# Root v0.1 Test Plan

## Testing Goal

Prove the core trust loop works:

```text
doctor → install → verify → history → rollback
```

## Unit Tests

### Package Resolution

- Resolves `ffmpeg`.
- Rejects unsupported packages.
- Returns correct binary name.
- Returns correct Nix attr.

### Events

- Serializes install event.
- Deserializes install event.
- Appends event to JSONL.
- Reads events in order.
- Handles empty event file.

### Snapshots

- Creates snapshot with empty package list.
- Creates snapshot with ffmpeg.
- Reads snapshot by id.
- Selects previous snapshot for rollback.
- Handles missing snapshot.

### Doctor

- Passes when Nix exists and paths are writable.
- Fails when Nix is missing.
- Fails when events file is not writable.
- Fails when snapshot directory is not writable.

### Output Rendering

- Renders healthy doctor output.
- Renders install success.
- Renders unsupported package.
- Renders empty history.
- Renders rollback unavailable.

## Integration Tests

Where possible, use temporary directories for Root state.

### Test: Doctor Creates State

```bash
ROOT_HOME=/tmp/root-test root doctor
```

Expected:

- Creates events file.
- Creates snapshots directory.
- Creates profile directory.

### Test: Unsupported Package

```bash
root install postgres
```

Expected:

- Non-zero exit.
- Helpful unsupported package message.
- No install event recorded.

### Test: Event Roundtrip

1. Append install event.
2. Run history.
3. Confirm event appears.

### Test: Snapshot Roundtrip

1. Create snapshot.
2. Read snapshot.
3. Confirm package state matches.

## Manual Smoke Test

Run on a clean macOS dev machine:

```bash
root doctor
root install ffmpeg
ffmpeg -version
root history
root rollback
root history
```

Expected:

- Doctor passes.
- Install completes.
- Verify passes.
- History shows install.
- Rollback completes.
- History shows rollback.

## Launch Blockers

Do not release v0.1 if:

- `root rollback` can leave Root state corrupted.
- `root history` silently loses events.
- `root install ffmpeg` succeeds but verification fails without warning.
- `root doctor` says healthy when Nix is missing.
- README implies Root manages the whole machine.
