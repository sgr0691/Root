# Terminal Recordings

This directory holds real terminal recordings for the Root launch video.

## Required Recordings

| File | Command | Duration |
|------|---------|----------|
| `plan-install.mp4` | `root plan install terraform` | 4–6s |
| `install.mp4` | `root install terraform` | 4–6s |
| `verify.mp4` | `root verify terraform` | 3–4s |
| `history.mp4` | `root history` | 3–4s |
| `rollback.mp4` | `root rollback` | 3–4s |

## Recording Guidelines

- Use a clean terminal with dark background (matches video theme `#0a0a0a`)
- Font: JetBrains Mono or SF Mono, 16–18pt
- Terminal width: ~80 columns
- Record at 30fps or 60fps (Remotion will handle frame matching)
- Capture the full command + output, including any success indicators
- Keep recordings tight — trim dead time before/after the command

## Recommended Tools

- **macOS**: Screen Studio, Kap, or CleanShot X
- **Cross-platform**: asciinema (then convert to video)

## Integration

Once recordings are in place, scenes switch from synthetic to recorded by changing one prop:

```tsx
// Before (synthetic)
<TerminalPlayback lines={[...]} source="synthetic" />

// After (real recording)
<TerminalPlayback
  source="recording"
  recordingSrc="../assets/recordings/plan-install.mp4"
/>
```

No scene logic changes required.
