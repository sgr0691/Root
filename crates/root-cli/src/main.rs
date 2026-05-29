use clap::{Parser, Subcommand};
use root_nix::RealNixAdapter;
use serde::Serialize;
use std::process;

#[derive(Parser, Debug)]
#[command(
    name = "root",
    about = "Root CLI - Deterministic package management via Nix",
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
    } else if msg.contains("verification") || msg.contains("Verification") {
        4
    } else if msg.contains("unsupported import source") || msg.contains("Only 'brew' is supported")
    {
        2
    } else if msg.contains("Drift") || msg.contains("drift") {
        5
    } else {
        1
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
                    message: format!("{:?}", e),
                });
            } else {
                eprintln!("Error: {:?}", e);
            }
            process::exit(code);
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let adapter = RealNixAdapter::new();

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
                    msg.push_str("\n\nNext:");
                    msg.push_str("\n  root install poppler");
                    msg.push_str("\n  root import brew");
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
                                    eprintln!("Error installing Nix: {:?}", e);
                                }
                                process::exit(code);
                            }
                        }
                    } else if !cli.json {
                        eprintln!("\nNix is required for Root.\nRun: root init --install-nix");
                    }
                }
            }
        }
        Commands::Plan { subcommand } => match subcommand {
            PlanSubcommands::Install { pkg } => {
                let report = handle_structured(cli.json, root_core::plan(&adapter, &pkg), |r| {
                    if r.found {
                        let attributes = r.attributes.join(", ");
                        format!(
                            "Install plan for {}\n\nWill add:\n  {} ({})\n\nWill not change:\n  (existing packages)\n\nWould create snapshot before install.\nNo changes made.",
                            r.package,
                            r.package,
                            if attributes.is_empty() { "latest" } else { &attributes }
                        )
                    } else {
                        format!("Package '{}' not found in nixpkgs", r.package)
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
                    eprintln!("Error: {:?}", e);
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
                } else {
                    if output.snapshots.is_empty() {
                        println!("No snapshots found.");
                    } else {
                        println!("Snapshot History:");
                        for snap in &output.snapshots {
                            println!(
                                "  {} | {} | {}",
                                snap.created_at.format("%Y-%m-%d %H:%M:%S"),
                                snap.id,
                                snap.reason
                            );
                        }
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
                    eprintln!("Error: {:?}", e);
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
                        println!("✓ Root profile active");
                        println!("✓ Machine matches root.lock");
                        println!("✓ No PATH conflicts detected");
                        println!("\nYour machine is in sync.");
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

                if (check && !report.issues.is_empty()) || !report.healthy {
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
                    eprintln!("Error running doctor: {:?}", e);
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
                        eprintln!("Error running verification: {:?}", e);
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
                        eprintln!("Error running brew import: {:?}", e);
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
