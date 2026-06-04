use clap::{Parser, Subcommand};
use root_nix::RealNixAdapter;
use serde::Serialize;
use std::process;

#[derive(Parser, Debug)]
#[command(
    name = "root",
    about = "Root v0.1.3 - curated package manager for developer CLI tools",
    version
)]
struct Cli {
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize Root directory structure
    Init {
        /// Install Nix automatically if not detected
        #[arg(long)]
        install_nix: bool,
    },
    /// Show the curated package catalog
    Catalog,
    /// Search for a package to install
    Plan {
        #[command(subcommand)]
        subcommand: PlanSubcommands,
    },
    /// Install a package
    Install {
        #[arg(value_name = "PACKAGE")]
        pkg: String,
    },
    /// List managed packages
    List,
    /// Remove a package
    Remove {
        #[arg(value_name = "PACKAGE")]
        pkg: String,
    },
    /// Show snapshot history
    History,
    /// Rollback to the previous state
    Rollback {
        #[arg(long)]
        last: bool,
    },
    /// Check system health and drift
    Doctor {
        /// Exit with a non-zero code if any issue/drift is detected
        #[arg(long)]
        check: bool,
    },
    /// Verify an installed package's binaries are executable
    Verify {
        #[arg(value_name = "PACKAGE")]
        pkg: String,
    },
    /// Import packages from other package managers (e.g., brew)
    Import {
        #[arg(value_name = "SOURCE")]
        source: String,
    },
    /// Regenerate root.lock from current state
    Lock,
    /// Reconcile Nix profile with root.lock
    Sync,
}

#[derive(Subcommand, Debug)]
enum PlanSubcommands {
    /// Show install plan for a package
    Install {
        #[arg(value_name = "PACKAGE")]
        pkg: String,
    },
}

#[derive(Serialize)]
struct GenericOutput {
    success: bool,
    message: String,
}

fn print_json<T: Serialize>(output: &T) {
    println!("{}", serde_json::to_string_pretty(output).unwrap());
}

fn exit_code_for_error(e: &anyhow::Error) -> i32 {
    if let Some(nix_err) = e.downcast_ref::<root_nix::NixError>() {
        return match nix_err {
            root_nix::NixError::NotInstalled => 7,
            root_nix::NixError::NotFound(_) => 3,
            root_nix::NixError::PlatformMissing(_) => 8,
            root_nix::NixError::Generic(_) => 1,
        };
    }
    let msg = format!("{:?}", e);
    if msg.contains("Rollback") || msg.contains("rollback") {
        6
    } else if msg.contains("verification")
        || msg.contains("Verification")
        || msg.contains("root.lock does not exist")
        || msg.contains("is not found in root.lock")
    {
        4
    } else if msg.contains("unsupported import source")
        || msg.contains("Only 'brew' is supported")
        || msg.contains("Root v0.1 does not support")
    {
        2
    } else if msg.contains("Drift") || msg.contains("drift") {
        5
    } else {
        1
    }
}

fn format_user_error(e: &anyhow::Error) -> String {
    if let Some(nix_err) = e.downcast_ref::<root_nix::NixError>() {
        return match nix_err {
            root_nix::NixError::NotInstalled => {
                "Nix is not installed or not available on PATH.\n\n\
                 To install Nix, run:  root init --install-nix\n\
                 Or visit:            https://nixos.org/download/"
                    .to_string()
            }
            root_nix::NixError::NotFound(pkg) => {
                format!(
                    "Package '{}' was not found in nixpkgs.\n\n\
                     This may mean the package name is incorrect or the attribute\n\
                     does not exist on your platform.",
                    pkg
                )
            }
            root_nix::NixError::PlatformMissing(pkg) => {
                format!(
                    "Package '{}' is not available for your platform.\n\n\
                     Try a different package or check nixpkgs for alternatives:\n  nix search nixpkgs {}",
                    pkg, pkg
                )
            }
            root_nix::NixError::Generic(msg) => {
                if msg.contains("error:") || msg.contains("warning:") || msg.contains("access") {
                    format!("Nix operation failed.\n\nDetails: {}", msg)
                } else {
                    format!("Nix operation failed: {}", msg)
                }
            }
        };
    }
    let msg = format!("{}", e);
    if msg.contains("No snapshots") {
        "No snapshots available for rollback.\n\n\
         Snapshots are created automatically before every install or remove.\n\
         Run:  root install ffmpeg\n\
         Then: root rollback --last"
            .to_string()
    } else if msg.contains("root.lock does not exist") {
        "No lockfile found.\n\n\
         Run:  root install <package>\n\
         This will create root.lock with deterministic Nix metadata."
            .to_string()
    } else if msg.contains("is not found in root.lock") {
        format!(
            "{}.\n\n\
             Install it first with:  root install {}",
            msg,
            msg.split('\'').nth(1).unwrap_or("the-package")
        )
    } else if msg.contains("does not support") || msg.contains("not support") {
        msg
    } else if msg.contains("stale lockfile") || msg.contains("lockfile is stale") {
        "The lockfile is stale or from a previous version.\n\n\
         Run:  root lock\n\
         This will regenerate the lockfile with current metadata."
            .to_string()
    } else if msg.contains("v2 lockfiles") || msg.contains("does not support v2") {
        msg
    } else if msg.contains("mutation is in progress") {
        "Another Root operation is in progress.\n\n\
         If no other terminal is running Root, delete the lock:\n  rm ~/.root/root.lockfile\nThen try again."
            .to_string()
    } else {
        msg
    }
}

