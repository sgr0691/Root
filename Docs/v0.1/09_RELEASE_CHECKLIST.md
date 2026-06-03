# Root v0.1 Release Checklist

## Code

- [ ] Rust workspace builds.
- [ ] `cargo fmt --check` passes.
- [ ] `cargo clippy` passes.
- [ ] `cargo test` passes.
- [ ] `root doctor` works.
- [ ] `root install ffmpeg` works.
- [ ] `root history` works.
- [ ] `root rollback` works.
- [ ] Unsupported package messages are clear.

## Docs

- [ ] README explains Root clearly.
- [ ] README says v0.1 only supports ffmpeg.
- [ ] README says rollback only applies to Root-managed packages.
- [ ] README includes demo commands.
- [ ] README includes known limitations.
- [ ] README includes roadmap.
- [ ] LICENSE is Apache 2.0.

## Demo

- [ ] Record terminal demo.
- [ ] Show doctor.
- [ ] Show install.
- [ ] Show ffmpeg verification.
- [ ] Show history.
- [ ] Show rollback.
- [ ] Show history after rollback.

## Launch

- [ ] GitHub repo public.
- [ ] README polished.
- [ ] Release tag created.
- [ ] X post drafted.
- [ ] Podcast thank-you post drafted.
- [ ] Ask for stars/contributors honestly.
- [ ] Open issues for next packages:
  - ripgrep
  - jq
  - poppler
  - imagemagick

## Do Not Launch If

- Install requires hidden manual steps.
- Rollback is unreliable.
- Doctor is misleading.
- README overpromises.
