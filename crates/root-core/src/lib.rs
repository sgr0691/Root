use anyhow::{Context, Result};
use root_lockfile::{
    get_root_dir, LockProfile, LockedPackage, LockedPackageOutput, LockedPackageV2, NixRuntime,
    NixpkgsConfig, NixpkgsConfigV2, RootLock, RootLockV2, Rootfile, ROOT_LOCK_SCHEMA_VERSION,
};
use root_nix::NixAdapter;
use root_sandbox::SandboxProvider;
use root_snapshot::{list_snapshot_summaries, list_snapshots, Snapshot, SnapshotSummary};
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq)]
pub struct VerifyCommand {
    pub binary: &'static str,
    pub args: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq)]
pub struct PackageSpec {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub nix_attr: &'static str,
    pub binaries: &'static [&'static str],
    pub verify: &'static [VerifyCommand],
    pub category: &'static str,
    pub description: &'static str,
}

pub const SUPPORTED_PACKAGES: &[PackageSpec] = &[
    // media
    PackageSpec {
        name: "ffmpeg",
        aliases: &[],
        nix_attr: "nixpkgs#ffmpeg",
        binaries: &["ffmpeg"],
        verify: &[VerifyCommand {
            binary: "ffmpeg",
            args: &["-version"],
        }],
        category: "media",
        description: "Video/audio processing",
    },
    PackageSpec {
        name: "imagemagick",
        aliases: &[],
        nix_attr: "nixpkgs#imagemagick",
        binaries: &["magick", "convert"],
        verify: &[VerifyCommand {
            binary: "magick",
            args: &["--version"],
        }],
        category: "media",
        description: "Image manipulation",
    },
    PackageSpec {
        name: "poppler",
        aliases: &[],
        nix_attr: "nixpkgs#poppler",
        binaries: &["pdftotext", "pdfinfo"],
        verify: &[
            VerifyCommand {
                binary: "pdftotext",
                args: &["-v"],
            },
            VerifyCommand {
                binary: "pdfinfo",
                args: &["-v"],
            },
        ],
        category: "media",
        description: "PDF utilities",
    },
    // search
    PackageSpec {
        name: "ripgrep",
        aliases: &["rg"],
        nix_attr: "nixpkgs#ripgrep",
        binaries: &["rg"],
        verify: &[VerifyCommand {
            binary: "rg",
            args: &["--version"],
        }],
        category: "search",
        description: "Fast recursive search",
    },
    PackageSpec {
        name: "fd",
        aliases: &[],
        nix_attr: "nixpkgs#fd",
        binaries: &["fd"],
        verify: &[VerifyCommand {
            binary: "fd",
            args: &["--version"],
        }],
        category: "search",
        description: "Fast file finder",
    },
    PackageSpec {
        name: "fzf",
        aliases: &[],
        nix_attr: "nixpkgs#fzf",
        binaries: &["fzf"],
        verify: &[VerifyCommand {
            binary: "fzf",
            args: &["--version"],
        }],
        category: "search",
        description: "Fuzzy file finder",
    },
    // dev
    PackageSpec {
        name: "bat",
        aliases: &[],
        nix_attr: "nixpkgs#bat",
        binaries: &["bat"],
        verify: &[VerifyCommand {
            binary: "bat",
            args: &["--version"],
        }],
        category: "dev",
        description: "File viewer with syntax highlighting",
    },
    PackageSpec {
        name: "bun",
        aliases: &[],
        nix_attr: "nixpkgs#bun",
        binaries: &["bun"],
        verify: &[VerifyCommand {
            binary: "bun",
            args: &["--version"],
        }],
        category: "dev",
        description: "JavaScript runtime and bundler",
    },
    PackageSpec {
        name: "eza",
        aliases: &[],
        nix_attr: "nixpkgs#eza",
        binaries: &["eza"],
        verify: &[VerifyCommand {
            binary: "eza",
            args: &["--version"],
        }],
        category: "dev",
        description: "Modern ls replacement",
    },
    PackageSpec {
        name: "gh",
        aliases: &[],
        nix_attr: "nixpkgs#gh",
        binaries: &["gh"],
        verify: &[VerifyCommand {
            binary: "gh",
            args: &["--version"],
        }],
        category: "dev",
        description: "GitHub CLI",
    },
    PackageSpec {
        name: "git-lfs",
        aliases: &[],
        nix_attr: "nixpkgs#git-lfs",
        binaries: &["git-lfs"],
        verify: &[VerifyCommand {
            binary: "git-lfs",
            args: &["--version"],
        }],
        category: "dev",
        description: "Git large file storage",
    },
    PackageSpec {
        name: "gnumake",
        aliases: &["make"],
        nix_attr: "nixpkgs#gnumake",
        binaries: &["make"],
        verify: &[VerifyCommand {
            binary: "make",
            args: &["--version"],
        }],
        category: "dev",
        description: "Build automation",
    },
    PackageSpec {
        name: "httpie",
        aliases: &[],
        nix_attr: "nixpkgs#httpie",
        binaries: &["http"],
        verify: &[VerifyCommand {
            binary: "http",
            args: &["--version"],
        }],
        category: "dev",
        description: "HTTP client",
    },
    PackageSpec {
        name: "jq",
        aliases: &[],
        nix_attr: "nixpkgs#jq",
        binaries: &["jq"],
        verify: &[VerifyCommand {
            binary: "jq",
            args: &["--version"],
        }],
        category: "dev",
        description: "JSON processor",
    },
    PackageSpec {
        name: "just",
        aliases: &[],
        nix_attr: "nixpkgs#just",
        binaries: &["just"],
        verify: &[VerifyCommand {
            binary: "just",
            args: &["--version"],
        }],
        category: "dev",
        description: "Command runner",
    },
    PackageSpec {
        name: "nodejs",
        aliases: &["node"],
        nix_attr: "nixpkgs#nodejs",
        binaries: &["node", "npm"],
        verify: &[VerifyCommand {
            binary: "node",
            args: &["--version"],
        }],
        category: "dev",
        description: "JavaScript runtime",
    },
    PackageSpec {
        name: "openssl",
        aliases: &[],
        nix_attr: "nixpkgs#openssl",
        binaries: &["openssl"],
        verify: &[VerifyCommand {
            binary: "openssl",
            args: &["version"],
        }],
        category: "dev",
        description: "Cryptography toolkit",
    },
    PackageSpec {
        name: "pkg-config",
        aliases: &[],
        nix_attr: "nixpkgs#pkg-config",
        binaries: &["pkg-config"],
        verify: &[VerifyCommand {
            binary: "pkg-config",
            args: &["--version"],
        }],
        category: "dev",
        description: "Package configuration",
    },
    PackageSpec {
        name: "python3",
        aliases: &["python"],
        nix_attr: "nixpkgs#python3",
        binaries: &["python3"],
        verify: &[VerifyCommand {
            binary: "python3",
            args: &["--version"],
        }],
        category: "dev",
        description: "Python interpreter",
    },
    PackageSpec {
        name: "sqlite",
        aliases: &[],
        nix_attr: "nixpkgs#sqlite",
        binaries: &["sqlite3"],
        verify: &[VerifyCommand {
            binary: "sqlite3",
            args: &["--version"],
        }],
        category: "dev",
        description: "SQL database engine",
    },
    PackageSpec {
        name: "tree",
        aliases: &[],
        nix_attr: "nixpkgs#tree",
        binaries: &["tree"],
        verify: &[VerifyCommand {
            binary: "tree",
            args: &["--version"],
        }],
        category: "dev",
        description: "Directory tree viewer",
    },
    PackageSpec {
        name: "uv",
        aliases: &[],
        nix_attr: "nixpkgs#uv",
        binaries: &["uv"],
        verify: &[VerifyCommand {
            binary: "uv",
            args: &["--version"],
        }],
        category: "dev",
        description: "Python package manager",
    },
    // net
    PackageSpec {
        name: "wget",
        aliases: &[],
        nix_attr: "nixpkgs#wget",
        binaries: &["wget"],
        verify: &[VerifyCommand {
            binary: "wget",
            args: &["--version"],
        }],
        category: "net",
        description: "URL downloader",
    },
    PackageSpec {
        name: "curl",
        aliases: &[],
        nix_attr: "nixpkgs#curl",
        binaries: &["curl"],
        verify: &[VerifyCommand {
            binary: "curl",
            args: &["--version"],
        }],
        category: "net",
        description: "URL transfer tool",
    },
    // language
    PackageSpec {
        name: "go",
        aliases: &["golang"],
        nix_attr: "nixpkgs#go",
        binaries: &["go"],
        verify: &[VerifyCommand {
            binary: "go",
            args: &["version"],
        }],
        category: "language",
        description: "Go programming language toolchain",
    },
    PackageSpec {
        name: "rustup",
        aliases: &[],
        nix_attr: "nixpkgs#rustup",
        binaries: &["rustup"],
        verify: &[VerifyCommand {
            binary: "rustup",
            args: &["--version"],
        }],
        category: "language",
        description: "Rust toolchain installer and manager",
    },
    // database
    PackageSpec {
        name: "postgresql",
        aliases: &["postgres"],
        nix_attr: "nixpkgs#postgresql",
        binaries: &["psql", "postgres"],
        verify: &[
            VerifyCommand {
                binary: "psql",
                args: &["--version"],
            },
            VerifyCommand {
                binary: "postgres",
                args: &["--version"],
            },
        ],
        category: "database",
        description: "PostgreSQL database server and CLI tools",
    },
    PackageSpec {
        name: "redis",
        aliases: &[],
        nix_attr: "nixpkgs#redis",
        binaries: &["redis-server", "redis-cli"],
        verify: &[
            VerifyCommand {
                binary: "redis-server",
                args: &["--version"],
            },
            VerifyCommand {
                binary: "redis-cli",
                args: &["--version"],
            },
        ],
        category: "database",
        description: "Redis server and command-line client",
    },
    // infrastructure
    PackageSpec {
        name: "terraform",
        aliases: &["tf"],
        nix_attr: "nixpkgs#terraform",
        binaries: &["terraform"],
        verify: &[VerifyCommand {
            binary: "terraform",
            args: &["version"],
        }],
        category: "infrastructure",
        description: "Infrastructure as code CLI",
    },
    PackageSpec {
        name: "kubectl",
        aliases: &["kube"],
        nix_attr: "nixpkgs#kubectl",
        binaries: &["kubectl"],
        verify: &[VerifyCommand {
            binary: "kubectl",
            args: &["version", "--client"],
        }],
        category: "infrastructure",
        description: "Kubernetes command-line tool",
    },
    PackageSpec {
        name: "helm",
        aliases: &[],
        nix_attr: "nixpkgs#kubernetes-helm",
        binaries: &["helm"],
        verify: &[VerifyCommand {
            binary: "helm",
            args: &["version", "--short"],
        }],
        category: "infrastructure",
        description: "Kubernetes package manager",
    },
    PackageSpec {
        name: "k9s",
        aliases: &[],
        nix_attr: "nixpkgs#k9s",
        binaries: &["k9s"],
        verify: &[VerifyCommand {
            binary: "k9s",
            args: &["version"],
        }],
        category: "infrastructure",
        description: "Terminal UI for Kubernetes clusters",
    },
    PackageSpec {
        name: "docker-client",
        aliases: &["docker"],
        nix_attr: "nixpkgs#docker-client",
        binaries: &["docker"],
        verify: &[VerifyCommand {
            binary: "docker",
            args: &["--version"],
        }],
        category: "infrastructure",
        description: "Docker CLI client",
    },
    // security
    PackageSpec {
        name: "age",
        aliases: &[],
        nix_attr: "nixpkgs#age",
        binaries: &["age", "age-keygen"],
        verify: &[
            VerifyCommand {
                binary: "age",
                args: &["--version"],
            },
            VerifyCommand {
                binary: "age-keygen",
                args: &["--version"],
            },
        ],
        category: "security",
        description: "Simple modern file encryption tool",
    },
    PackageSpec {
        name: "sops",
        aliases: &[],
        nix_attr: "nixpkgs#sops",
        binaries: &["sops"],
        verify: &[VerifyCommand {
            binary: "sops",
            args: &["--version"],
        }],
        category: "security",
        description: "Editor for encrypted secrets",
    },
    // editor
    PackageSpec {
        name: "neovim",
        aliases: &["nvim"],
        nix_attr: "nixpkgs#neovim",
        binaries: &["nvim"],
        verify: &[VerifyCommand {
            binary: "nvim",
            args: &["--version"],
        }],
        category: "editor",
        description: "Modern Vim-based text editor",
    },
    // terminal
    PackageSpec {
        name: "tmux",
        aliases: &[],
        nix_attr: "nixpkgs#tmux",
        binaries: &["tmux"],
        verify: &[VerifyCommand {
            binary: "tmux",
            args: &["-V"],
        }],
        category: "terminal",
        description: "Terminal multiplexer",
    },
    // git
    PackageSpec {
        name: "git-delta",
        aliases: &["delta"],
        nix_attr: "nixpkgs#git-delta",
        binaries: &["delta"],
        verify: &[VerifyCommand {
            binary: "delta",
            args: &["--version"],
        }],
        category: "git",
        description: "Syntax-highlighted Git diff viewer.",
    },
    // terminal
    PackageSpec {
        name: "zoxide",
        aliases: &["z"],
        nix_attr: "nixpkgs#zoxide",
        binaries: &["zoxide"],
        verify: &[VerifyCommand {
            binary: "zoxide",
            args: &["--version"],
        }],
        category: "terminal",
        description: "Smarter directory navigation for the terminal.",
    },
    PackageSpec {
        name: "direnv",
        aliases: &[],
        nix_attr: "nixpkgs#direnv",
        binaries: &["direnv"],
        verify: &[VerifyCommand {
            binary: "direnv",
            args: &["version"],
        }],
        category: "terminal",
        description: "Automatically loads and unloads environment variables per directory.",
    },
    PackageSpec {
        name: "starship",
        aliases: &[],
        nix_attr: "nixpkgs#starship",
        binaries: &["starship"],
        verify: &[VerifyCommand {
            binary: "starship",
            args: &["--version"],
        }],
        category: "terminal",
        description: "Cross-shell customizable prompt.",
    },
    // git
    PackageSpec {
        name: "lazygit",
        aliases: &["lg"],
        nix_attr: "nixpkgs#lazygit",
        binaries: &["lazygit"],
        verify: &[VerifyCommand {
            binary: "lazygit",
            args: &["--version"],
        }],
        category: "git",
        description: "Terminal UI for Git workflows.",
    },
];

type UpdateTarget = (String, &'static PackageSpec);

fn resolve_package(name: &str) -> Option<&'static PackageSpec> {
    SUPPORTED_PACKAGES
        .iter()
        .find(|p| p.name == name || p.aliases.contains(&name))
}

/// A file-based mutex guard for mutation commands.
///
/// Acquires the lock by atomically creating `~/.root/root.lockfile`
/// with the current PID and a timestamp. On contention, checks whether
/// the lock-holding process is still alive. If the process is dead,
/// the stale lock is removed and re-acquisition is attempted.
///
/// Released on Drop (which removes the lock file).
#[derive(Debug)]
struct MutationGuard {
    lock_path: PathBuf,
}

