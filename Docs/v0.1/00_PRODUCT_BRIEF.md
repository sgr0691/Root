# Root v0.1 Product Brief

## One-liner

Root is a safer Brew for developers and AI coding agents, powered by Nix with no Nix knowledge needed.

## v0.1 Positioning

Root v0.1 is not a complete package manager.

Root v0.1 is a trust demo.

It proves that Root can:

1. Install a real developer tool through Nix.
2. Record that change as a machine event.
3. Show the user a readable machine history.
4. Roll the machine back to the previous managed state.
5. Diagnose whether Root is healthy enough to trust.

## Core Promise

When a developer runs:

```bash
root install ffmpeg
```

Root should answer:

- What will change?
- What changed?
- What stayed unchanged?
- Was the install verified?
- Can I roll it back?

## Why v0.1 Is So Small

The podcast feedback validated the wedge:

> Developers like the idea of Nix, but many do not want to learn Nix.

That means v0.1 should not start with the whole Root vision.
It should start with one clean experience that makes a developer say:

> Oh, I get it. This is Brew, but safer.

## v0.1 User

A developer who:

- Uses Homebrew today.
- Has been burned by Homebrew upgrades or dependency drift.
- Is curious about Nix.
- Does not want to write Nix expressions.
- Wants a simple CLI.
- May use AI coding agents later, but does not need that in v0.1.

## v0.1 Success Definition

Root v0.1 is successful if a user can run:

```bash
root doctor
root install ffmpeg
root history
root rollback
root doctor
```

and clearly understand:

- Root works on their machine.
- ffmpeg was installed.
- Root recorded the event.
- Root can roll back the managed environment.
- Root did not ask them to understand Nix.

## v0.1 Demo Script

```bash
root doctor
root install ffmpeg
ffmpeg -version
root history
root rollback
root history
```

The demo should show the full trust loop:

```text
Check machine
Install tool
Verify tool
Show history
Undo change
Show history again
```