fn handle_structured<T: Serialize>(
    json: bool,
    res: anyhow::Result<T>,
    human_fn: impl FnOnce(&T) -> String,
) -> Option<T> {
    match res {
        Ok(val) => {
            if json {
                print_json(&val);
            } else {
                println!("{}", human_fn(&val));
            }
            Some(val)
        }
        Err(e) => {
            let code = exit_code_for_error(&e);
            if json {
                print_json(&GenericOutput {
                    success: false,
                    message: format!("{}", e),
                });
            } else {
                eprintln!("Error: {}", format_user_error(&e));
            }
            process::exit(code);
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let profile_path = std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("~"))
        .join(".root")
        .join("profiles")
        .join("default");
    let adapter = RealNixAdapter::new(profile_path);

    match cli.command {
        Commands::Init { install_nix } => {
            let report = handle_structured(cli.json, root_core::init(&adapter), |r| {
                let mut msg = String::from("Root initialized.\n");
                msg.push_str(&format!("\n✓ Root directory created at {}", r.root_dir));
                if r.nix_detected {
                    msg.push_str("\n✓ Nix detected");
                } else {
                    msg.push_str("\n✗ Nix not detected");
                }
                if r.profile_ready {
                    msg.push_str("\n✓ Root profile ready");
                }
                if r.snapshot_enabled {
                    msg.push_str("\n✓ Snapshot system enabled");
                }
                if r.nix_detected {
                    msg.push_str("\n\nRoot is ready.");
                    msg.push_str("\n\nTry Root in 60 seconds:");
                    msg.push_str("\n  root doctor           Check system health");
                    msg.push_str("\n  root install ffmpeg   Install your first package");
                    msg.push_str("\n  root history          View history");
                    msg.push_str("\n  root verify ffmpeg    Verify package");
                    msg.push_str("\n  root rollback --last  Undo the install");
                    msg.push_str("\n\nRun `root catalog` to see all supported packages.");
                }
                msg
            });

            if let Some(mut report) = report {
                if !report.nix_detected {
                    if install_nix {
                        if !cli.json {
                            println!("Installing Nix...");
                        }
                        match root_core::install_nix() {
                            Ok(()) => {
                                report.nix_detected = true;
                                if !cli.json {
                                    println!("✓ Nix installed successfully.");
                                }
                            }
                            Err(e) => {
                                let code = exit_code_for_error(&e);
                                if cli.json {
                                    print_json(&GenericOutput {
                                        success: false,
                                        message: format!("Failed to install Nix: {:?}", e),
                                    });
                                } else {
                                    eprintln!("Error installing Nix: {}", e);
                                }
                                process::exit(code);
                            }
                        }
                    } else if !cli.json {
                        eprintln!("\nNix is required for Root.\n\nTo install Nix, run:\n  root init --install-nix\n\nOr install Nix manually from:\n  https://nixos.org/download/");
                    }
                }
            }
        }
        Commands::Catalog => {
            let output = root_core::catalog();
            if cli.json {
                print_json(&output);
            } else {
                println!("Root supported packages\n");
                let mut categories: std::collections::BTreeMap<
                    &str,
                    Vec<&root_core::CatalogEntry>,
                > = std::collections::BTreeMap::new();
                for pkg in &output.packages {
                    categories.entry(pkg.category).or_default().push(pkg);
                }
                for (category, pkgs) in &categories {
                    println!("{}", category);
                    for pkg in pkgs {
                        println!("  {:<12} {}", pkg.name, pkg.description);
                    }
                    println!();
                }
            }
        }
        Commands::Plan { subcommand } => match subcommand {
            PlanSubcommands::Install { pkg } => {
                let report = handle_structured(cli.json, root_core::plan(&adapter, &pkg), |r| {
                    if r.found {
                        let binaries = r.expected_binaries.join(", ");
                        let title = if let Some(ref input) = r.original_input {
                            format!("Install plan for {} → {}", input, r.package)
                        } else {
                            format!("Install plan for {}", r.package)
                        };
                        format!(
                            "{}\n\
                             \n\
                             Overview\n\
                               Package:     {}\n\
                               Nix attr:    {}\n\
                               Binaries:    {}\n\
                               Verify:      {}\n\
                             \n\
                             Steps that will be performed:\n\
                             1. Supported package check\n\
                             2. Resolve Nix metadata (version, store path, derivation)\n\
                             3. Pin nixpkgs revision in root.lock\n\
                             4. Create pre-install snapshot\n\
                             5. Install to Root-managed profile\n\
                             6. Verify profile contains locked store paths\n\
                             7. Update root.lock with deterministic metadata\n\
                             8. Record history event\n\
                             \n\
                             Rollback available: yes (via root rollback --last)\n\
                             \n\
                             This is a preview. No changes have been made.",
                            title,
                            r.package,
                            r.nix_attr,
                            binaries,
                            r.verify_commands.join(", "),
                        )
                    } else {
                        format!(
                            "Package '{}' was not found in nixpkgs.\n\
                             \n\
                             The package name is on the supported list but Nix could not resolve it.\n\
                             This may mean the attribute does not exist on your platform.",
                            r.package
                        )
                    }
                });
                if let Some(report) = report {
                    if !report.found {
                        process::exit(3);
                    }
                }
            }
        },
        Commands::Install { pkg } => {
            if !cli.json {
                println!("Planning install...");
            }
            let _ = handle_structured(cli.json, root_core::install(&adapter, &pkg), |r| {
                let mut msg = format!("Installed {}.", r.package);
                if !r.changed.is_empty() {
                    msg.push_str(&format!("\nChanged: {}.", r.changed.join(", ")));
                }
                if !r.unchanged.is_empty() {
                    msg.push_str(&format!("\nUnchanged: {}.", r.unchanged.join(", ")));
                }
                msg.push_str(&format!("\nSnapshot saved: {}", r.snapshot_id));
                msg.push_str("\nRollback available with: root rollback --last");
                msg
            });
        }
        Commands::List => match root_core::list(&adapter) {
            Ok(output) => {
                if cli.json {
                    print_json(&output);
                } else {
                    println!("Managed Packages:");
                    if output.packages.is_empty() {
                        println!("  (none)");
                    } else {
                        for pkg in &output.packages {
                            println!("  - {} ({})", pkg.name, pkg.version);
                        }
                    }
                    println!("\nNix Profile State:");
                    println!("{}", output.nix_profile);
                }
            }
            Err(e) => {
                let code = exit_code_for_error(&e);
                if cli.json {
                    print_json(&GenericOutput {
                        success: false,
                        message: format!("{:?}", e),
                    });
                } else {
                    eprintln!("Error: {}", format_user_error(&e));
                }
                process::exit(code);
            }
        },
        Commands::Remove { pkg } => {
            let _ = handle_structured(cli.json, root_core::remove(&adapter, &pkg), |r| {
                let mut msg = format!("Removed {}.", r.package);
                msg.push_str(&format!("\nSnapshot saved: {}", r.snapshot_id));
                msg.push_str("\nRollback available with: root rollback --last");
                msg
            });
        }
        Commands::History => match root_core::history() {
            Ok(output) => {
                if cli.json {
                    print_json(&output);
                } else if output.snapshots.is_empty() && output.events.is_empty() {
                    println!("No Root-managed snapshots yet.");
                    println!();
                    println!("Run `root catalog` to see supported packages.");
                    println!();
                    println!("Try:");
                    println!("  root install ffmpeg");
                } else {
                    println!("Root history");
                    println!();
                    if !output.snapshots.is_empty() {
                        println!("Snapshots");
                        for snapshot in &output.snapshots {
                            println!("  {}", snapshot.created_at);
                            println!("    snapshot: {}", snapshot.id);
                            println!("    reason: {}", snapshot.reason);
                            println!("    packages: {}", snapshot.package_count);
                            println!("    lock hash: {}", snapshot.lock_content_hash);
                            println!();
                        }
                    }
                    if !output.events.is_empty() {
                        println!("Events");
                    }
                    for event in &output.events {
                        let type_str = format!("{:?}", event.event_type).to_lowercase();
                        println!("  {}", event.timestamp);
                        println!("    event: {}", type_str);
                        println!("    status: {:?}", event.status);
                        if let Some(ref pkg) = event.package {
                            println!("    package: {}", pkg);
                        }
                        if let Some(ref sid) = event.snapshot_id {
                            println!("    snapshot: {}", sid);
                        }
                        if let Some(ref rsid) = event.restored_snapshot_id {
                            println!("    restored: {}", rsid);
                        }
                        println!();
                    }
                }
            }
            Err(e) => {
                let code = exit_code_for_error(&e);
                if cli.json {
                    print_json(&GenericOutput {
                        success: false,
                        message: format!("{:?}", e),
                    });
                } else {
                    eprintln!("Error: {}", format_user_error(&e));
                }
                process::exit(code);
            }
        },
        Commands::Rollback { last } => {
            if last {
                let _ = handle_structured(cli.json, root_core::rollback_last(&adapter), |r| {
                    let mut msg = format!("Rolled back to {}.", r.from_snapshot);
                    if !r.packages_removed.is_empty() {
                        msg.push_str(&format!("\nRemoved: {}", r.packages_removed.join(", ")));
                    }
                    if !r.packages_restored.is_empty() {
                        msg.push_str(&format!("\nRestored: {}", r.packages_restored.join(", ")));
                    }
                    msg
                });
            } else {
                if cli.json {
                    print_json(&GenericOutput {
                        success: false,
                        message: "Currently only `root rollback --last` is supported".into(),
                    });
                } else {
                    eprintln!("Error: Currently only `root rollback --last` is supported");
                }
                process::exit(2);
            }
        }
        Commands::Doctor { check } => match root_core::doctor(&adapter) {
            Ok(report) => {
                if cli.json {
                    print_json(&report);
                } else {
                    println!("Root health check\n");
                    if report.issues.is_empty() {
                        println!("✓ Nix available");
                        println!("✓ Root profile ready");
                        println!("✓ Event ledger writable");
                        println!("✓ No issues detected");
                        println!("\nRoot is ready.");
                        println!("\nRun `root catalog` to see all supported packages.");
                        println!("\nNext steps:");
                        println!("  root install ffmpeg    Install your first package");
                        println!("  root history           View snapshot history");
                        println!("  root verify ffmpeg     Verify package binaries");
                        println!("  root rollback --last   Undo the last change");
                    } else {
                        for issue in &report.issues {
                            let icon = match issue.severity {
                                root_doctor::IssueSeverity::Error => "✗",
                                root_doctor::IssueSeverity::Warning => "△",
                            };
                            println!("{} {}: {}", icon, issue.category, issue.description);
                            println!("  Suggestion: {}", issue.suggestion);
                            println!();
                        }
                        if report.healthy {
                            println!("System health: healthy with warnings");
                        } else {
                            println!("System health: UNHEALTHY — run `root sync` to repair");
                        }
                    }
                }

                if check && !report.issues.is_empty() {
                    process::exit(5);
                }
            }
            Err(e) => {
                let code = exit_code_for_error(&e);
                if cli.json {
                    print_json(&GenericOutput {
                        success: false,
                        message: format!("{:?}", e),
                    });
                } else {
                    eprintln!("Error running doctor: {}", format_user_error(&e));
                }
                process::exit(code);
            }
        },
        Commands::Verify { pkg } => {
            match root_core::verify(&pkg) {
                Ok(report) => {
                    if cli.json {
                        print_json(&report);
                    } else {
                        println!("Verifying binaries for package '{}'...", pkg);
                        println!();

                        for bin_res in &report.binaries {
                            if bin_res.success {
                                println!(
                                    "  🟢 {} : Executable (Exit code {})",
                                    bin_res.binary,
                                    bin_res.exit_code.unwrap_or(0)
                                );
                            } else {
                                println!("  🔴 {} : FAILED to execute", bin_res.binary);
                                if let Some(ref err) = bin_res.error_message {
                                    println!("     Error: {}", err);
                                } else {
                                    println!("     Exit code: {}", bin_res.exit_code.unwrap_or(-1));
                                }
                                if !bin_res.stderr.is_empty() {
                                    println!("     Stderr: {}", bin_res.stderr.trim());
                                }
                            }
                            if let Some(ref resolved_path) = bin_res.resolved_path {
                                println!("     Path: {}", resolved_path);
                            }
                            if !bin_res.attempted_args.is_empty() {
                                println!("     Args: {}", bin_res.attempted_args.join(" "));
                            }
                        }
                        println!();

                        if report.success {
                            println!(
                                "Verification SUCCESS: All binaries for '{}' are fully functional.",
                                pkg
                            );
                        } else {
                            println!("Verification FAILED: One or more binaries failed execution checks.");
                        }
                    }

                    if !report.success {
                        process::exit(4);
                    }
                }
                Err(e) => {
                    let code = exit_code_for_error(&e);
                    if cli.json {
                        print_json(&GenericOutput {
                            success: false,
                            message: format!("{:?}", e),
                        });
                    } else {
                        eprintln!("Error running verification: {}", format_user_error(&e));
                    }
                    process::exit(code);
                }
            }
        }
        Commands::Import { source } => {
            if source != "brew" {
                if cli.json {
                    print_json(&GenericOutput {
                        success: false,
                        message: format!("Unsupported import source: {}", source),
                    });
                } else {
                    eprintln!("Error: Only 'brew' is supported as an import source currently.");
                }
                process::exit(2);
            }

            let cur_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            match root_core::import_brew(&cur_dir) {
                Ok(report) => {
                    if cli.json {
                        print_json(&report);
                    } else {
                        println!("Scanning Homebrew environment...");
                        println!();

                        if !report.brew_detected {
                            println!("❌ Homebrew is not detected on this system or not available in PATH.");
                            process::exit(1);
                        }

                        println!("✅ Homebrew detected!");
                        println!("📦 Formulae found (CLI): {}", report.formulae_found);
                        println!("🖥️  Casks found (GUI/ignored): {}", report.casks_found);
                        println!();

                        println!(
                            "Successfully mapped {} formulae to Nixpkgs attributes.",
                            report.formulae_found
                        );
                        println!("Saved candidate import config to: ./Rootfile.import");
                        println!();

                        println!("👉 Next Steps:");
                        println!("   1. Review the generated './Rootfile.import' candidate file.");
                        println!("   2. Rename it to '~/.root/Rootfile' or merge it into your active config.");
                        println!("   3. Run `root install` (without arguments) in the future to reconcile and lock the state.");
                    }
                }
                Err(e) => {
                    let code = exit_code_for_error(&e);
                    if cli.json {
                        print_json(&GenericOutput {
                            success: false,
                            message: format!("{:?}", e),
                        });
                    } else {
                        eprintln!("Error running brew import: {}", format_user_error(&e));
                    }
                    process::exit(code);
                }
            }
        }
        Commands::Lock => {
            let _ = handle_structured(cli.json, root_core::lock(&adapter), |r| {
                let mut msg = String::from("Locked current state.");
                if !r.packages_locked.is_empty() {
                    msg.push_str(&format!(
                        "\nPackages locked: {}.",
                        r.packages_locked.join(", ")
                    ));
                }
                if !r.packages_removed.is_empty() {
                    msg.push_str(&format!(
                        "\nPackages removed from lock: {}.",
                        r.packages_removed.join(", ")
                    ));
                }
                msg
            });
        }
        Commands::Sync => {
            let _ = handle_structured(cli.json, root_core::sync(&adapter), |r| {
                let mut msg = String::from("Synced Nix profile with root.lock.");
                if !r.installed.is_empty() {
                    msg.push_str(&format!("\nInstalled: {}.", r.installed.join(", ")));
                }
                if !r.removed.is_empty() {
                    msg.push_str(&format!("\nRemoved: {}.", r.removed.join(", ")));
                }
                if !r.unchanged.is_empty() {
                    msg.push_str(&format!("\nUnchanged: {}.", r.unchanged.join(", ")));
                }
                msg.push_str(&format!("\nSnapshot saved: {}", r.snapshot_id));
                msg
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
