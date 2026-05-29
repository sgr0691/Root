use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct BrewImportReport {
    pub brew_detected: bool,
    pub formulae_found: usize,
    pub casks_found: usize,
    pub imported_packages: Vec<String>,
    pub ignored_casks: Vec<String>,
    pub mapped_packages: HashMap<String, String>,
}

/// A mapping dictionary of common Homebrew formula names to Nixpkgs attributes
fn get_brew_to_nix_mappings() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("pinentry-mac".to_string(), "pinentry".to_string());
    m.insert("gnu-sed".to_string(), "gnused".to_string());
    m.insert("gnu-tar".to_string(), "gnutar".to_string());
    m.insert("openssl@3".to_string(), "openssl".to_string());
    m.insert("openssl@1.1".to_string(), "openssl_1_1".to_string());
    m.insert("node@22".to_string(), "nodejs_22".to_string());
    m.insert("node@20".to_string(), "nodejs_20".to_string());
    m.insert("python@3.12".to_string(), "python312".to_string());
    m.insert("python@3.11".to_string(), "python311".to_string());
    m.insert("postgresql@16".to_string(), "postgresql_16".to_string());
    m.insert("postgresql@15".to_string(), "postgresql_15".to_string());
    m.insert("mysql@8.0".to_string(), "mysql80".to_string());
    m.insert("mysql@8.4".to_string(), "mysql84".to_string());
    m.insert("libtool".to_string(), "libtool".to_string());
    m
}

pub fn detect_brew() -> bool {
    Command::new("brew")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Core mapping and parsing engine, extracted for perfect unit testing.
pub fn parse_and_map_brew_output(
    formulae_out: &str,
    casks_out: &str,
) -> (BrewImportReport, String) {
    let mappings = get_brew_to_nix_mappings();

    let formulae: Vec<String> = formulae_out
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let casks: Vec<String> = casks_out
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let mut mapped_packages = HashMap::new();
    let mut imported_packages = Vec::new();

    for formula in &formulae {
        let nix_attr = mappings
            .get(formula)
            .cloned()
            .unwrap_or_else(|| formula.clone());
        mapped_packages.insert(formula.clone(), nix_attr.clone());
        imported_packages.push(nix_attr);
    }

    imported_packages.sort();

    // Generate Rootfile.import content
    let mut toml_content =
        String::from("# Candidate Rootfile imported from Homebrew\n\n[packages]\n");
    for pkg in &imported_packages {
        toml_content.push_str(&format!("{} = \"latest\"\n", pkg));
    }

    let report = BrewImportReport {
        brew_detected: true,
        formulae_found: formulae.len(),
        casks_found: casks.len(),
        imported_packages,
        ignored_casks: casks,
        mapped_packages,
    };

    (report, toml_content)
}

pub fn import_brew(dest_dir: &Path) -> Result<BrewImportReport> {
    if !detect_brew() {
        return Ok(BrewImportReport {
            brew_detected: false,
            formulae_found: 0,
            casks_found: 0,
            imported_packages: Vec::new(),
            ignored_casks: Vec::new(),
            mapped_packages: HashMap::new(),
        });
    }

    // Run brew list --formula
    let formulae_output = Command::new("brew")
        .args(["list", "--formula"])
        .output()
        .context("Failed to run 'brew list --formula'")?;
    let formulae_out = String::from_utf8_lossy(&formulae_output.stdout);

    // Run brew list --cask
    let casks_output = Command::new("brew")
        .args(["list", "--cask"])
        .output()
        .context("Failed to run 'brew list --cask'")?;
    let casks_out = String::from_utf8_lossy(&casks_output.stdout);

    let (report, toml_content) = parse_and_map_brew_output(&formulae_out, &casks_out);

    // Write Rootfile.import
    let dest_path = dest_dir.join("Rootfile.import");
    fs::write(&dest_path, toml_content).context(format!(
        "Failed to write candidate import Rootfile to {:?}",
        dest_path
    ))?;

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_map_brew_output() {
        let formulae = "poppler\nopenssl@3\ngnu-sed\n";
        let casks = "google-chrome\nvisual-studio-code\n";

        let (report, toml_content) = parse_and_map_brew_output(formulae, casks);

        assert!(report.brew_detected);
        assert_eq!(report.formulae_found, 3);
        assert_eq!(report.casks_found, 2);

        // Mapped attributes
        assert_eq!(report.mapped_packages.get("openssl@3").unwrap(), "openssl");
        assert_eq!(report.mapped_packages.get("gnu-sed").unwrap(), "gnused");
        assert_eq!(report.mapped_packages.get("poppler").unwrap(), "poppler");

        // Verify sorted packages
        assert_eq!(
            report.imported_packages,
            vec!["gnused", "openssl", "poppler"]
        );

        // Ignored casks
        assert_eq!(
            report.ignored_casks,
            vec!["google-chrome", "visual-studio-code"]
        );

        // TOML Verification
        assert!(toml_content.contains("gnused = \"latest\""));
        assert!(toml_content.contains("openssl = \"latest\""));
        assert!(toml_content.contains("poppler = \"latest\""));
    }
}