impl MutationGuard {
    fn acquire() -> anyhow::Result<Self> {
        let dir = root_lockfile::init_root_dir()?;
        let lock_path = dir.join("root.lockfile");

        let pid = std::process::id();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let lock_content = format!("{}\n{}\n", pid, now);

        match Self::try_acquire(&lock_path, &lock_content) {
            Ok(()) => Ok(Self { lock_path }),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                // Stale lock recovery: read the lock file, check if the PID is alive
                match Self::check_stale_lock(&lock_path) {
                    Ok(true) => Err(anyhow::anyhow!(
                        "Another Root mutation is in progress (PID {}).\n\
                         If this is unexpected, delete ~/.root/root.lockfile and try again.",
                        pid
                    )),
                    Ok(false) => {
                        // Stale lock — remove it and retry
                        let _ = std::fs::remove_file(&lock_path);
                        Self::try_acquire(&lock_path, &lock_content).with_context(|| {
                            "Failed to acquire mutation lock after recovering stale lock"
                        })?;
                        Ok(Self { lock_path })
                    }
                    Err(_) => Err(anyhow::anyhow!(
                        "Lock file ~/.root/root.lockfile exists and could not be read.\n\
                         Delete it manually and try again."
                    )),
                }
            }
            Err(e) => Err(anyhow::anyhow!("Failed to acquire mutation lock: {}", e)),
        }
    }

    fn try_acquire(lock_path: &Path, content: &str) -> std::io::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(lock_path)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        Ok(())
    }

    /// Returns `Ok(true)` if the lock holder's PID is still alive,
    /// `Ok(false)` if the process is dead (stale), or `Err` if the lock
    /// file is unreadable or malformed.
    fn check_stale_lock(lock_path: &Path) -> Result<bool> {
        let mut content = String::new();
        std::fs::File::open(lock_path)
            .and_then(|mut f| f.read_to_string(&mut content))
            .map_err(|_| anyhow::anyhow!("Cannot read lock file"))?;

        let pid_str = content.lines().next().unwrap_or("").trim();
        let lock_pid: u32 = pid_str
            .parse()
            .map_err(|_| anyhow::anyhow!("Malformed lock file (invalid PID)"))?;

        // Check if the PID is alive (signal 0 test, portable on Unix)
        let status = std::process::Command::new("kill")
            .arg("-0")
            .arg(lock_pid.to_string())
            .output()
            .map_err(|_| anyhow::anyhow!("Cannot check process liveness"))?;

        Ok(status.status.success())
    }
}

impl Drop for MutationGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.lock_path);
    }
}

pub mod brew;
pub mod events;
pub mod execution;
pub mod policy;

pub use execution::{run, RunReport, RunRequest};

