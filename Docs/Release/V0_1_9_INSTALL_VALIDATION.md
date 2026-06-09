# Root v0.1.9 Live Install Validation Matrix

Manual real-machine testing document for v0.1.9 release validation.

---

## Instructions

For each package, run the following commands and record the results:

```bash
root plan install <pkg>
root install <pkg>
root verify <pkg>
root history
root rollback
```

---

## Packages

### ffmpeg

| Step | Command | Expected Result | Actual |
|------|---------|----------------|--------|
| Plan | `root plan install ffmpeg` | Shows ffmpeg, nix attr, binaries, verify commands | ☐ |
| Install | `root install ffmpeg` | Success, snapshot created | ☐ |
| Verify | `root verify ffmpeg` | Binary found in profile, `ffmpeg -version` works | ☐ |
| History | `root history` | Install event recorded | ☐ |
| Rollback | `root rollback --last` | ffmpeg removed, prior state restored | ☐ |

---

### poppler

| Step | Command | Expected Result | Actual |
|------|---------|----------------|--------|
| Plan | `root plan install poppler` | Shows poppler with pdftotext, pdfinfo binaries | ☐ |
| Install | `root install poppler` | Success | ☐ |
| Verify | `root verify poppler` | Both pdftotext and pdfinfo verified | ☐ |
| History | `root history` | Events recorded | ☐ |
| Rollback | `root rollback --last` | poppler removed | ☐ |

---

### ripgrep

| Step | Command | Expected Result | Actual |
|------|---------|----------------|--------|
| Plan | `root plan install ripgrep` | Shows ripgrep with rg binary | ☐ |
| Install | `root install ripgrep` | Success | ☐ |
| Verify | `root verify ripgrep` | rg verified in profile | ☐ |
| History | `root history` | Events recorded | ☐ |
| Rollback | `root rollback --last` | ripgrep removed | ☐ |

---

### jq

| Step | Command | Expected Result | Actual |
|------|---------|----------------|--------|
| Plan | `root plan install jq` | Shows jq | ☐ |
| Install | `root install jq` | Success | ☐ |
| Verify | `root verify jq` | jq verified | ☐ |
| Rollback | `root rollback --last` | jq removed | ☐ |

---

### go

| Step | Command | Expected Result | Actual |
|------|---------|----------------|--------|
| Plan | `root plan install go` | Shows go | ☐ |
| Install | `root install go` | Success | ☐ |
| Verify | `root verify go` | `go version` works | ☐ |
| Rollback | `root rollback --last` | go removed | ☐ |

---

### postgresql

| Step | Command | Expected Result | Actual |
|------|---------|----------------|--------|
| Plan | `root plan install postgresql` | Shows psql, postgres binaries | ☐ |
| Install | `root install postgresql` | Success | ☐ |
| Verify | `root verify postgresql` | Both psql and postgres verified | ☐ |
| Rollback | `root rollback --last` | postgresql removed | ☐ |

---

### terraform

| Step | Command | Expected Result | Actual |
|------|---------|----------------|--------|
| Plan | `root plan install terraform` | Shows terraform | ☐ |
| Install | `root install terraform` | Success | ☐ |
| Verify | `root verify terraform` | `terraform version` works | ☐ |
| Rollback | `root rollback --last` | terraform removed | ☐ |

---

### kubectl

| Step | Command | Expected Result | Actual |
|------|---------|----------------|--------|
| Plan | `root plan install kubectl` | Shows kubectl | ☐ |
| Install | `root install kubectl` | Success | ☐ |
| Verify | `root verify kubectl` | `kubectl version --client` works | ☐ |
| Rollback | `root rollback --last` | kubectl removed | ☐ |

---

### docker-client

| Step | Command | Expected Result | Actual |
|------|---------|----------------|--------|
| Plan | `root plan install docker-client` | Shows docker-client | ☐ |
| Install | `root install docker-client` | Success | ☐ |
| Verify | `root verify docker-client` | `docker --version` works | ☐ |
| Rollback | `root rollback --last` | docker-client removed | ☐ |

---

### git-delta

| Step | Command | Expected Result | Actual |
|------|---------|----------------|--------|
| Plan | `root plan install git-delta` | Shows git-delta | ☐ |
| Install | `root install git-delta` | Success | ☐ |
| Verify | `root verify git-delta` | `delta --version` works | ☐ |
| Rollback | `root rollback --last` | git-delta removed | ☐ |

---

### zoxide

| Step | Command | Expected Result | Actual |
|------|---------|----------------|--------|
| Plan | `root plan install zoxide` | Shows zoxide | ☐ |
| Install | `root install zoxide` | Success | ☐ |
| Verify | `root verify zoxide` | `zoxide --version` works | ☐ |
| Rollback | `root rollback --last` | zoxide removed | ☐ |

---

### lazygit

| Step | Command | Expected Result | Actual |
|------|---------|----------------|--------|
| Plan | `root plan install lazygit` | Shows lazygit | ☐ |
| Install | `root install lazygit` | Success | ☐ |
| Verify | `root verify lazygit` | `lazygit --version` works | ☐ |
| Rollback | `root rollback --last` | lazygit removed | ☐ |

---

## Cross-Cutting Checks

### Lockfile correctness
- [ ] All output store paths do NOT end in `.drv`
- [ ] All `drv_path` fields DO end in `.drv`
- [ ] Alias installs store canonical `name` and original `requested`
- [ ] `nixpkgs.rev` is a concrete commit hash, not `"unknown"`

### Verification correctness
- [ ] `root verify` uses `~/.root/profiles/default/bin/<binary>`, not global PATH
- [ ] Missing profile binary fails even if global binary exists
- [ ] Multi-binary packages report each binary separately
- [ ] Non-standard args are correct:
  - `go version`
  - `terraform version`
  - `kubectl version --client`
  - `helm version --short`
  - `tmux -V`
  - `direnv version`

### Rollback correctness
- [ ] Rollback restores from locked state, not package names
- [ ] Rollback verifies resulting profile store paths
- [ ] Rollback fails closed if locked state cannot be reproduced
- [ ] Rollback records history events
- [ ] Rollback does not overwrite Rootfile/root.lock before successful mutation
