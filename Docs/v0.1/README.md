# Root v0.1 Believable Ship Bundle

This documentation bundle scopes the smallest credible Root release.

Root v0.1 should prove one thing:

> A developer can install a tool, see what changed, inspect the history, and roll back safely.

The intentionally narrow v0.1 command surface:

```bash
root install ffmpeg
root history
root rollback
root doctor
```

Non-goals for v0.1:

- Agent runtime
- Permissions
- Sandboxes
- Machine sync
- Desktop app
- Cloud
- Enterprise
- Team workflows
- Full Homebrew replacement
- Full Nix abstraction

The v0.1 goal is not to finish Root.
The v0.1 goal is to make Root believable.