#[derive(Debug, Serialize)]
pub struct ListPackage {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct ListOutput {
    pub packages: Vec<ListPackage>,
    pub nix_profile: String,
}

#[derive(Debug, Serialize)]
pub struct InitReport {
    pub success: bool,
    pub root_dir: String,
    pub nix_detected: bool,
    pub profile_ready: bool,
    pub snapshot_enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct InstallReport {
    pub success: bool,
    pub operation: &'static str,
    pub package: String,
    pub changed: Vec<String>,
    pub unchanged: Vec<String>,
    pub snapshot_id: String,
    pub rollback_available: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PlanReport {
    pub package: String,
    pub original_input: Option<String>,
    pub found: bool,
    pub description: String,
    pub would_create_snapshot: bool,
    pub attributes: Vec<String>,
    pub nix_attr: String,
    pub expected_binaries: Vec<String>,
    pub verify_commands: Vec<String>,
    pub rollback_available: bool,
}

#[derive(Debug, Serialize)]
pub struct SearchMatch {
    pub name: &'static str,
    pub aliases: Vec<String>,
    pub category: &'static str,
    pub description: &'static str,
    pub nix_attr: &'static str,
    pub binaries: Vec<String>,
    pub verify: Vec<String>,
    pub matched_fields: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchOutput {
    pub query: String,
    pub matches: Vec<SearchMatch>,
    pub supported_count: usize,
}

#[derive(Debug, Serialize)]
pub struct RemoveReport {
    pub success: bool,
    pub package: String,
    pub snapshot_id: String,
    pub rollback_available: bool,
}

#[derive(Debug, Serialize)]
pub struct RollbackReport {
    pub success: bool,
    pub from_snapshot: String,
    pub packages_removed: Vec<String>,
    pub packages_restored: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct HistoryOutput {
    pub snapshots: Vec<SnapshotSummary>,
    pub events: Vec<events::RootEvent>,
}

#[derive(Debug, Serialize)]
pub struct UpdateReport {
    pub success: bool,
    pub requested: Option<String>,
    pub updated: Vec<String>,
    pub unchanged: Vec<String>,
    pub skipped: Vec<String>,
    pub failed: Vec<String>,
    pub snapshot_id: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PermissionsReport {
    pub success: bool,
    pub path: String,
    pub source: &'static str,
    pub policy: policy::RootPolicy,
}

#[derive(Debug, Serialize)]
pub struct PolicyApplyReport {
    pub success: bool,
    pub path: String,
    pub version: u32,
}

pub fn permissions() -> Result<PermissionsReport> {
    root_lockfile::init_root_dir()?;
    let path = policy::policy_path()?;
    let (policy, configured) = policy::read_policy()?;
    Ok(PermissionsReport {
        success: true,
        path: path.to_string_lossy().to_string(),
        source: if configured { "configured" } else { "default" },
        policy,
    })
}

pub fn apply_policy(source: &Path) -> Result<PolicyApplyReport> {
    let _guard = MutationGuard::acquire()?;
    let (policy, destination) = policy::apply_policy(source)?;
    events::record_policy_event(
        "root policy apply",
        events::RootEventStatus::Completed,
        "applied",
        format!("Activated policy from {}", source.display()),
    )?;
    Ok(PolicyApplyReport {
        success: true,
        path: destination.to_string_lossy().to_string(),
        version: policy.version,
    })
}

pub(crate) fn enforce_policy(action: policy::PolicyAction, subject: Option<&str>) -> Result<()> {
    let (active_policy, _) = policy::read_policy()?;
    let decision = policy::evaluate(&active_policy, action, subject);
    let status = if decision.allowed {
        events::RootEventStatus::Completed
    } else {
        events::RootEventStatus::Failed
    };
    events::record_policy_event(
        &format!("policy check {}", action.as_str()),
        status,
        if decision.allowed {
            "allowed"
        } else {
            "denied"
        },
        decision.reason.clone(),
    )?;
    if decision.allowed {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Policy denied {}{}: {}",
            action.as_str(),
            subject
                .map(|value| format!(" '{}'", value))
                .unwrap_or_default(),
            decision.reason
        ))
    }
}

/// Install Nix using the official multi-user installer.
pub fn install_nix() -> Result<()> {
    let status = std::process::Command::new("sh")
        .args(["-c", "curl -L https://nixos.org/nix/install | sh"])
        .status()
        .context("Failed to run Nix installer")?;

    if !status.success() {
        Err(anyhow::anyhow!(
            "Nix installer exited with code {:?}",
            status.code()
        ))
    } else {
        Ok(())
    }
}

pub fn init(adapter: &impl NixAdapter) -> Result<InitReport> {
    let root_dir =
        root_lockfile::init_root_dir().context("Failed to initialize Root directories")?;
    let nix_detected = adapter
        .check_availability()
        .map_err(|e| anyhow::anyhow!(e))
        .unwrap_or(false);
    Ok(InitReport {
        success: true,
        root_dir: root_dir.to_string_lossy().to_string(),
        nix_detected,
        profile_ready: true,
        snapshot_enabled: true,
    })
}

pub(crate) fn get_or_create_rootfile() -> Result<Rootfile> {
    let dir = get_root_dir()?;
    let path = dir.join("Rootfile");
    if path.exists() {
        Rootfile::read_from_file(&path)
    } else {
        Ok(Rootfile::default())
    }
}

fn save_rootfile(rootfile: &Rootfile) -> Result<()> {
    let path = get_root_dir()?.join("Rootfile");
    rootfile.write_to_file(&path)
}

fn get_or_create_lock() -> Result<RootLock> {
    let dir = get_root_dir()?;
    let path = dir.join("root.lock");
    if path.exists() {
        RootLock::read_from_file(&path)
    } else {
        Ok(RootLock {
            version: ROOT_LOCK_SCHEMA_VERSION,
            platform: root_lockfile::detect_platform()?,
            nixpkgs: NixpkgsConfig {
                rev: "unknown".into(),
                source: "github:NixOS/nixpkgs".into(),
            },
            packages: Vec::new(),
        })
    }
}

#[cfg(test)]
fn save_lock(lock: &RootLock) -> Result<()> {
    let path = get_root_dir()?.join("root.lock");
    lock.write_to_file(&path)
}

fn get_or_create_lock_v2() -> Result<RootLockV2> {
    let dir = get_root_dir()?;
    let path = dir.join("root.lock");
    if path.exists() {
        RootLockV2::read_from_file(&path)
            .or_else(|_| RootLock::read_from_file(&path).map(|lock| lock.to_v2()))
    } else {
        Ok(get_or_create_lock()?.to_v2())
    }
}

fn save_lock_v2(lock: &RootLockV2) -> Result<()> {
    let path = get_root_dir()?.join("root.lock");
    lock.write_to_file(&path)
}

fn legacy_lock_from_v2(lock: &RootLockV2) -> RootLock {
    RootLock {
        version: lock.version,
        platform: lock.platform.clone(),
        nixpkgs: NixpkgsConfig {
            rev: lock.nixpkgs.rev.clone(),
            source: lock.nixpkgs.source.clone(),
        },
        packages: lock.packages.iter().map(legacy_package_from_v2).collect(),
    }
}

fn locked_package_changed(current: &LockedPackageV2, target: &LockedPackageV2) -> bool {
    serde_json::to_value(current).ok() != serde_json::to_value(target).ok()
}

fn locked_installable_for(
    adapter: &impl NixAdapter,
    pkg: &str,
) -> Result<(root_nix::FlakeMetadata, String)> {
    let flake = adapter
        .flake_metadata("nixpkgs")
        .map_err(|e| anyhow::anyhow!(e))?;
    let locked_ref = flake
        .locked_url
        .clone()
        .or_else(|| {
            flake
                .rev
                .as_ref()
                .map(|rev| format!("github:NixOS/nixpkgs/{}", rev))
        })
        .unwrap_or_else(|| "nixpkgs".to_string());
    Ok((flake, format!("{}#{}", locked_ref, pkg)))
}

fn deterministic_package_from_resolution(
    canonical_name: &str,
    requested: &str,
    installable: &str,
    resolution: &root_nix::LockedPackageResolution,
) -> Result<LockedPackageV2> {
    let version = resolution
        .metadata
        .version
        .clone()
        .or_else(|| {
            resolution.metadata.name.as_ref().and_then(|name| {
                name.strip_prefix(&format!("{}-", canonical_name))
                    .map(|value| value.to_string())
            })
        })
        .unwrap_or_else(|| "unknown".to_string());

    let mut outputs = BTreeMap::new();
    let mut store_paths = BTreeMap::new();
    for output in &resolution.outputs {
        let path = output.path.to_string_lossy().to_string();
        if path.ends_with(".drv") {
            return Err(anyhow::anyhow!(
                "Root resolved a derivation path but no realized output path for {}. \
                 Expected an output store path, got: {}",
                canonical_name,
                path
            ));
        }
        let path_info = resolution
            .path_info
            .iter()
            .find(|info| info.path == output.path);
        outputs.insert(
            output.output_name.clone(),
            LockedPackageOutput {
                store_path: path.clone(),
                content_hash: None,
                nar_hash: path_info.and_then(|info| info.nar_hash.clone()),
                references: path_info
                    .map(|info| {
                        info.references
                            .iter()
                            .map(|reference| reference.to_string_lossy().to_string())
                            .collect()
                    })
                    .unwrap_or_default(),
            },
        );
        store_paths.insert(output.output_name.clone(), path);
    }

    let primary_store_path = store_paths
        .get("out")
        .cloned()
        .or_else(|| store_paths.values().next().cloned())
        .unwrap_or_default();

    let mut meta = BTreeMap::new();
    if let Some(name) = &resolution.metadata.name {
        meta.insert("name".to_string(), serde_json::Value::String(name.clone()));
    }
    if let Some(description) = &resolution.metadata.description {
        meta.insert(
            "description".to_string(),
            serde_json::Value::String(description.clone()),
        );
    }

    let binaries = resolve_package(canonical_name)
        .map(|spec| {
            spec.binaries
                .iter()
                .map(|binary| (*binary).to_string())
                .collect()
        })
        .unwrap_or_else(|| vec![canonical_name.to_string()]);

    let mut package = LockedPackageV2 {
        name: canonical_name.to_string(),
        requested: requested.to_string(),
        version,
        attribute: canonical_name.to_string(),
        store_path: primary_store_path,
        binaries,
        installable: Some(installable.to_string()),
        flake_attribute: Some(canonical_name.to_string()),
        drv_path: Some(
            resolution
                .derivation
                .derivation_path
                .to_string_lossy()
                .to_string(),
        ),
        outputs,
        store_paths,
        meta,
        content_hash: None,
    };

    let hash_input = serde_json::to_vec(&package).unwrap_or_default();
    package.content_hash = Some(root_lockfile::compute_sha256(&hash_input));
    Ok(package)
}

fn legacy_package_from_v2(package: &LockedPackageV2) -> LockedPackage {
    LockedPackage {
        name: package.name.clone(),
        requested: package.requested.clone(),
        version: package.version.clone(),
        attribute: package.attribute.clone(),
        store_path: package.store_path.clone(),
        binaries: package.binaries.clone(),
    }
}

fn build_v2_lock(
    old_lock: &RootLock,
    flake: &root_nix::FlakeMetadata,
    packages: Vec<LockedPackageV2>,
) -> Result<RootLockV2> {
    let root_dir = get_root_dir()?;
    let now = chrono::Utc::now().to_rfc3339();
    Ok(RootLockV2 {
        version: ROOT_LOCK_SCHEMA_VERSION,
        root_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        created_at: Some(now.clone()),
        updated_at: Some(now),
        platform: old_lock.platform.clone(),
        nix: NixRuntime {
            version: None,
            system: Some(old_lock.platform.clone()),
            store_dir: Some("/nix/store".to_string()),
            sandbox: None,
        },
        nixpkgs: NixpkgsConfigV2 {
            rev: flake
                .rev
                .clone()
                .unwrap_or_else(|| old_lock.nixpkgs.rev.clone()),
            source: flake.original_url.clone(),
            flake_ref: flake.locked_url.clone(),
            nar_hash: flake.nar_hash.clone(),
            last_modified: flake.last_modified.map(|value| value.to_string()),
            system: Some(old_lock.platform.clone()),
            config: BTreeMap::new(),
            overlays: Vec::new(),
        },
        profile: LockProfile {
            name: "default".to_string(),
            path: Some(
                root_dir
                    .join("profiles")
                    .join("default")
                    .to_string_lossy()
                    .to_string(),
            ),
            generation: None,
        },
        packages,
    })
}

fn verify_profile_contains_outputs(
    adapter: &impl NixAdapter,
    outputs: &BTreeMap<String, String>,
) -> Result<()> {
    for store_path in outputs.values() {
        if store_path.ends_with(".drv") {
            return Err(anyhow::anyhow!(
                "Root resolved a derivation path but no realized output path. \
                 Refusing to verify .drv path as an installed output: {}",
                store_path
            ));
        }
    }
    let profile_json = adapter
        .profile_list_json()
        .map_err(|e| anyhow::anyhow!(e))?;
    for store_path in outputs.values() {
        if !profile_json.contains(store_path) {
            return Err(anyhow::anyhow!(
                "Installed profile did not contain locked Nix store path {}",
                store_path
            ));
        }
    }
    Ok(())
}

fn parse_attributes(search_output: &str) -> Vec<String> {
    search_output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("* nixpkgs#") {
                rest.split_whitespace().next().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn search_match_for_package(query: &str, spec: &'static PackageSpec) -> Option<SearchMatch> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return None;
    }

    let mut matched_fields = Vec::new();
    if spec.name.to_lowercase().contains(&query) {
        matched_fields.push("name".to_string());
    }
    if spec
        .aliases
        .iter()
        .any(|alias| alias.to_lowercase().contains(&query))
    {
        matched_fields.push("alias".to_string());
    }
    if spec.category.to_lowercase().contains(&query) {
        matched_fields.push("category".to_string());
    }
    if spec.description.to_lowercase().contains(&query) {
        matched_fields.push("description".to_string());
    }
    if spec.nix_attr.to_lowercase().contains(&query) {
        matched_fields.push("nix_attr".to_string());
    }
    if spec
        .binaries
        .iter()
        .any(|binary| binary.to_lowercase().contains(&query))
    {
        matched_fields.push("binary".to_string());
    }

    if matched_fields.is_empty() {
        return None;
    }

    Some(SearchMatch {
        name: spec.name,
        aliases: spec
            .aliases
            .iter()
            .map(|alias| (*alias).to_string())
            .collect(),
        category: spec.category,
        description: spec.description,
        nix_attr: spec.nix_attr,
        binaries: spec
            .binaries
            .iter()
            .map(|binary| (*binary).to_string())
            .collect(),
        verify: spec
            .verify
            .iter()
            .map(|verify| format!("{} {}", verify.binary, verify.args.join(" ")))
            .collect(),
        matched_fields,
    })
}

fn search_rank(query: &str, package: &SearchMatch) -> u8 {
    let query = query.trim().to_lowercase();
    if package.name.eq_ignore_ascii_case(&query) {
        return 0;
    }
    if package
        .aliases
        .iter()
        .any(|alias| alias.eq_ignore_ascii_case(&query))
    {
        return 1;
    }
    if package
        .binaries
        .iter()
        .any(|binary| binary.eq_ignore_ascii_case(&query))
    {
        return 2;
    }
    if package.name.to_lowercase().contains(&query) {
        return 3;
    }
    if package
        .aliases
        .iter()
        .any(|alias| alias.to_lowercase().contains(&query))
    {
        return 4;
    }
    if package
        .binaries
        .iter()
        .any(|binary| binary.to_lowercase().contains(&query))
    {
        return 5;
    }
    if package.category.to_lowercase().contains(&query) {
        return 6;
    }
    if package.nix_attr.to_lowercase().contains(&query) {
        return 7;
    }
    8
}

pub fn search(query: &str) -> SearchOutput {
    let mut matches: Vec<SearchMatch> = SUPPORTED_PACKAGES
        .iter()
        .filter_map(|spec| search_match_for_package(query, spec))
        .collect();
    matches.sort_by(|a, b| {
        search_rank(query, a)
            .cmp(&search_rank(query, b))
            .then_with(|| a.name.cmp(b.name))
    });

    SearchOutput {
        query: query.to_string(),
        matches,
        supported_count: SUPPORTED_PACKAGES.len(),
    }
}

pub fn plan(adapter: &impl NixAdapter, pkg: &str) -> Result<PlanReport> {
    let spec = resolve_package(pkg).ok_or_else(|| {
        anyhow::anyhow!(
            "Root does not support \"{}\" yet.\n\nSupported packages:\n{}\n\nMore packages are coming soon.",
            pkg,
            format_supported_packages()
        )
    })?;
    let canonical = spec.name;
    let original_input = if canonical != pkg {
        Some(pkg.to_string())
    } else {
        None
    };
    match adapter.search(canonical) {
        Ok(description) => {
            let attributes = parse_attributes(&description);
            Ok(PlanReport {
                package: canonical.to_string(),
                original_input,
                found: true,
                description,
                would_create_snapshot: true,
                attributes,
                nix_attr: spec.nix_attr.to_string(),
                expected_binaries: spec.binaries.iter().map(|b| (*b).to_string()).collect(),
                verify_commands: spec
                    .verify
                    .iter()
                    .map(|v| format!("{} {}", v.binary, v.args.join(" ")))
                    .collect(),
                rollback_available: true,
            })
        }
        Err(root_nix::NixError::NotFound(_)) => Ok(PlanReport {
            package: canonical.to_string(),
            original_input,
            found: false,
            description: String::new(),
            would_create_snapshot: true,
            attributes: Vec::new(),
            nix_attr: spec.nix_attr.to_string(),
            expected_binaries: spec.binaries.iter().map(|b| (*b).to_string()).collect(),
            verify_commands: spec
                .verify
                .iter()
                .map(|v| format!("{} {}", v.binary, v.args.join(" ")))
                .collect(),
            rollback_available: true,
        }),
        Err(e) => Err(anyhow::anyhow!(e)),
    }
}

pub fn install(adapter: &impl NixAdapter, pkg: &str) -> Result<InstallReport> {
    let spec = resolve_package(pkg).ok_or_else(|| {
        anyhow::anyhow!(
            "Root does not support \"{}\" yet.\n\nSupported packages:\n{}\n\nMore packages are coming soon.",
            pkg,
            format_supported_packages()
        )
    })?;
    let canonical = spec.name;
    let original = pkg;
    enforce_policy(policy::PolicyAction::Install, Some(canonical))?;
    let _guard = MutationGuard::acquire()?;
    let lock = get_or_create_lock_v2()?;
    let before_packages: Vec<String> = lock.packages.iter().map(|p| p.name.clone()).collect();

    let (flake, installable) = locked_installable_for(adapter, canonical)?;
    let resolution = adapter
        .resolve_locked_package(canonical, Some(&installable))
        .map_err(|e| anyhow::anyhow!(e))?;
    let locked_package =
        deterministic_package_from_resolution(canonical, original, &installable, &resolution)?;

    let snapshot = Snapshot::create_from_v2(&format!("before install {}", canonical), &lock)?;
    let snapshot_id = snapshot.id.clone();

    adapter
        .install_installable(canonical, &installable)
        .map_err(|e| anyhow::anyhow!(e))?;
    verify_profile_contains_outputs(adapter, &locked_package.store_paths)?;

    let mut rootfile = get_or_create_rootfile()?;
    rootfile
        .packages
        .insert(canonical.to_string(), locked_package.version.clone());
    save_rootfile(&rootfile)?;

    let mut v2_packages: Vec<LockedPackageV2> = lock
        .packages
        .iter()
        .filter(|package| package.name != canonical)
        .cloned()
        .collect();
    v2_packages.push(locked_package.clone());
    let legacy_lock = legacy_lock_from_v2(&lock);
    let v2_lock = build_v2_lock(&legacy_lock, &flake, v2_packages)?;
    save_lock_v2(&v2_lock)?;

    let _ = events::record_event(
        events::RootEventType::Install,
        events::RootEventStatus::Verified,
        &format!("root install {}", canonical),
        Some(canonical.to_string()),
        Some(snapshot_id.clone()),
        None,
        Some("Package installed successfully".to_string()),
    )?;

    let after_packages: Vec<String> = v2_lock.packages.iter().map(|p| p.name.clone()).collect();

    let changed: Vec<String> = after_packages
        .iter()
        .filter(|p| !before_packages.contains(p))
        .cloned()
        .collect();
    let unchanged: Vec<String> = before_packages
        .iter()
        .filter(|p| after_packages.contains(p))
        .cloned()
        .collect();

    Ok(InstallReport {
        success: true,
        operation: "install",
        package: canonical.to_string(),
        changed,
        unchanged,
        snapshot_id,
        rollback_available: true,
        warnings: Vec::new(),
    })
}

fn update_requested_package(
    requested: Option<&str>,
    rootfile: &Rootfile,
) -> Result<(Vec<UpdateTarget>, Vec<String>)> {
    if let Some(pkg) = requested {
        let spec = resolve_package(pkg).ok_or_else(|| {
            anyhow::anyhow!(
                "Root does not support \"{}\" yet.\n\nSupported packages:\n{}\n\nMore packages are coming soon.",
                pkg,
                format_supported_packages()
            )
        })?;
        if !rootfile.packages.contains_key(spec.name) && !rootfile.packages.contains_key(pkg) {
            return Err(anyhow::anyhow!(
                "Package '{}' is not listed in Rootfile.\n\nInstall it first with:  root install {}",
                spec.name,
                spec.name
            ));
        }
        return Ok((vec![(pkg.to_string(), spec)], Vec::new()));
    }

    let mut keys: Vec<&str> = rootfile.packages.keys().map(String::as_str).collect();
    keys.sort_unstable();
    let mut targets = Vec::new();
    let mut skipped = Vec::new();
    let mut seen = std::collections::BTreeSet::new();

    for key in keys {
        if let Some(spec) = resolve_package(key) {
            if seen.insert(spec.name) {
                targets.push((key.to_string(), spec));
            }
        } else {
            skipped.push(key.to_string());
        }
    }

    Ok((targets, skipped))
}

pub fn update(adapter: &impl NixAdapter, pkg: Option<&str>) -> Result<UpdateReport> {
    root_lockfile::init_root_dir()?;
    let rootfile = get_or_create_rootfile()?;
    let (targets, skipped) = update_requested_package(pkg, &rootfile)?;

    if targets.is_empty() {
        return Ok(UpdateReport {
            success: true,
            requested: pkg.map(str::to_string),
            updated: Vec::new(),
            unchanged: Vec::new(),
            skipped,
            failed: Vec::new(),
            snapshot_id: None,
            warnings: vec!["No supported packages found in Rootfile.".to_string()],
        });
    }

    for (_, spec) in &targets {
        enforce_policy(policy::PolicyAction::Update, Some(spec.name))?;
    }
    let _guard = MutationGuard::acquire()?;
    let current_lock = get_or_create_lock_v2()?;

    let snapshot = Snapshot::create_from_v2(
        &match pkg {
            Some(pkg) => format!("before update {}", pkg),
            None => "before update all".to_string(),
        },
        &current_lock,
    )?;
    let snapshot_id = snapshot.id.clone();

    let mut updated = Vec::new();
    let mut unchanged = Vec::new();
    let mut resolved_packages = Vec::new();
    let mut flake_for_lock = None;

    for (requested_key, spec) in &targets {
        let canonical = spec.name;
        let old_package = current_lock
            .packages
            .iter()
            .find(|package| package.name == canonical);
        let requested_name = old_package
            .map(|package| package.requested.as_str())
            .unwrap_or(requested_key.as_str());

        let (flake, installable) = locked_installable_for(adapter, canonical)?;
        let resolution = adapter
            .resolve_locked_package(canonical, Some(&installable))
            .map_err(|e| anyhow::anyhow!(e))?;
        let locked_package = deterministic_package_from_resolution(
            canonical,
            requested_name,
            &installable,
            &resolution,
        )?;

        if old_package
            .map(|old_package| locked_package_changed(old_package, &locked_package))
            .unwrap_or(true)
        {
            adapter.remove(canonical).map_err(|e| anyhow::anyhow!(e))?;
            adapter
                .install_installable(canonical, &installable)
                .map_err(|e| anyhow::anyhow!(e))?;
            verify_profile_contains_outputs(adapter, &locked_package.store_paths)?;
            updated.push(canonical.to_string());
        } else {
            unchanged.push(canonical.to_string());
        }

        flake_for_lock = Some(flake);
        resolved_packages.push(locked_package);
    }

    let target_names: std::collections::BTreeSet<&str> =
        targets.iter().map(|(_, spec)| spec.name).collect();
    let mut v2_packages: Vec<LockedPackageV2> = current_lock
        .packages
        .iter()
        .filter(|package| !target_names.contains(package.name.as_str()))
        .cloned()
        .collect();
    v2_packages.extend(resolved_packages.clone());

    let flake = match flake_for_lock {
        Some(flake) => flake,
        None => adapter
            .flake_metadata("nixpkgs")
            .map_err(|e| anyhow::anyhow!(e))?,
    };
    let legacy_lock = legacy_lock_from_v2(&current_lock);
    let new_lock = build_v2_lock(&legacy_lock, &flake, v2_packages)?;

    let mut next_rootfile = rootfile;
    for package in &resolved_packages {
        next_rootfile.packages.remove(&package.requested);
        next_rootfile
            .packages
            .insert(package.name.clone(), package.version.clone());
    }
    save_rootfile(&next_rootfile)?;
    save_lock_v2(&new_lock)?;

    let message = format!(
        "Updated: {}. Unchanged: {}. Skipped: {}.",
        updated.join(", "),
        unchanged.join(", "),
        skipped.join(", ")
    );
    let _ = events::record_event(
        events::RootEventType::Update,
        events::RootEventStatus::Completed,
        &match pkg {
            Some(pkg) => format!("root update {}", pkg),
            None => "root update".to_string(),
        },
        if targets.len() == 1 {
            Some(targets[0].1.name.to_string())
        } else {
            None
        },
        Some(snapshot_id.clone()),
        None,
        Some(message),
    )?;

    Ok(UpdateReport {
        success: true,
        requested: pkg.map(str::to_string),
        updated,
        unchanged,
        skipped,
        failed: Vec::new(),
        snapshot_id: Some(snapshot_id),
        warnings: Vec::new(),
    })
}

pub fn list(adapter: &impl NixAdapter) -> Result<ListOutput> {
    root_lockfile::init_root_dir()?;
    let rootfile = get_or_create_rootfile()?;
    let packages: Vec<ListPackage> = rootfile
        .packages
        .into_iter()
        .map(|(name, version)| ListPackage { name, version })
        .collect();

    let nix_profile = adapter.list().map_err(|e| anyhow::anyhow!(e))?;

    Ok(ListOutput {
        packages,
        nix_profile,
    })
}

pub fn remove(adapter: &impl NixAdapter, pkg: &str) -> Result<RemoveReport> {
    root_lockfile::init_root_dir()?;
    enforce_policy(policy::PolicyAction::Remove, Some(pkg))?;
    let _guard = MutationGuard::acquire()?;
    let mut lock = get_or_create_lock_v2()?;

    // Create snapshot before mutation
    let snapshot = Snapshot::create_from_v2(&format!("before remove {}", pkg), &lock)?;
    let snapshot_id = snapshot.id.clone();

    adapter.remove(pkg).map_err(|e| anyhow::anyhow!(e))?;

    let mut rootfile = get_or_create_rootfile()?;
    rootfile.packages.remove(pkg);
    save_rootfile(&rootfile)?;

    lock.packages.retain(|p| p.name != pkg);
    lock.updated_at = Some(chrono::Utc::now().to_rfc3339());
    save_lock_v2(&lock)?;

    let _ = events::record_event(
        events::RootEventType::Remove,
        events::RootEventStatus::Completed,
        &format!("root remove {}", pkg),
        Some(pkg.to_string()),
        Some(snapshot_id.clone()),
        None,
        Some("Package removed successfully".to_string()),
    )?;

    Ok(RemoveReport {
        success: true,
        package: pkg.to_string(),
        snapshot_id,
        rollback_available: true,
    })
}

pub fn history() -> Result<HistoryOutput> {
    root_lockfile::init_root_dir()?;
    Ok(HistoryOutput {
        snapshots: list_snapshot_summaries()?,
        events: events::read_events()?,
    })
}

pub fn rollback_last(adapter: &impl NixAdapter) -> Result<RollbackReport> {
    root_lockfile::init_root_dir()?;
    let _guard = MutationGuard::acquire()?;
    let snaps = list_snapshots()?;
    if snaps.is_empty() {
        return Err(anyhow::anyhow!("No snapshots available for rollback."));
    }

    let last_snap = &snaps[0];
    let current_lock = get_or_create_lock_v2()?;
    let target_lock = last_snap.restored_lock();

    // Step 1: Compute rollback plan
    let mut packages_to_remove = Vec::new();
    for curr_pkg in &current_lock.packages {
        let target_pkg = target_lock
            .packages
            .iter()
            .find(|package| package.name == curr_pkg.name);
        if target_pkg
            .map(|target_pkg| locked_package_changed(curr_pkg, target_pkg))
            .unwrap_or(true)
        {
            packages_to_remove.push(curr_pkg.name.clone());
        }
    }

    let mut packages_to_install = Vec::new();
    for target_pkg in &target_lock.packages {
        let current_pkg = current_lock
            .packages
            .iter()
            .find(|package| package.name == target_pkg.name);
        if current_pkg
            .map(|current_pkg| locked_package_changed(current_pkg, target_pkg))
            .unwrap_or(true)
        {
            packages_to_install.push(target_pkg.name.clone());
        }
    }

    // Step 2: Create a pre-rollback snapshot (for safety)
    let pre_rollback_snap = root_snapshot::Snapshot::create_from_v2(
        &format!("before rollback to {}", last_snap.id),
        &current_lock,
    )?;

    // Step 3: Execute Nix profile changes FIRST
    for pkg in &packages_to_remove {
        adapter.remove(pkg).map_err(|e| {
            let _ = events::record_event(
                events::RootEventType::Rollback,
                events::RootEventStatus::Failed,
                "root rollback --last",
                None,
                Some(pre_rollback_snap.id.clone()),
                Some(last_snap.id.clone()),
                Some(format!("Failed to remove package '{}': {}", pkg, e)),
            );
            anyhow::anyhow!("Rollback failed to remove '{}': {}", pkg, e)
        })?;
    }

    for pkg in &packages_to_install {
        let locked_pkg = target_lock
            .packages
            .iter()
            .find(|package| package.name == *pkg)
            .ok_or_else(|| {
                anyhow::anyhow!("Snapshot package '{}' is missing lock metadata", pkg)
            })?;
        let install_result = if let Some(installable) = locked_pkg.installable.as_deref() {
            adapter.install_installable(pkg, installable)
        } else {
            adapter.install(pkg)
        };
        install_result.map_err(|e| {
            let _ = events::record_event(
                events::RootEventType::Rollback,
                events::RootEventStatus::Failed,
                "root rollback --last",
                None,
                Some(pre_rollback_snap.id.clone()),
                Some(last_snap.id.clone()),
                Some(format!("Failed to install package '{}': {}", pkg, e)),
            );
            anyhow::anyhow!("Rollback failed to install '{}': {}", pkg, e)
        })?;

        // Verify profile contains the locked store paths from the snapshot
        verify_profile_contains_outputs(adapter, &locked_pkg.store_paths).map_err(|e| {
            let _ = events::record_event(
                events::RootEventType::Rollback,
                events::RootEventStatus::Failed,
                "root rollback --last",
                None,
                Some(pre_rollback_snap.id.clone()),
                Some(last_snap.id.clone()),
                Some(format!("Rollback verification failed for '{}': {}", pkg, e)),
            );
            anyhow::anyhow!("Rollback verification failed for '{}': {}", pkg, e)
        })?;
    }

    // Step 4: ONLY NOW update Rootfile and root.lock (after Nix succeeded)
    let mut rootfile = get_or_create_rootfile()?;
    rootfile.packages.clear();

    for pkg in &target_lock.packages {
        rootfile
            .packages
            .insert(pkg.name.clone(), pkg.version.clone());
    }

    save_rootfile(&rootfile)?;
    save_lock_v2(&target_lock)?;

    // Step 5: Record rollback event
    let _ = events::record_event(
        events::RootEventType::Rollback,
        events::RootEventStatus::Completed,
        "root rollback --last",
        None,
        Some(pre_rollback_snap.id.clone()),
        Some(last_snap.id.clone()),
        Some(format!(
            "Removed: {}. Restored: {}.",
            packages_to_remove.join(", "),
            packages_to_install.join(", ")
        )),
    )?;

    Ok(RollbackReport {
        success: true,
        from_snapshot: last_snap.id.clone(),
        packages_removed: packages_to_remove,
        packages_restored: packages_to_install,
    })
}

pub fn doctor(adapter: &impl NixAdapter) -> Result<root_doctor::DoctorReport> {
    let report = root_doctor::run_diagnostics(adapter)?;
    if get_root_dir().map(|path| path.exists()).unwrap_or(false) {
        let _ = events::record_event(
            events::RootEventType::Doctor,
            events::RootEventStatus::Completed,
            "root doctor",
            None,
            None,
            None,
            if report.healthy {
                Some("System healthy".to_string())
            } else {
                Some(format!("Issues found: {}", report.issues.len()))
            },
        );
    }
    Ok(report)
}

pub fn verify(pkg: &str) -> Result<root_verify::VerificationReport> {
    match root_verify::verify_package(pkg) {
        Ok(report) => {
            let event_type = if report.success {
                events::RootEventType::Verification
            } else {
                events::RootEventType::VerificationFailed
            };
            let status = if report.success {
                events::RootEventStatus::Verified
            } else {
                events::RootEventStatus::Failed
            };
            let message = if report.success {
                Some(format!("Verified package '{}'.", pkg))
            } else {
                Some(format!(
                    "Verification failed for '{}': {}",
                    pkg,
                    report.errors.join("; ")
                ))
            };
            let _ = events::record_event(
                event_type,
                status,
                &format!("root verify {}", pkg),
                Some(pkg.to_string()),
                None,
                None,
                message,
            );
            Ok(report)
        }
        Err(err) => {
            let _ = events::record_event(
                events::RootEventType::VerificationFailed,
                events::RootEventStatus::Failed,
                &format!("root verify {}", pkg),
                Some(pkg.to_string()),
                None,
                None,
                Some(err.to_string()),
            );
            Err(err)
        }
    }
}

pub fn import_brew(dest_dir: &Path) -> Result<brew::BrewImportReport> {
    brew::import_brew(dest_dir)
}

fn parse_nix_profile_packages(list_output: &str) -> Vec<String> {
    list_output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            // Match mock adapter format: "Index: 0 - nixpkgs#poppler"
            if let Some(idx) = line.find("nixpkgs#") {
                let pkg = line[idx + 8..].trim().to_string();
                if !pkg.is_empty() {
                    return Some(pkg);
                }
            }
            None
        })
        .collect()
}

#[derive(Debug, Serialize)]
pub struct LockReport {
    pub success: bool,
    pub packages_locked: Vec<String>,
    pub packages_removed: Vec<String>,
    pub snapshot_id: Option<String>,
}

pub fn lock(adapter: &impl NixAdapter) -> Result<LockReport> {
    root_lockfile::init_root_dir()?;
    let _guard = MutationGuard::acquire()?;
    let rootfile = get_or_create_rootfile()?;
    let old_lock = get_or_create_lock()?;

    // Snapshot existing lockfile state before overwriting
    let snapshot_id = {
        let lock_path = get_root_dir()?.join("root.lock");
        if lock_path.exists() {
            let old_v2_lock = get_or_create_lock_v2()?;
            let snapshot = Snapshot::create_from_v2("before lock", &old_v2_lock)?;
            Some(snapshot.id)
        } else {
            None
        }
    };

    let mut packages_locked = Vec::new();
    let mut packages_removed = Vec::new();
    let mut v2_packages = Vec::new();
    let mut flake_for_lock = None;

    // Build a deterministic lock from Rootfile intent by resolving Nix metadata.
    for name in rootfile.packages.keys() {
        let (flake, installable) = locked_installable_for(adapter, name)?;
        let resolution = adapter
            .resolve_locked_package(name, Some(&installable))
            .map_err(|e| anyhow::anyhow!(e))?;
        let locked_package =
            deterministic_package_from_resolution(name, name, &installable, &resolution)?;
        flake_for_lock = Some(flake);
        packages_locked.push(name.clone());
        v2_packages.push(locked_package);
    }

    // Detect packages that were in old lock but not in new lock
    for old_pkg in &old_lock.packages {
        if !v2_packages.iter().any(|p| p.name == old_pkg.name) {
            packages_removed.push(old_pkg.name.clone());
        }
    }

    let flake = match flake_for_lock {
        Some(flake) => flake,
        None => adapter
            .flake_metadata("nixpkgs")
            .map_err(|e| anyhow::anyhow!(e))?,
    };
    let new_lock = build_v2_lock(&old_lock, &flake, v2_packages)?;
    save_lock_v2(&new_lock)?;

    Ok(LockReport {
        success: true,
        packages_locked,
        packages_removed,
        snapshot_id,
    })
}

#[derive(Debug, Serialize)]
pub struct SyncReport {
    pub success: bool,
    pub installed: Vec<String>,
    pub removed: Vec<String>,
    pub unchanged: Vec<String>,
    pub snapshot_id: String,
}

#[derive(Debug, Serialize)]
pub struct SandboxCreateReport {
    pub success: bool,
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct SandboxRunReport {
    pub success: bool,
    pub sandbox_id: String,
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Serialize)]
pub struct SandboxListReport {
    pub success: bool,
    pub sandboxes: Vec<root_sandbox::SandboxInstance>,
}

#[derive(Debug, Serialize)]
pub struct SandboxDestroyReport {
    pub success: bool,
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct DriftIssue {
    pub category: String,
    pub description: String,
    pub suggestion: String,
}

#[derive(Debug, Serialize)]
pub struct StatusReport {
    pub success: bool,
    pub healthy: bool,
    pub state: String,
    pub rootfile_packages: usize,
    pub lockfile_packages: usize,
    pub profile_packages: usize,
    pub machine_id: String,
    pub hostname: String,
    pub drift_details: Vec<DriftIssue>,
}

#[derive(Debug, Serialize)]
pub struct RestoreReport {
    pub success: bool,
    pub lock_path: String,
    pub installed: Vec<String>,
    pub removed: Vec<String>,
    pub unchanged: Vec<String>,
    pub snapshot_id: String,
}

#[derive(Debug, Clone)]
struct ProfilePackageEntry {
    package: String,
    store_paths: Vec<String>,
}

#[derive(Debug)]
struct ProfileReconcileReport {
    installed: Vec<String>,
    removed: Vec<String>,
    unchanged: Vec<String>,
    snapshot_id: String,
}

#[derive(Debug, Serialize)]
pub struct CatalogEntry {
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub nix_attr: &'static str,
    pub binaries: Vec<String>,
    pub aliases: Vec<String>,
    pub verify: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CatalogOutput {
    pub packages: Vec<CatalogEntry>,
}

pub fn catalog() -> CatalogOutput {
    CatalogOutput {
        packages: SUPPORTED_PACKAGES
            .iter()
            .map(|p| CatalogEntry {
                name: p.name,
                description: p.description,
                category: p.category,
                nix_attr: p.nix_attr,
                binaries: p.binaries.iter().map(|b| (*b).to_string()).collect(),
                aliases: p.aliases.iter().map(|a| (*a).to_string()).collect(),
                verify: p
                    .verify
                    .iter()
                    .map(|v| format!("{} {}", v.binary, v.args.join(" ")))
                    .collect(),
            })
            .collect(),
    }
}

fn get_or_create_machine_id() -> Result<String> {
    let dir = root_lockfile::get_root_dir()?;
    let path = dir.join("machine.json");
    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("HOST"))
        .unwrap_or_else(|_| "unknown".to_string());
    let now = chrono::Utc::now().to_rfc3339();

    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;
        let id = json["machine_id"].as_str().unwrap_or("unknown").to_string();
        let mut json = json;
        json["last_seen"] = serde_json::Value::String(now);
        if json.get("hostname").and_then(|v| v.as_str()) != Some(&hostname) {
            json["hostname"] = serde_json::Value::String(hostname.clone());
        }
        std::fs::write(&path, serde_json::to_string_pretty(&json)?)?;
        Ok(id)
    } else {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let id = format!("root-{:016x}", seed as u64);
        let json = serde_json::json!({
            "machine_id": id,
            "hostname": hostname,
            "platform": root_lockfile::detect_platform().unwrap_or_else(|_| "unknown".to_string()),
            "first_seen": now,
            "last_seen": now,
        });
        std::fs::write(&path, serde_json::to_string_pretty(&json)?)?;
        Ok(id)
    }
}

fn format_supported_packages() -> String {
    let mut lines = Vec::new();
    let mut categories: Vec<(&str, Vec<&PackageSpec>)> = Vec::new();
    for pkg in SUPPORTED_PACKAGES {
        let idx = categories.iter().position(|(c, _)| *c == pkg.category);
        if let Some(idx) = idx {
            categories[idx].1.push(pkg);
        } else {
            categories.push((pkg.category, vec![pkg]));
        }
    }
    for (category, pkgs) in &categories {
        lines.push(format!("  {}:", category));
        for pkg in pkgs {
            lines.push(format!("    {:<12} {}", pkg.name, pkg.description));
        }
    }
    lines.join("\n")
}

fn package_name_from_installable(installable: &str) -> String {
    installable
        .rsplit_once('#')
        .map(|(_, package)| package)
        .unwrap_or(installable)
        .to_string()
}

fn parse_profile_package_entries_from_json(profile_json: &str) -> Vec<ProfilePackageEntry> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(profile_json) else {
        return Vec::new();
    };

    let values: Vec<&serde_json::Value> = match &value {
        serde_json::Value::Array(entries) => entries.iter().collect(),
        serde_json::Value::Object(map) => {
            if let Some(elements) = map.get("elements").and_then(|value| value.as_array()) {
                elements.iter().collect()
            } else {
                map.values().collect()
            }
        }
        _ => Vec::new(),
    };

    values
        .into_iter()
        .filter_map(|value| {
            let object = value.as_object()?;
            let installable = object
                .get("installable")
                .or_else(|| object.get("originalUrl"))
                .or_else(|| object.get("original_url"))
                .and_then(|value| value.as_str());
            let attr_path = object
                .get("attrPath")
                .or_else(|| object.get("attr_path"))
                .and_then(|value| value.as_str());
            let package = attr_path
                .filter(|value| !value.is_empty())
                .map(|value| value.rsplit('.').next().unwrap_or(value).to_string())
                .or_else(|| installable.map(package_name_from_installable))?;
            let store_paths = object
                .get("storePaths")
                .or_else(|| object.get("store_paths"))
                .and_then(|value| value.as_array())
                .map(|values| {
                    values
                        .iter()
                        .filter_map(|value| value.as_str().map(ToString::to_string))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            Some(ProfilePackageEntry {
                package,
                store_paths,
            })
        })
        .collect()
}

fn profile_packages(adapter: &impl NixAdapter) -> Result<Vec<ProfilePackageEntry>> {
    let json_entries = adapter
        .profile_list_json()
        .map(|json| parse_profile_package_entries_from_json(&json))
        .map_err(|e| anyhow::anyhow!(e))?;
    if !json_entries.is_empty() {
        return Ok(json_entries);
    }

    let nix_list = adapter.list().map_err(|e| anyhow::anyhow!(e))?;
    Ok(parse_nix_profile_packages(&nix_list)
        .iter()
        .map(|package| ProfilePackageEntry {
            package: package.clone(),
            store_paths: Vec::new(),
        })
        .collect())
}

fn locked_package_installed(
    profile_entries: &[ProfilePackageEntry],
    locked_package: &LockedPackageV2,
) -> bool {
    profile_entries.iter().any(|entry| {
        if entry.package != locked_package.name {
            return false;
        }
        if locked_package.store_paths.is_empty() {
            return true;
        }
        locked_package
            .store_paths
            .values()
            .all(|store_path| entry.store_paths.iter().any(|path| path == store_path))
    })
}

fn write_rootfile_from_v2_lock(lock: &RootLockV2) -> Result<()> {
    let mut rootfile = get_or_create_rootfile()?;
    rootfile.packages.clear();
    for package in &lock.packages {
        rootfile
            .packages
            .insert(package.name.clone(), package.version.clone());
    }
    save_rootfile(&rootfile)
}

fn reconcile_profile_to_lock(
    adapter: &impl NixAdapter,
    target_lock: &RootLockV2,
    snapshot_reason: &str,
    command: &str,
    event_type: events::RootEventType,
) -> Result<ProfileReconcileReport> {
    let current_lock = get_or_create_lock_v2()?;
    let snapshot = Snapshot::create_from_v2(snapshot_reason, &current_lock)?;
    let snapshot_id = snapshot.id.clone();

    let profile_entries = profile_packages(adapter)?;
    let locked_names: std::collections::BTreeSet<&str> = target_lock
        .packages
        .iter()
        .map(|package| package.name.as_str())
        .collect();
    let mut installed = Vec::new();
    let mut unchanged = Vec::new();

    for package in &target_lock.packages {
        if locked_package_installed(&profile_entries, package) {
            unchanged.push(package.name.clone());
            continue;
        }

        let install_result = if let Some(installable) = package.installable.as_deref() {
            adapter.install_installable(&package.name, installable)
        } else {
            adapter.install(&package.name)
        };
        install_result.map_err(|e| {
            let _ = events::record_event(
                event_type.clone(),
                events::RootEventStatus::Failed,
                command,
                Some(package.name.clone()),
                Some(snapshot_id.clone()),
                None,
                Some(format!(
                    "Failed to install package '{}': {}",
                    package.name, e
                )),
            );
            anyhow::anyhow!("{} failed to install '{}': {}", command, package.name, e)
        })?;

        verify_profile_contains_outputs(adapter, &package.store_paths).map_err(|e| {
            let _ = events::record_event(
                event_type.clone(),
                events::RootEventStatus::Failed,
                command,
                Some(package.name.clone()),
                Some(snapshot_id.clone()),
                None,
                Some(format!(
                    "Profile verification failed for '{}': {}",
                    package.name, e
                )),
            );
            anyhow::anyhow!(
                "{} verification failed for '{}': {}",
                command,
                package.name,
                e
            )
        })?;
        installed.push(package.name.clone());
    }

    let mut removed = Vec::new();
    for entry in &profile_entries {
        if !locked_names.contains(entry.package.as_str()) {
            adapter.remove(&entry.package).map_err(|e| {
                let _ = events::record_event(
                    event_type.clone(),
                    events::RootEventStatus::Failed,
                    command,
                    Some(entry.package.clone()),
                    Some(snapshot_id.clone()),
                    None,
                    Some(format!(
                        "Failed to remove package '{}': {}",
                        entry.package, e
                    )),
                );
                anyhow::anyhow!("{} failed to remove '{}': {}", command, entry.package, e)
            })?;
            removed.push(entry.package.clone());
        }
    }

    save_lock_v2(target_lock)?;
    write_rootfile_from_v2_lock(target_lock)?;

    let _ = events::record_event(
        event_type,
        events::RootEventStatus::Completed,
        command,
        None,
        Some(snapshot_id.clone()),
        None,
        Some(format!(
            "Installed: {}. Removed: {}. Unchanged: {}.",
            installed.join(", "),
            removed.join(", "),
            unchanged.join(", ")
        )),
    )?;

    Ok(ProfileReconcileReport {
        installed,
        removed,
        unchanged,
        snapshot_id,
    })
}

pub fn sync(adapter: &impl NixAdapter) -> Result<SyncReport> {
    root_lockfile::init_root_dir()?;
    enforce_policy(policy::PolicyAction::Sync, None)?;
    let policy_lock = get_or_create_lock_v2()?;
    for package in &policy_lock.packages {
        enforce_policy(policy::PolicyAction::Sync, Some(&package.name))?;
    }
    let _guard = MutationGuard::acquire()?;

    let root_dir = get_root_dir()?;
    let lock_path = root_dir.join("root.lock");
    if lock_path.exists() {
        if let Ok(v2_lock) = RootLockV2::read_from_file(&lock_path) {
            if v2_lock.version < ROOT_LOCK_SCHEMA_VERSION {
                return sync_legacy_lock(adapter);
            }
        }
    }

    let lock = get_or_create_lock_v2()?;
    let report = reconcile_profile_to_lock(
        adapter,
        &lock,
        "before sync",
        "root sync",
        events::RootEventType::Update,
    )?;

    Ok(SyncReport {
        success: true,
        installed: report.installed,
        removed: report.removed,
        unchanged: report.unchanged,
        snapshot_id: report.snapshot_id,
    })
}

fn sync_legacy_lock(adapter: &impl NixAdapter) -> Result<SyncReport> {
    let lock = get_or_create_lock()?;
    let nix_list = adapter.list().map_err(|e| anyhow::anyhow!(e))?;
    let profile_pkgs = parse_nix_profile_packages(&nix_list);

    let locked_names: Vec<String> = lock.packages.iter().map(|p| p.name.clone()).collect();

    let to_install: Vec<String> = locked_names
        .iter()
        .filter(|p| !profile_pkgs.contains(p))
        .cloned()
        .collect();
    let to_remove: Vec<String> = profile_pkgs
        .iter()
        .filter(|p| !locked_names.contains(p))
        .cloned()
        .collect();
    let unchanged: Vec<String> = locked_names
        .iter()
        .filter(|p| profile_pkgs.contains(p))
        .cloned()
        .collect();

    let snapshot = Snapshot::create("before sync", &lock)?;
    let snapshot_id = snapshot.id.clone();

    for pkg in &to_install {
        adapter.install(pkg).map_err(|e| anyhow::anyhow!(e))?;
    }
    for pkg in &to_remove {
        adapter.remove(pkg).map_err(|e| anyhow::anyhow!(e))?;
    }

    let mut rootfile = get_or_create_rootfile()?;
    rootfile.packages.clear();
    for pkg in &lock.packages {
        rootfile
            .packages
            .insert(pkg.name.clone(), pkg.version.clone());
    }
    save_rootfile(&rootfile)?;

    let _ = events::record_event(
        events::RootEventType::Update,
        events::RootEventStatus::Completed,
        "root sync",
        None,
        Some(snapshot_id.clone()),
        None,
        Some(format!(
            "Installed: {}. Removed: {}. Unchanged: {}.",
            to_install.join(", "),
            to_remove.join(", "),
            unchanged.join(", ")
        )),
    )?;

    Ok(SyncReport {
        success: true,
        installed: to_install,
        removed: to_remove,
        unchanged,
        snapshot_id,
    })
}

pub fn restore(adapter: &impl NixAdapter, lock_path: Option<&Path>) -> Result<RestoreReport> {
    root_lockfile::init_root_dir()?;
    let selected_lock_path = match lock_path {
        Some(path) => path.to_path_buf(),
        None => get_root_dir()?.join("root.lock"),
    };
    let target_lock = RootLockV2::read_from_file(&selected_lock_path)
        .or_else(|_| RootLock::read_from_file(&selected_lock_path).map(|lock| lock.to_v2()))?;
    enforce_policy(policy::PolicyAction::Restore, None)?;
    for package in &target_lock.packages {
        enforce_policy(policy::PolicyAction::Restore, Some(&package.name))?;
    }
    let _guard = MutationGuard::acquire()?;
    let report = reconcile_profile_to_lock(
        adapter,
        &target_lock,
        &format!(
            "before restore from {}",
            selected_lock_path.to_string_lossy()
        ),
        "root restore",
        events::RootEventType::Restore,
    )?;

    Ok(RestoreReport {
        success: true,
        lock_path: selected_lock_path.to_string_lossy().to_string(),
        installed: report.installed,
        removed: report.removed,
        unchanged: report.unchanged,
        snapshot_id: report.snapshot_id,
    })
}

pub fn sandbox_create(
    provider: &impl SandboxProvider,
    name: Option<&str>,
    image: Option<&str>,
) -> Result<SandboxCreateReport> {
    root_lockfile::init_root_dir()?;
    let sandbox_name = name.unwrap_or("default");
    enforce_policy(policy::PolicyAction::SandboxCreate, Some(sandbox_name))?;

    let available = provider.check_availability()?;
    if !available {
        return Err(anyhow::anyhow!(
            "No sandbox provider is available.\n\n\
             Root requires Docker to create sandboxes.\n\
             Install Docker Desktop from https://docker.com\n\
             Then verify with: docker info"
        ));
    }

    let instance = provider
        .create(sandbox_name, image)
        .map_err(|e| anyhow::anyhow!("Sandbox create failed: {}", e))?;

    let _ = events::record_event(
        events::RootEventType::Sandbox,
        events::RootEventStatus::Completed,
        &format!("root sandbox create {}", sandbox_name),
        None,
        None,
        None,
        Some(format!(
            "Created sandbox '{}' (id: {})",
            instance.name, instance.id
        )),
    );

    Ok(SandboxCreateReport {
        success: true,
        id: instance.id,
        name: instance.name,
        image: instance.image,
        status: instance.status,
        created_at: instance.created_at,
    })
}

pub fn sandbox_run(
    provider: &impl SandboxProvider,
    id: &str,
    command: &[String],
) -> Result<SandboxRunReport> {
    root_lockfile::init_root_dir()?;
    enforce_policy(policy::PolicyAction::SandboxRun, Some(id))?;

    let cmd_str: Vec<&str> = command.iter().map(String::as_str).collect();
    let result = provider
        .run_command(id, &cmd_str)
        .map_err(|e| anyhow::anyhow!("Sandbox exec failed: {}", e))?;

    let status = if result.exit_code == 0 {
        events::RootEventStatus::Completed
    } else {
        events::RootEventStatus::Failed
    };

    let _ = events::record_event(
        events::RootEventType::Sandbox,
        status,
        &format!("root sandbox run {}", id),
        None,
        None,
        None,
        Some(format!(
            "Executed in sandbox '{}': exit code {}",
            id, result.exit_code
        )),
    );

    Ok(SandboxRunReport {
        success: result.exit_code == 0,
        sandbox_id: id.to_string(),
        command: command.join(" "),
        exit_code: result.exit_code,
        stdout: result.stdout,
        stderr: result.stderr,
    })
}

pub fn sandbox_list(provider: &impl SandboxProvider) -> Result<SandboxListReport> {
    root_lockfile::init_root_dir()?;
    let sandboxes = provider
        .list()
        .map_err(|e| anyhow::anyhow!("Sandbox list failed: {}", e))?;
    Ok(SandboxListReport {
        success: true,
        sandboxes,
    })
}

pub fn sandbox_destroy(provider: &impl SandboxProvider, id: &str) -> Result<SandboxDestroyReport> {
    root_lockfile::init_root_dir()?;
    enforce_policy(policy::PolicyAction::SandboxDestroy, Some(id))?;

    provider
        .destroy(id)
        .map_err(|e| anyhow::anyhow!("Sandbox destroy failed: {}", e))?;

    let _ = events::record_event(
        events::RootEventType::Sandbox,
        events::RootEventStatus::Completed,
        &format!("root sandbox destroy {}", id),
        None,
        None,
        None,
        Some(format!("Destroyed sandbox '{}'", id)),
    );

    Ok(SandboxDestroyReport {
        success: true,
        id: id.to_string(),
    })
}

pub fn status(adapter: &impl root_nix::NixAdapter) -> Result<StatusReport> {
    root_lockfile::init_root_dir()?;
    let machine_id = get_or_create_machine_id()?;
    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("HOST"))
        .unwrap_or_else(|_| "unknown".to_string());

    let rootfile = get_or_create_rootfile()?;
    let lock = get_or_create_lock_v2()?;
    let (profile_entries, profile_error) = match profile_packages(adapter) {
        Ok(entries) => (entries, None),
        Err(error) => (Vec::new(), Some(error.to_string())),
    };

    let rootfile_count = rootfile.packages.len();
    let lockfile_count = lock.packages.len();
    let profile_count = profile_entries.len();

    let mut drift_details = Vec::new();
    let mut healthy = true;

    if let Some(error) = profile_error {
        drift_details.push(DriftIssue {
            category: "profile-unavailable".to_string(),
            description: format!("Could not inspect the Root-managed Nix profile: {}", error),
            suggestion: "Run `root doctor` to diagnose Nix and profile availability".to_string(),
        });
        healthy = false;
    }

    // Compare Rootfile vs lockfile
    for pkg_name in rootfile.packages.keys() {
        if !lock.packages.iter().any(|p| p.name == *pkg_name) {
            drift_details.push(DriftIssue {
                category: "rootfile-lockfile-mismatch".to_string(),
                description: format!("Package '{}' is in Rootfile but not in root.lock", pkg_name),
                suggestion: "Run `root lock` to regenerate root.lock from Rootfile intent"
                    .to_string(),
            });
            healthy = false;
        }
    }

    // Compare lockfile vs profile
    for pkg in &lock.packages {
        if !profile_entries.iter().any(|e| e.package == pkg.name) {
            drift_details.push(DriftIssue {
                category: "lockfile-profile-mismatch".to_string(),
                description: format!(
                    "Package '{}' is in root.lock but not in Nix profile",
                    pkg.name
                ),
                suggestion: "Run `root sync` to install the locked package".to_string(),
            });
            healthy = false;
        }
    }

    for entry in &profile_entries {
        if !lock.packages.iter().any(|p| p.name == entry.package) {
            drift_details.push(DriftIssue {
                category: "profile-lockfile-mismatch".to_string(),
                description: format!(
                    "Package '{}' is in Nix profile but not in root.lock",
                    entry.package
                ),
                suggestion: "Run `root sync` to remove the extra package".to_string(),
            });
            healthy = false;
        }
    }

    let state = if healthy {
        "Healthy".to_string()
    } else if drift_details
        .iter()
        .any(|d| d.category == "lockfile-profile-mismatch" || d.category == "profile-unavailable")
    {
        "NeedsAttention".to_string()
    } else {
        "Drifted".to_string()
    };

    Ok(StatusReport {
        success: true,
        healthy,
        state,
        rootfile_packages: rootfile_count,
        lockfile_packages: lockfile_count,
        profile_packages: profile_count,
        machine_id,
        hostname,
        drift_details,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use root_nix::MockNixAdapter;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::sync::Mutex;

    /// Serializes tests that mutate process-global env vars (ROOT_DIR).
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn test_tmp_dir(name: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!("root_test_{}_{}_{}", name, std::process::id(), n))
    }

    fn write_fake_binary(root_dir: &std::path::Path, name: &str, body: &str) {
        let bin_dir = root_dir.join("profiles").join("default").join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        let path = bin_dir.join(name);
        std::fs::write(&path, body).unwrap();
        #[cfg(unix)]
        {
            let mut permissions = std::fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&path, permissions).unwrap();
        }
    }

    #[test]
    fn test_snapshots_and_rollback() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("snapshots");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        // Manually set up an initial package (bypasses allowlist)
        adapter.install("test-pkg-1").unwrap();
        let mut lock = get_or_create_lock().unwrap();
        lock.packages.push(LockedPackage {
            name: "test-pkg-1".into(),
            requested: "test-pkg-1".into(),
            version: "latest".into(),
            attribute: "test-pkg-1".into(),
            store_path: root_lockfile::derive_store_path("test-pkg-1", "latest"),
            binaries: vec!["test-pkg-1".into()],
        });
        save_lock(&lock).unwrap();
        let mut rootfile = get_or_create_rootfile().unwrap();
        rootfile
            .packages
            .insert("test-pkg-1".into(), "latest".into());
        save_rootfile(&rootfile).unwrap();

        // Use core::install for the only supported package
        install(&adapter, "ffmpeg").unwrap();

        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let deterministic_lock = RootLockV2::read_from_file(&lock_path).unwrap();
        let ffmpeg = deterministic_lock
            .packages
            .iter()
            .find(|package| package.name == "ffmpeg")
            .unwrap();
        assert_eq!(deterministic_lock.version, ROOT_LOCK_SCHEMA_VERSION);
        assert_ne!(ffmpeg.version, "latest");
        assert!(ffmpeg.installable.as_deref().unwrap().contains("#ffmpeg"));
        assert!(ffmpeg.drv_path.as_deref().unwrap().ends_with(".drv"));
        assert!(ffmpeg.store_path.starts_with("/nix/store/"));
        assert!(!ffmpeg.has_placeholder_store_path());

        let hist = history().unwrap();
        assert!(hist
            .snapshots
            .iter()
            .any(|snapshot| snapshot.reason == "before install ffmpeg"));
        assert!(hist
            .events
            .iter()
            .any(|e| e.package.as_deref() == Some("ffmpeg")));

        let res = rollback_last(&adapter).unwrap();
        assert!(res.success);

        let rf = get_or_create_rootfile().unwrap();
        // Rollback reverts the ffmpeg install, leaving test-pkg-1
        assert!(rf.packages.contains_key("test-pkg-1"));
        assert!(!rf.packages.contains_key("ffmpeg"));

        let restored_lock = RootLockV2::read_from_file(&lock_path).unwrap();
        assert_eq!(restored_lock.version, ROOT_LOCK_SCHEMA_VERSION);
        assert!(restored_lock
            .packages
            .iter()
            .any(|package| package.name == "test-pkg-1"));
        assert!(!restored_lock
            .packages
            .iter()
            .any(|package| package.name == "ffmpeg"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_remove_preserves_v2_lock_and_records_snapshot() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("remove_v2");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        install(&adapter, "ffmpeg").unwrap();
        let remove_report = remove(&adapter, "ffmpeg").unwrap();
        assert!(remove_report.success);

        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let lock = RootLockV2::read_from_file(&lock_path).unwrap();
        assert_eq!(lock.version, ROOT_LOCK_SCHEMA_VERSION);
        assert!(lock.packages.is_empty());

        let hist = history().unwrap();
        assert!(hist
            .snapshots
            .iter()
            .any(|snapshot| snapshot.id == remove_report.snapshot_id));
        assert!(hist.events.iter().any(|event| {
            event.event_type == events::RootEventType::Remove
                && event.package.as_deref() == Some("ffmpeg")
        }));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_poppler_install_writes_binary_metadata() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("poppler_binaries");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        install(&adapter, "poppler").unwrap();

        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let lock = RootLockV2::read_from_file(&lock_path).unwrap();
        let poppler = lock
            .packages
            .iter()
            .find(|package| package.name == "poppler")
            .unwrap();
        assert_eq!(
            poppler.binaries,
            vec!["pdftotext".to_string(), "pdfinfo".to_string()]
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_verify_records_success_event() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("verify_event");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let root_dir = root_lockfile::init_root_dir().unwrap();
        std::env::set_var(
            "PATH",
            root_dir.join("profiles").join("default").join("bin"),
        );
        write_fake_binary(
            &root_dir,
            "ffmpeg",
            "#!/bin/sh\necho 'ffmpeg version root'\n",
        );

        let mut lock = get_or_create_lock().unwrap();
        lock.packages.push(LockedPackage {
            name: "ffmpeg".into(),
            requested: "ffmpeg".into(),
            version: "7.1".into(),
            attribute: "ffmpeg".into(),
            store_path: root_lockfile::derive_store_path("ffmpeg", "7.1"),
            binaries: vec!["ffmpeg".into()],
        });
        save_lock(&lock).unwrap();

        let report = verify("ffmpeg").unwrap();
        assert!(report.success);

        let hist = history().unwrap();
        assert!(hist.events.iter().any(|event| {
            event.event_type == events::RootEventType::Verification
                && event.status == events::RootEventStatus::Verified
                && event.package.as_deref() == Some("ffmpeg")
        }));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_lock_generates_lockfile() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("lock");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        // Install something via adapter so it's in the profile
        adapter.install("ripgrep").unwrap();

        // Add to Rootfile
        let mut rf = get_or_create_rootfile().unwrap();
        rf.packages.insert("ripgrep".into(), "latest".into());
        save_rootfile(&rf).unwrap();

        let report = lock(&adapter).unwrap();
        assert!(report.success);
        assert!(report.packages_locked.contains(&"ripgrep".to_string()));

        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let deterministic_lock = RootLockV2::read_from_file(&lock_path).unwrap();
        let locked_package = deterministic_lock
            .packages
            .iter()
            .find(|p| p.name == "ripgrep")
            .unwrap();
        assert_eq!(deterministic_lock.version, ROOT_LOCK_SCHEMA_VERSION);
        assert_ne!(deterministic_lock.nixpkgs.rev, "unknown");
        assert_ne!(locked_package.version, "latest");
        assert!(locked_package.store_path.starts_with("/nix/store/"));
        assert!(!locked_package.has_placeholder_store_path());

        let lockfile = get_or_create_lock().unwrap();
        assert!(lockfile.packages.iter().any(|p| p.name == "ripgrep"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_lock_creates_snapshot_before_writing() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("lock_snapshot");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        // First lock on fresh root — no existing lockfile, so no snapshot
        let mut rf = get_or_create_rootfile().unwrap();
        rf.packages.insert("ripgrep".into(), "latest".into());
        save_rootfile(&rf).unwrap();

        let report = lock(&adapter).unwrap();
        assert!(report.success);
        assert!(report.snapshot_id.is_none());

        let before_snaps = list_snapshots().unwrap();
        let before_count = before_snaps.len();

        // Second lock — existing lockfile should be snapshotted before overwrite
        let report2 = lock(&adapter).unwrap();
        assert!(report2.success);
        assert!(
            report2.snapshot_id.is_some(),
            "expected a snapshot when an existing lockfile is present"
        );

        let after_snaps = list_snapshots().unwrap();
        assert_eq!(
            after_snaps.len(),
            before_count + 1,
            "expected exactly one new snapshot"
        );
        assert!(after_snaps.iter().any(|s| s.reason == "before lock"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_sync_reconciles_profile() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("sync");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        // Setup: lock says pkg-a and pkg-b, but profile only has pkg-a + pkg-c
        adapter.install("pkg-a").unwrap();
        adapter.install("pkg-c").unwrap();

        let mut lock = RootLock {
            version: 1,
            platform: root_lockfile::detect_platform().unwrap_or_else(|_| "aarch64-darwin".into()),
            nixpkgs: NixpkgsConfig {
                rev: "some-rev".into(),
                source: "github:NixOS/nixpkgs".into(),
            },
            packages: vec![],
        };
        lock.packages.push(LockedPackage {
            name: "pkg-a".into(),
            requested: "pkg-a".into(),
            version: "latest".into(),
            attribute: "pkg-a".into(),
            store_path: root_lockfile::derive_store_path("pkg-a", "latest"),
            binaries: vec!["pkg-a".into()],
        });
        lock.packages.push(LockedPackage {
            name: "pkg-b".into(),
            requested: "pkg-b".into(),
            version: "latest".into(),
            attribute: "pkg-b".into(),
            store_path: root_lockfile::derive_store_path("pkg-b", "latest"),
            binaries: vec!["pkg-b".into()],
        });
        lock.write_to_file(&root_lockfile::get_root_dir().unwrap().join("root.lock"))
            .unwrap();

        let report = sync(&adapter).unwrap();
        assert!(report.success);
        assert!(report.installed.contains(&"pkg-b".to_string()));
        assert!(report.removed.contains(&"pkg-c".to_string()));
        assert!(report.unchanged.contains(&"pkg-a".to_string()));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_sync_reconciles_v2_lock() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("sync_v2");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let root_dir = root_lockfile::init_root_dir().unwrap();
        let adapter = MockNixAdapter::new(true);

        adapter.install("fd").unwrap();
        let (flake, installable) = locked_installable_for(&adapter, "ripgrep").unwrap();
        let resolution = adapter
            .resolve_locked_package("ripgrep", Some(&installable))
            .unwrap();
        let locked_pkg =
            deterministic_package_from_resolution("ripgrep", "ripgrep", &installable, &resolution)
                .unwrap();
        let v2_lock = build_v2_lock(
            &RootLock {
                version: 1,
                platform: root_lockfile::detect_platform()
                    .unwrap_or_else(|_| "aarch64-darwin".into()),
                nixpkgs: NixpkgsConfig {
                    rev: "some-rev".into(),
                    source: "github:NixOS/nixpkgs".into(),
                },
                packages: vec![],
            },
            &flake,
            vec![locked_pkg],
        )
        .unwrap();
        v2_lock.write_to_file(&root_dir.join("root.lock")).unwrap();

        let report = sync(&adapter).unwrap();
        assert!(report.success);
        assert!(report.installed.contains(&"ripgrep".to_string()));
        assert!(report.removed.contains(&"fd".to_string()));
        assert!(report.unchanged.is_empty());

        let rf = get_or_create_rootfile().unwrap();
        assert!(rf.packages.contains_key("ripgrep"));
        assert!(!rf.packages.contains_key("fd"));

        let hist = history().unwrap();
        assert!(hist.events.iter().any(|event| {
            event.event_type == events::RootEventType::Update
                && event.command == "root sync"
                && event.snapshot_id.as_deref() == Some(report.snapshot_id.as_str())
        }));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_restore_from_shared_v2_lock() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("restore_v2");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let root_dir = root_lockfile::init_root_dir().unwrap();
        let adapter = MockNixAdapter::new(true);

        adapter.install("fd").unwrap();
        let (flake, installable) = locked_installable_for(&adapter, "ripgrep").unwrap();
        let resolution = adapter
            .resolve_locked_package("ripgrep", Some(&installable))
            .unwrap();
        let locked_pkg =
            deterministic_package_from_resolution("ripgrep", "ripgrep", &installable, &resolution)
                .unwrap();
        let shared_lock = build_v2_lock(
            &RootLock {
                version: 1,
                platform: root_lockfile::detect_platform()
                    .unwrap_or_else(|_| "aarch64-darwin".into()),
                nixpkgs: NixpkgsConfig {
                    rev: "some-rev".into(),
                    source: "github:NixOS/nixpkgs".into(),
                },
                packages: vec![],
            },
            &flake,
            vec![locked_pkg],
        )
        .unwrap();
        let shared_lock_path = root_dir.join("shared-root.lock");
        shared_lock.write_to_file(&shared_lock_path).unwrap();

        let report = restore(&adapter, Some(&shared_lock_path)).unwrap();
        assert!(report.success);
        assert_eq!(
            report.lock_path,
            shared_lock_path.to_string_lossy().to_string()
        );
        assert!(report.installed.contains(&"ripgrep".to_string()));
        assert!(report.removed.contains(&"fd".to_string()));

        let active_lock = RootLockV2::read_from_file(&root_dir.join("root.lock")).unwrap();
        assert!(active_lock.packages.iter().any(|p| p.name == "ripgrep"));
        assert!(!active_lock.packages.iter().any(|p| p.name == "fd"));

        let rf = get_or_create_rootfile().unwrap();
        assert!(rf.packages.contains_key("ripgrep"));
        assert!(!rf.packages.contains_key("fd"));

        let hist = history().unwrap();
        assert!(hist.events.iter().any(|event| {
            event.event_type == events::RootEventType::Restore
                && event.command == "root restore"
                && event.snapshot_id.as_deref() == Some(report.snapshot_id.as_str())
        }));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_rollback_v2_verifies_store_paths() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("rollback_v2_verify");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        // Set up an initial v1 package (simulates pre-v1.2 state)
        adapter.install("ripgrep").unwrap();
        let mut lock = get_or_create_lock().unwrap();
        lock.packages.push(LockedPackage {
            name: "ripgrep".into(),
            requested: "ripgrep".into(),
            version: "latest".into(),
            attribute: "ripgrep".into(),
            store_path: root_lockfile::derive_store_path("ripgrep", "latest"),
            binaries: vec!["rg".into()],
        });
        save_lock(&lock).unwrap();
        let mut rootfile = get_or_create_rootfile().unwrap();
        rootfile.packages.insert("ripgrep".into(), "latest".into());
        save_rootfile(&rootfile).unwrap();

        // Install v2 package
        install(&adapter, "ffmpeg").unwrap();

        // Rollback should succeed (profile contains expected paths)
        let res = rollback_last(&adapter).unwrap();
        assert!(res.success);
        assert!(res.packages_removed.contains(&"ffmpeg".to_string()));

        // Verify final state: ripgrep present, ffmpeg absent
        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let restored_lock = RootLockV2::read_from_file(&lock_path).unwrap();
        assert!(restored_lock.packages.iter().any(|p| p.name == "ripgrep"));
        assert!(!restored_lock.packages.iter().any(|p| p.name == "ffmpeg"));

        let rf = get_or_create_rootfile().unwrap();
        assert!(rf.packages.contains_key("ripgrep"));
        assert!(!rf.packages.contains_key("ffmpeg"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_install_on_v1_lock_migrates_to_v2() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("install_migrate_v1");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        // Create a v1 lock with "latest" version
        adapter.install("test-pkg-1").unwrap();
        let mut lock = get_or_create_lock().unwrap();
        lock.packages.push(LockedPackage {
            name: "test-pkg-1".into(),
            requested: "test-pkg-1".into(),
            version: "latest".into(),
            attribute: "test-pkg-1".into(),
            store_path: root_lockfile::derive_store_path("test-pkg-1", "latest"),
            binaries: vec!["test-pkg-1".into()],
        });
        save_lock(&lock).unwrap();

        // Install ffmpeg (v2 deterministically)
        install(&adapter, "ffmpeg").unwrap();

        // Verify lock is now v2 with real metadata
        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let migrated_lock = RootLockV2::read_from_file(&lock_path).unwrap();
        assert_eq!(migrated_lock.version, ROOT_LOCK_SCHEMA_VERSION);

        // Old package preserved (v1 fields carry over; v2-only fields may be empty)
        let old_pkg = migrated_lock
            .packages
            .iter()
            .find(|p| p.name == "test-pkg-1")
            .unwrap();
        assert_eq!(old_pkg.version, "latest");
        assert!(!old_pkg.store_path.is_empty());

        // New package has deterministic metadata
        let new_pkg = migrated_lock
            .packages
            .iter()
            .find(|p| p.name == "ffmpeg")
            .unwrap();
        assert_ne!(new_pkg.version, "latest");
        assert!(new_pkg.store_path.starts_with("/nix/store/"));
        assert!(new_pkg.drv_path.as_deref().unwrap().ends_with(".drv"));
        assert!(!new_pkg.store_paths.is_empty());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_rollback_event_recorded_on_success() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("rollback_event_success");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        install(&adapter, "ffmpeg").unwrap();
        let report = rollback_last(&adapter).unwrap();
        assert!(report.success);

        let hist = history().unwrap();
        assert!(hist.events.iter().any(|event| {
            event.event_type == events::RootEventType::Rollback
                && event.status == events::RootEventStatus::Completed
        }));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_rollback_failure_preserves_lockfile_and_rootfile() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("rollback_failure_preserve");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        install(&adapter, "ffmpeg").unwrap();

        let before_lock = get_or_create_lock_v2().unwrap();
        let before_rootfile = get_or_create_rootfile().unwrap();

        // Break the adapter by making it unavailable
        // We simulate failure by rolling back into a situation where
        // the adapter can't produce required outputs.
        let adapter2 = MockNixAdapter::new(false);

        let result = rollback_last(&adapter2);
        assert!(result.is_err());

        // Lockfile and Rootfile must be unchanged
        let after_lock = get_or_create_lock_v2().unwrap();
        let after_rootfile = get_or_create_rootfile().unwrap();

        assert_eq!(
            serde_json::to_string(&before_lock).unwrap(),
            serde_json::to_string(&after_lock).unwrap(),
            "Lockfile must not change after failed rollback"
        );
        assert_eq!(
            serde_json::to_string(&before_rootfile).unwrap(),
            serde_json::to_string(&after_rootfile).unwrap(),
            "Rootfile must not change after failed rollback"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    // ─── Package catalog tests ────────────────────────────────────────────

    #[test]
    fn test_all_packages_have_unique_names() {
        let mut names = std::collections::HashSet::new();
        for pkg in SUPPORTED_PACKAGES {
            assert!(
                names.insert(pkg.name),
                "Duplicate package name: {}",
                pkg.name
            );
        }
    }

    #[test]
    fn test_all_packages_have_nix_attr() {
        for pkg in SUPPORTED_PACKAGES {
            assert!(
                !pkg.nix_attr.is_empty(),
                "Package {} has empty nix_attr",
                pkg.name
            );
        }
    }

    #[test]
    fn test_all_packages_have_at_least_one_binary() {
        for pkg in SUPPORTED_PACKAGES {
            assert!(
                !pkg.binaries.is_empty(),
                "Package {} has no binaries",
                pkg.name
            );
        }
    }

    #[test]
    fn test_all_packages_have_at_least_one_verify_command() {
        for pkg in SUPPORTED_PACKAGES {
            assert!(
                !pkg.verify.is_empty(),
                "Package {} has no verify commands",
                pkg.name
            );
        }
    }

    #[test]
    fn test_verify_binary_matches_expected_binaries() {
        for pkg in SUPPORTED_PACKAGES {
            for verify_cmd in pkg.verify {
                assert!(
                    pkg.binaries.contains(&verify_cmd.binary),
                    "Package {}: verify binary '{}' is not in expected binaries {:?}",
                    pkg.name,
                    verify_cmd.binary,
                    pkg.binaries
                );
            }
        }
    }

    #[test]
    fn test_aliases_dont_collide_with_package_names() {
        let names: std::collections::HashSet<&str> =
            SUPPORTED_PACKAGES.iter().map(|p| p.name).collect();
        let mut all_aliases = std::collections::HashSet::new();
        for pkg in SUPPORTED_PACKAGES {
            for alias in pkg.aliases {
                assert!(
                    !names.contains(alias),
                    "Alias '{}' collides with package name",
                    alias
                );
                assert!(all_aliases.insert(alias), "Duplicate alias: {}", alias);
            }
        }
    }

    #[test]
    fn test_unsupported_package_is_rejected_before_nix_call() {
        use root_nix::MockNixAdapter;
        let adapter = MockNixAdapter::new(true);
        let err = plan(&adapter, "nonexistent_pkg_xyz").unwrap_err();
        assert!(err.to_string().contains("does not support"));
    }

    #[test]
    fn test_unsupported_install_is_rejected_before_nix_call() {
        use root_nix::MockNixAdapter;
        let adapter = MockNixAdapter::new(true);
        let err = install(&adapter, "nonexistent_pkg_xyz").unwrap_err();
        assert!(err.to_string().contains("does not support"));
    }

    #[test]
    fn test_catalog_includes_all_supported_packages() {
        let output = catalog();
        assert_eq!(output.packages.len(), SUPPORTED_PACKAGES.len());
        for entry in &output.packages {
            assert!(SUPPORTED_PACKAGES.iter().any(|p| p.name == entry.name));
        }
    }

    #[test]
    fn test_catalog_entries_have_required_fields() {
        let output = catalog();
        for entry in &output.packages {
            assert!(!entry.name.is_empty());
            assert!(!entry.description.is_empty());
            assert!(!entry.category.is_empty());
            assert!(!entry.nix_attr.is_empty());
            assert!(!entry.binaries.is_empty());
            assert!(!entry.verify.is_empty());
        }
    }

    #[test]
    fn test_search_by_alias_resolves_canonical_package() {
        let output = search("rg");
        assert_eq!(output.query, "rg");
        let ripgrep = output
            .matches
            .iter()
            .find(|package| package.name == "ripgrep")
            .unwrap();
        assert!(ripgrep.aliases.contains(&"rg".to_string()));
        assert!(ripgrep.matched_fields.contains(&"alias".to_string()));
        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["matches"][0]["name"], "ripgrep");
    }

    #[test]
    fn test_search_matches_category_description_binary_and_nix_attr() {
        let category = search("search");
        assert!(category
            .matches
            .iter()
            .any(|package| package.name == "ripgrep"));

        let description = search("recursive");
        assert!(description
            .matches
            .iter()
            .any(|package| package.name == "ripgrep"));

        let binary = search("pdfinfo");
        assert!(binary
            .matches
            .iter()
            .any(|package| package.name == "poppler"));

        let nix_attr = search("nixpkgs#kubectl");
        assert!(nix_attr
            .matches
            .iter()
            .any(|package| package.name == "kubectl"));
    }

    #[test]
    fn test_search_is_case_insensitive_and_reports_no_matches() {
        let output = search("TERRAFORM");
        assert!(output
            .matches
            .iter()
            .any(|package| package.name == "terraform"));

        let no_match = search("definitely-not-a-root-package");
        assert!(no_match.matches.is_empty());
        assert_eq!(no_match.supported_count, SUPPORTED_PACKAGES.len());
    }

    #[test]
    fn test_plan_with_alias_resolves_to_canonical() {
        use root_nix::MockNixAdapter;
        let adapter = MockNixAdapter::new(true);
        let report = plan(&adapter, "rg").unwrap();
        assert_eq!(report.package, "ripgrep");
        assert_eq!(report.original_input, Some("rg".to_string()));
    }

    #[test]
    fn test_plan_with_canonical_name_has_no_original_input() {
        use root_nix::MockNixAdapter;
        let adapter = MockNixAdapter::new(true);
        let report = plan(&adapter, "ripgrep").unwrap();
        assert_eq!(report.original_input, None);
    }

    #[test]
    fn test_install_with_alias_stores_canonical_name() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("install_alias");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        install(&adapter, "rg").unwrap();

        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let lock = RootLockV2::read_from_file(&lock_path).unwrap();
        let rg_pkg = lock.packages.iter().find(|p| p.name == "ripgrep").unwrap();
        assert_eq!(rg_pkg.name, "ripgrep");
        assert_eq!(rg_pkg.requested, "rg");
        assert_eq!(rg_pkg.attribute, "ripgrep");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_update_with_alias_targets_canonical_package() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("update_alias");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        adapter.install("ripgrep").unwrap();
        let mut lock = get_or_create_lock().unwrap();
        lock.packages.push(LockedPackage {
            name: "ripgrep".into(),
            requested: "ripgrep".into(),
            version: "latest".into(),
            attribute: "ripgrep".into(),
            store_path: root_lockfile::derive_store_path("ripgrep", "latest"),
            binaries: vec!["rg".into()],
        });
        save_lock(&lock).unwrap();
        let mut rootfile = get_or_create_rootfile().unwrap();
        rootfile.packages.insert("ripgrep".into(), "latest".into());
        save_rootfile(&rootfile).unwrap();

        let report = update(&adapter, Some("rg")).unwrap();
        assert!(report.success);
        assert!(report.updated.contains(&"ripgrep".to_string()));
        assert!(report.snapshot_id.is_some());

        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let lock = RootLockV2::read_from_file(&lock_path).unwrap();
        let rg_pkg = lock.packages.iter().find(|p| p.name == "ripgrep").unwrap();
        assert_eq!(rg_pkg.name, "ripgrep");
        assert_eq!(rg_pkg.requested, "ripgrep");
        assert_ne!(rg_pkg.version, "latest");
        assert!(rg_pkg.store_path.starts_with("/nix/store/"));

        let hist = history().unwrap();
        assert!(hist.events.iter().any(|event| {
            event.event_type == events::RootEventType::Update
                && event.package.as_deref() == Some("ripgrep")
        }));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_update_all_packages_from_rootfile() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("update_all");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        let mut rootfile = get_or_create_rootfile().unwrap();
        rootfile.packages.insert("ripgrep".into(), "latest".into());
        rootfile.packages.insert("fd".into(), "latest".into());
        rootfile
            .packages
            .insert("unsupported-local".into(), "latest".into());
        save_rootfile(&rootfile).unwrap();

        let report = update(&adapter, None).unwrap();
        assert!(report.success);
        assert!(report.updated.contains(&"ripgrep".to_string()));
        assert!(report.updated.contains(&"fd".to_string()));
        assert!(report.skipped.contains(&"unsupported-local".to_string()));

        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let lock = RootLockV2::read_from_file(&lock_path).unwrap();
        assert!(lock.packages.iter().any(|p| p.name == "ripgrep"));
        assert!(lock.packages.iter().any(|p| p.name == "fd"));
        assert!(!lock.packages.iter().any(|p| p.name == "unsupported-local"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_update_rejects_unsupported_or_unmanaged_package() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("update_rejects");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        let unsupported = update(&adapter, Some("nonexistent_pkg_xyz")).unwrap_err();
        assert!(unsupported.to_string().contains("does not support"));

        let unmanaged = update(&adapter, Some("ripgrep")).unwrap_err();
        assert!(unmanaged.to_string().contains("not listed in Rootfile"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_install_with_node_alias_stores_nodejs() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("install_node_alias");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        install(&adapter, "node").unwrap();

        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let lock = RootLockV2::read_from_file(&lock_path).unwrap();
        let node_pkg = lock.packages.iter().find(|p| p.name == "nodejs").unwrap();
        assert_eq!(node_pkg.name, "nodejs");
        assert_eq!(node_pkg.requested, "node");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_install_with_make_alias_stores_gnumake() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("install_make_alias");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        install(&adapter, "make").unwrap();

        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let lock = RootLockV2::read_from_file(&lock_path).unwrap();
        let make_pkg = lock.packages.iter().find(|p| p.name == "gnumake").unwrap();
        assert_eq!(make_pkg.name, "gnumake");
        assert_eq!(make_pkg.requested, "make");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_install_with_python_alias_stores_python3() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("install_python_alias");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        install(&adapter, "python").unwrap();

        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let lock = RootLockV2::read_from_file(&lock_path).unwrap();
        let py_pkg = lock.packages.iter().find(|p| p.name == "python3").unwrap();
        assert_eq!(py_pkg.name, "python3");
        assert_eq!(py_pkg.requested, "python");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_plan_with_golang_alias_resolves_to_go() {
        use root_nix::MockNixAdapter;
        let adapter = MockNixAdapter::new(true);
        let report = plan(&adapter, "golang").unwrap();
        assert_eq!(report.package, "go");
        assert_eq!(report.original_input, Some("golang".to_string()));
    }

    #[test]
    fn test_plan_with_postgres_alias_resolves_to_postgresql() {
        use root_nix::MockNixAdapter;
        let adapter = MockNixAdapter::new(true);
        let report = plan(&adapter, "postgres").unwrap();
        assert_eq!(report.package, "postgresql");
        assert_eq!(report.original_input, Some("postgres".to_string()));
    }

    #[test]
    fn test_plan_with_tf_alias_resolves_to_terraform() {
        use root_nix::MockNixAdapter;
        let adapter = MockNixAdapter::new(true);
        let report = plan(&adapter, "tf").unwrap();
        assert_eq!(report.package, "terraform");
        assert_eq!(report.original_input, Some("tf".to_string()));
    }

    #[test]
    fn test_plan_with_kube_alias_resolves_to_kubectl() {
        use root_nix::MockNixAdapter;
        let adapter = MockNixAdapter::new(true);
        let report = plan(&adapter, "kube").unwrap();
        assert_eq!(report.package, "kubectl");
        assert_eq!(report.original_input, Some("kube".to_string()));
    }

    #[test]
    fn test_plan_with_docker_alias_resolves_to_docker_client() {
        use root_nix::MockNixAdapter;
        let adapter = MockNixAdapter::new(true);
        let report = plan(&adapter, "docker").unwrap();
        assert_eq!(report.package, "docker-client");
        assert_eq!(report.original_input, Some("docker".to_string()));
    }

    #[test]
    fn test_plan_with_nvim_alias_resolves_to_neovim() {
        use root_nix::MockNixAdapter;
        let adapter = MockNixAdapter::new(true);
        let report = plan(&adapter, "nvim").unwrap();
        assert_eq!(report.package, "neovim");
        assert_eq!(report.original_input, Some("nvim".to_string()));
    }

    #[test]
    fn test_install_with_golang_alias_stores_go() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("install_golang_alias");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        install(&adapter, "golang").unwrap();

        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let lock = RootLockV2::read_from_file(&lock_path).unwrap();
        let pkg = lock.packages.iter().find(|p| p.name == "go").unwrap();
        assert_eq!(pkg.name, "go");
        assert_eq!(pkg.requested, "golang");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_install_with_docker_alias_stores_docker_client() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("install_docker_alias");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        install(&adapter, "docker").unwrap();

        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let lock = RootLockV2::read_from_file(&lock_path).unwrap();
        let pkg = lock
            .packages
            .iter()
            .find(|p| p.name == "docker-client")
            .unwrap();
        assert_eq!(pkg.name, "docker-client");
        assert_eq!(pkg.requested, "docker");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_install_with_tf_alias_stores_terraform() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("install_tf_alias");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        install(&adapter, "tf").unwrap();

        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let lock = RootLockV2::read_from_file(&lock_path).unwrap();
        let pkg = lock
            .packages
            .iter()
            .find(|p| p.name == "terraform")
            .unwrap();
        assert_eq!(pkg.name, "terraform");
        assert_eq!(pkg.requested, "tf");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_install_with_nvim_alias_stores_neovim() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("install_nvim_alias");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        install(&adapter, "nvim").unwrap();

        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let lock = RootLockV2::read_from_file(&lock_path).unwrap();
        let pkg = lock.packages.iter().find(|p| p.name == "neovim").unwrap();
        assert_eq!(pkg.name, "neovim");
        assert_eq!(pkg.requested, "nvim");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_plan_with_delta_alias_resolves_to_git_delta() {
        use root_nix::MockNixAdapter;
        let adapter = MockNixAdapter::new(true);
        let report = plan(&adapter, "delta").unwrap();
        assert_eq!(report.package, "git-delta");
        assert_eq!(report.original_input, Some("delta".to_string()));
    }

    #[test]
    fn test_plan_with_z_alias_resolves_to_zoxide() {
        use root_nix::MockNixAdapter;
        let adapter = MockNixAdapter::new(true);
        let report = plan(&adapter, "z").unwrap();
        assert_eq!(report.package, "zoxide");
        assert_eq!(report.original_input, Some("z".to_string()));
    }

    #[test]
    fn test_plan_with_lg_alias_resolves_to_lazygit() {
        use root_nix::MockNixAdapter;
        let adapter = MockNixAdapter::new(true);
        let report = plan(&adapter, "lg").unwrap();
        assert_eq!(report.package, "lazygit");
        assert_eq!(report.original_input, Some("lg".to_string()));
    }

    #[test]
    fn test_install_with_delta_alias_stores_git_delta() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("install_delta_alias");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        install(&adapter, "delta").unwrap();

        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let lock = RootLockV2::read_from_file(&lock_path).unwrap();
        let pkg = lock
            .packages
            .iter()
            .find(|p| p.name == "git-delta")
            .unwrap();
        assert_eq!(pkg.name, "git-delta");
        assert_eq!(pkg.requested, "delta");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_install_with_z_alias_stores_zoxide() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("install_z_alias");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        install(&adapter, "z").unwrap();

        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let lock = RootLockV2::read_from_file(&lock_path).unwrap();
        let pkg = lock.packages.iter().find(|p| p.name == "zoxide").unwrap();
        assert_eq!(pkg.name, "zoxide");
        assert_eq!(pkg.requested, "z");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_install_with_lg_alias_stores_lazygit() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("install_lg_alias");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir();
        let adapter = MockNixAdapter::new(true);

        install(&adapter, "lg").unwrap();

        let lock_path = root_lockfile::get_root_dir().unwrap().join("root.lock");
        let lock = RootLockV2::read_from_file(&lock_path).unwrap();
        let pkg = lock.packages.iter().find(|p| p.name == "lazygit").unwrap();
        assert_eq!(pkg.name, "lazygit");
        assert_eq!(pkg.requested, "lg");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_catalog_json_includes_aliases_and_verify() {
        let output = catalog();
        for entry in &output.packages {
            if let Some(spec) = SUPPORTED_PACKAGES.iter().find(|p| p.name == entry.name) {
                assert_eq!(entry.aliases.len(), spec.aliases.len());
                assert_eq!(entry.verify.len(), spec.verify.len());
            }
        }
    }

    #[test]
    fn test_resolve_package_by_name() {
        assert!(resolve_package("ffmpeg").is_some());
        assert!(resolve_package("ripgrep").is_some());
        assert!(resolve_package("jq").is_some());
        assert!(resolve_package("poppler").is_some());
        assert!(resolve_package("fd").is_some());
        assert!(resolve_package("gh").is_some());
        assert!(resolve_package("go").is_some());
        assert!(resolve_package("rustup").is_some());
        assert!(resolve_package("postgresql").is_some());
        assert!(resolve_package("redis").is_some());
        assert!(resolve_package("terraform").is_some());
        assert!(resolve_package("kubectl").is_some());
        assert!(resolve_package("helm").is_some());
        assert!(resolve_package("k9s").is_some());
        assert!(resolve_package("docker-client").is_some());
        assert!(resolve_package("age").is_some());
        assert!(resolve_package("sops").is_some());
        assert!(resolve_package("neovim").is_some());
        assert!(resolve_package("tmux").is_some());
        assert!(resolve_package("git-delta").is_some());
        assert!(resolve_package("zoxide").is_some());
        assert!(resolve_package("direnv").is_some());
        assert!(resolve_package("starship").is_some());
        assert!(resolve_package("lazygit").is_some());
    }

    #[test]
    fn test_resolve_package_by_alias() {
        assert!(resolve_package("rg").is_some(), "Alias 'rg' should resolve");
        assert!(
            resolve_package("make").is_some(),
            "Alias 'make' should resolve"
        );
        assert!(
            resolve_package("node").is_some(),
            "Alias 'node' should resolve"
        );
        assert!(
            resolve_package("python").is_some(),
            "Alias 'python' should resolve"
        );
        assert!(
            resolve_package("golang").is_some(),
            "Alias 'golang' should resolve"
        );
        assert!(
            resolve_package("postgres").is_some(),
            "Alias 'postgres' should resolve"
        );
        assert!(resolve_package("tf").is_some(), "Alias 'tf' should resolve");
        assert!(
            resolve_package("kube").is_some(),
            "Alias 'kube' should resolve"
        );
        assert!(
            resolve_package("docker").is_some(),
            "Alias 'docker' should resolve"
        );
        assert!(
            resolve_package("nvim").is_some(),
            "Alias 'nvim' should resolve"
        );
        assert!(
            resolve_package("delta").is_some(),
            "Alias 'delta' should resolve"
        );
        assert!(resolve_package("z").is_some(), "Alias 'z' should resolve");
        assert!(resolve_package("lg").is_some(), "Alias 'lg' should resolve");
    }

    #[test]
    fn test_error_message_shows_categories() {
        use root_nix::MockNixAdapter;
        let adapter = MockNixAdapter::new(true);
        let err = plan(&adapter, "nonexistent_pkg_xyz").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("media:"));
        assert!(msg.contains("search:"));
        assert!(msg.contains("dev:"));
        assert!(msg.contains("net:"));
        assert!(msg.contains("language:"));
        assert!(msg.contains("database:"));
        assert!(msg.contains("infrastructure:"));
        assert!(msg.contains("security:"));
        assert!(msg.contains("editor:"));
        assert!(msg.contains("terminal:"));
        assert!(msg.contains("git:"));
    }

    #[test]
    fn test_deterministic_package_stores_drv_only_in_drv_path() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("drv_isolation");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir().unwrap();

        let adapter = MockNixAdapter::new(true);
        adapter.install("ffmpeg").unwrap();
        let (_flake, installable) = locked_installable_for(&adapter, "ffmpeg").unwrap();
        let resolution = adapter
            .resolve_locked_package("ffmpeg", Some(&installable))
            .unwrap();

        let locked =
            deterministic_package_from_resolution("ffmpeg", "ffmpeg", &installable, &resolution)
                .unwrap();

        // drv_path must end in .drv
        assert!(
            locked.drv_path.as_ref().unwrap().ends_with(".drv"),
            "drv_path should end in .drv: {:?}",
            locked.drv_path
        );

        // store_path must NOT end in .drv
        assert!(
            !locked.store_path.ends_with(".drv"),
            "store_path should not end in .drv: {}",
            locked.store_path
        );

        // All store_paths values must NOT end in .drv
        for (output_name, path) in &locked.store_paths {
            assert!(
                !path.ends_with(".drv"),
                "store_paths[{}] should not end in .drv: {}",
                output_name,
                path
            );
        }

        // All outputs store_path values must NOT end in .drv
        for (output_name, output) in &locked.outputs {
            assert!(
                !output.store_path.ends_with(".drv"),
                "outputs[{}].store_path should not end in .drv: {}",
                output_name,
                output.store_path
            );
        }

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_deterministic_package_rejects_drv_output_path() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("drv_reject");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir().unwrap();

        // Build a resolution where outputs contain a .drv path
        let drv_output = root_nix::BuildOutputPath {
            output_name: "out".to_string(),
            path: std::path::PathBuf::from("/nix/store/abc-ffmpeg-8.1.drv"),
        };
        let resolution = root_nix::LockedPackageResolution {
            package: "ffmpeg".to_string(),
            installable: "nixpkgs#ffmpeg".to_string(),
            metadata: root_nix::PackageMetadata {
                package: "ffmpeg".to_string(),
                installable: "nixpkgs#ffmpeg".to_string(),
                name: Some("ffmpeg-8.1".to_string()),
                version: Some("8.1".to_string()),
                description: None,
                raw_json: "{}".to_string(),
            },
            derivation: root_nix::DerivationInfo {
                package: "ffmpeg".to_string(),
                installable: "nixpkgs#ffmpeg".to_string(),
                derivation_path: std::path::PathBuf::from("/nix/store/abc-ffmpeg-8.1.drv"),
                output_paths: vec![drv_output.clone()],
            },
            outputs: vec![drv_output],
            path_info: vec![],
        };

        let result = deterministic_package_from_resolution(
            "ffmpeg",
            "ffmpeg",
            "nixpkgs#ffmpeg",
            &resolution,
        );
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("derivation path"));
        assert!(err_msg.contains("ffmpeg"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_verify_profile_rejects_drv_paths() {
        let adapter = MockNixAdapter::new(true);
        adapter.install("ffmpeg").unwrap();

        let mut outputs = BTreeMap::new();
        outputs.insert(
            "out".to_string(),
            "/nix/store/abc-ffmpeg-8.1.drv".to_string(),
        );

        let result = verify_profile_contains_outputs(&adapter, &outputs);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("derivation path"));
        assert!(err_msg.contains(".drv"));
    }

    #[test]
    fn test_verify_profile_succeeds_with_real_output_path() {
        let adapter = MockNixAdapter::new(true);
        adapter.install("ffmpeg").unwrap();

        // Build store_paths that match what the mock's profile_list_json returns.
        // The mock uses mock_store_path(package) for profile entries.
        // We just need to verify that non-.drv paths pass the guard and the
        // profile check. Use the exact path the mock generates for "ffmpeg".
        let mock_path = {
            let token = format!("{:032x}", {
                "ffmpeg".bytes().fold(0xcbf29ce484222325u64, |h, b| {
                    (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
                })
            });
            let version = {
                let n = "ffmpeg".bytes().fold(0xcbf29ce484222325u64, |h, b| {
                    (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
                });
                format!("0.1.{}", n % 1000)
            };
            format!("/nix/store/{}-ffmpeg-{}", token, version)
        };

        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), mock_path);

        let result = verify_profile_contains_outputs(&adapter, &outputs);
        assert!(result.is_ok(), "verify should succeed: {:?}", result);
    }

    #[test]
    fn test_lockfile_drv_and_output_path_separation() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("lockfile_separation");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        let _ = root_lockfile::init_root_dir().unwrap();

        let adapter = MockNixAdapter::new(true);
        adapter.install("ffmpeg").unwrap();
        let (_flake, installable) = locked_installable_for(&adapter, "ffmpeg").unwrap();
        let resolution = adapter
            .resolve_locked_package("ffmpeg", Some(&installable))
            .unwrap();
        let locked =
            deterministic_package_from_resolution("ffmpeg", "ffmpeg", &installable, &resolution)
                .unwrap();

        // Serialize to JSON and verify structure
        let json = serde_json::to_string_pretty(&locked).unwrap();

        // drv_path field must contain .drv
        assert!(
            json.contains("\"drv_path\""),
            "lockfile should have drv_path field"
        );
        let drv_val = locked.drv_path.as_ref().unwrap();
        assert!(drv_val.ends_with(".drv"), "drv_path value must end in .drv");

        // storePath field must NOT contain .drv
        assert!(
            !locked.store_path.ends_with(".drv"),
            "storePath must not end in .drv"
        );

        // storePaths values must NOT contain .drv
        for path in locked.store_paths.values() {
            assert!(!path.ends_with(".drv"), "storePaths must not end in .drv");
        }

        // outputs storePath values must NOT contain .drv
        for output in locked.outputs.values() {
            assert!(
                !output.store_path.ends_with(".drv"),
                "outputs storePath must not end in .drv"
            );
        }

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_run_rootfile_task_with_root_profile_path() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("run_task");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        root_lockfile::init_root_dir().unwrap();
        write_fake_binary(
            &tmp,
            "root-test-tool",
            "#!/bin/sh\nprintf 'from-root-profile\\n'\n",
        );

        let mut rootfile = Rootfile::default();
        rootfile
            .tasks
            .insert("profile-check".to_string(), "root-test-tool".to_string());
        rootfile.write_to_file(&tmp.join("Rootfile")).unwrap();

        let report = run(RunRequest::Task("profile-check".to_string())).unwrap();
        assert!(report.success);
        assert_eq!(report.exit_code, 0);
        assert_eq!(report.stdout, "from-root-profile\n");
        assert_eq!(report.task.as_deref(), Some("profile-check"));

        let events = events::read_events().unwrap();
        assert!(events.iter().any(|event| {
            event.event_type == events::RootEventType::Execution
                && event.task_name.as_deref() == Some("profile-check")
                && event.exit_code == Some(0)
        }));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_run_workflow_file() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("run_workflow");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        root_lockfile::init_root_dir().unwrap();
        let workflow_path = tmp.join("ci.root.toml");
        std::fs::write(
            &workflow_path,
            "version = 1\nname = \"ci\"\ncommand = \"printf workflow-ok\"\n",
        )
        .unwrap();

        let report = run(RunRequest::Workflow(workflow_path)).unwrap();
        assert!(report.success);
        assert_eq!(report.source, "workflow");
        assert_eq!(report.task.as_deref(), Some("ci"));
        assert_eq!(report.stdout, "workflow-ok");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_policy_apply_and_permissions_report() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("policy_apply");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        root_lockfile::init_root_dir().unwrap();
        let source = tmp.join("source-policy.toml");
        std::fs::write(&source, "version = 1\n[execution]\nrun = \"deny\"\n").unwrap();

        let applied = apply_policy(&source).unwrap();
        assert_eq!(applied.version, 1);
        let report = permissions().unwrap();
        assert_eq!(report.source, "configured");
        assert_eq!(report.policy.execution.run, policy::PolicyMode::Deny);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_denied_install_creates_no_snapshot() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("policy_denied_install");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        root_lockfile::init_root_dir().unwrap();
        std::fs::write(
            tmp.join("policy.toml"),
            "version = 1\n[packages]\ninstall = \"deny\"\n",
        )
        .unwrap();

        let adapter = MockNixAdapter::new(true);
        let error = install(&adapter, "ripgrep").unwrap_err();
        assert!(error.to_string().contains("Policy denied install"));
        assert!(list_snapshots().unwrap().is_empty());

        let events = events::read_events().unwrap();
        assert!(events.iter().any(|event| {
            event.event_type == events::RootEventType::Policy
                && event.policy_decision.as_deref() == Some("denied")
        }));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_sandbox_policy_denies_before_provider_mutation() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("sandbox_policy");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        root_lockfile::init_root_dir().unwrap();
        std::fs::write(
            tmp.join("policy.toml"),
            "version = 1\n[sandboxes]\ncreate = \"deny\"\n",
        )
        .unwrap();

        let provider = root_sandbox::MockSandboxProvider::new(true);
        let error = sandbox_create(&provider, Some("blocked"), None).unwrap_err();
        assert!(error.to_string().contains("Policy denied sandbox-create"));
        assert!(provider.list().unwrap().is_empty());

        let events = events::read_events().unwrap();
        assert!(events.iter().any(|event| {
            event.event_type == events::RootEventType::Policy
                && event.policy_decision.as_deref() == Some("denied")
                && event.command == "policy check sandbox-create"
        }));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_status_reports_healthy_and_profile_drift() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("status_drift");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        root_lockfile::init_root_dir().unwrap();
        let adapter = MockNixAdapter::new(true);

        install(&adapter, "ripgrep").unwrap();
        let healthy = status(&adapter).unwrap();
        assert!(healthy.healthy);
        assert_eq!(healthy.state, "Healthy");
        assert!(healthy.drift_details.is_empty());
        assert_ne!(healthy.machine_id, "unknown");

        adapter.remove("ripgrep").unwrap();
        let drifted = status(&adapter).unwrap();
        assert!(!drifted.healthy);
        assert_eq!(drifted.state, "NeedsAttention");
        assert_eq!(drifted.machine_id, healthy.machine_id);
        assert!(drifted.drift_details.iter().any(|issue| {
            issue.category == "lockfile-profile-mismatch" && issue.suggestion.contains("root sync")
        }));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_status_reports_unavailable_profile_as_needs_attention() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("status_unavailable");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        root_lockfile::init_root_dir().unwrap();
        let adapter = MockNixAdapter::new(false);

        let report = status(&adapter).unwrap();
        assert!(!report.healthy);
        assert_eq!(report.state, "NeedsAttention");
        assert!(report.drift_details.iter().any(|issue| {
            issue.category == "profile-unavailable" && issue.suggestion.contains("root doctor")
        }));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_mutation_guard_acquires_and_releases() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("mutation_guard");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        root_lockfile::init_root_dir().unwrap();

        let guard = MutationGuard::acquire().unwrap();
        // Second acquire should fail (already held)
        assert!(MutationGuard::acquire().is_err());
        drop(guard);
        // After release, should be able to acquire again
        let guard2 = MutationGuard::acquire().unwrap();
        drop(guard2);
    }

    #[test]
    fn test_mutation_guard_stale_lock_recovery() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("mutation_stale");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        root_lockfile::init_root_dir().unwrap();

        // Write a stale lock file with a non-existent PID (PID 999999999 likely dead)
        let lock_path = tmp.join("root.lockfile");
        std::fs::write(&lock_path, "999999999\n0\n").unwrap();
        // Acquire should detect stale lock, remove it, and succeed
        let guard = MutationGuard::acquire().unwrap();
        assert!(lock_path.exists());
        // The lock content should contain our actual PID
        let content = std::fs::read_to_string(&lock_path).unwrap();
        let pid_line = content.lines().next().unwrap().trim();
        assert_eq!(pid_line.parse::<u32>().unwrap(), std::process::id());
        drop(guard);
        assert!(!lock_path.exists());
    }

    #[test]
    fn test_mutation_guard_malformed_lock_fails_safely() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let tmp = test_tmp_dir("mutation_malformed");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("ROOT_DIR", &tmp);
        root_lockfile::init_root_dir().unwrap();

        // Write a malformed lock file
        let lock_path = tmp.join("root.lockfile");
        std::fs::write(&lock_path, "not-a-pid\n").unwrap();
        // Acquire should return an error about the malformed lock
        let err = MutationGuard::acquire().unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("lock") || err_msg.contains("stale") || err_msg.contains("manual"),
            "Expected error about lock file, got: {}",
            err_msg
        );
    }
}
