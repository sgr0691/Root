# Root v0.1 CLI Spec

## Command: `root doctor`

### Purpose

Check if Root is ready to manage packages.

### Usage

```bash
root doctor
```

### Checks

- Nix exists.
- Root data directory is writable.
- Events file is writable.
- Snapshot directory is writable.
- Profile directory is writable.
- v0.1 package allowlist is loaded.

### Success Output

```text
Root doctor

✓ Nix installed
✓ Root data directory writable
✓ History store writable
✓ Snapshot directory writable
✓ Root profile available
✓ ffmpeg supported

Root is ready.
```

### Failure Output

```text
Root doctor

✗ Nix not found

Root needs Nix to install packages safely.

Install Nix:
  https://nixos.org/download

Then run:
  root doctor
```

## Command: `root install ffmpeg`

### Purpose

Install ffmpeg through Root's Nix-backed managed profile.

### Usage

```bash
root install ffmpeg
```

### Behavior

1. Validate ffmpeg is supported.
2. Run doctor preflight.
3. Create snapshot.
4. Install through Nix.
5. Verify ffmpeg.
6. Record event.
7. Print rollback instructions.

### Success Output

```text
Root install

Package:
  ffmpeg

Snapshot:
  created snap_01HX...

Installing with Nix...
Verifying ffmpeg...

Done.

Changed:
  + ffmpeg

Verified:
  ffmpeg -version succeeded

Rollback:
  root rollback
```

### Unsupported Package Output

```text
Root v0.1 does not support "postgres" yet.

Supported packages:
  ffmpeg

More packages are coming soon.
```

## Command: `root history`

### Purpose

Show Root-managed machine events.

### Usage

```bash
root history
```

### Success Output

```text
Root history

2026-06-03 14:22
  install ffmpeg
  status: verified
  snapshot: snap_01HX...

2026-06-03 14:31
  rollback
  status: completed
  restored: snap_01HX...
```

### Empty Output

```text
Root history

No Root-managed changes yet.

Try:
  root install ffmpeg
```

## Command: `root rollback`

### Purpose

Undo the latest Root-managed install by restoring the previous Root snapshot.

### Usage

```bash
root rollback
```

### Success Output

```text
Root rollback

Restored:
  snap_01HX...

Reverted:
  ffmpeg

History updated.
```

### No Rollback Output

```text
Root rollback

No rollback is available yet.

Root can only roll back changes it has managed.
Try:
  root install ffmpeg
```
