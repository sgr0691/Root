use serde::{Deserialize, Serialize};

/// A single step in the Root Safe Install Protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolStep {
    pub order: u32,
    pub command: String,
    pub description: String,
    pub required: bool,
}

/// The full Safe Install Protocol definition for AI coding agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProtocol {
    pub name: String,
    pub version: String,
    pub description: String,
    pub steps: Vec<ProtocolStep>,
    pub restrictions: Vec<String>,
}

impl Default for AgentProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentProtocol {
    /// Creates the standard Root Safe Install Protocol (version 1.0.0)
    pub fn new() -> Self {
        Self {
            name: "Root Safe Install Protocol".into(),
            version: "1.0.0".into(),
            description: "Deterministic package management protocol for AI coding agents".into(),
            steps: vec![
                ProtocolStep {
                    order: 1,
                    command: "root doctor --json".into(),
                    description: "Check environment health and detect drift before making changes".into(),
                    required: true,
                },
                ProtocolStep {
                    order: 2,
                    command: "root plan install <package> --json".into(),
                    description: "Search for package availability in nixpkgs before installing".into(),
                    required: true,
                },
                ProtocolStep {
                    order: 3,
                    command: "root install <package> --json".into(),
                    description: "Install the package via Nix with automatic pre-install snapshot".into(),
                    required: true,
                },
                ProtocolStep {
                    order: 4,
                    command: "root verify <package> --json".into(),
                    description: "Verify every installed binary is executable and functional".into(),
                    required: true,
                },
                ProtocolStep {
                    order: 5,
                    command: "root rollback --last --json".into(),
                    description: "Revert to the previous snapshot if verification fails".into(),
                    required: false,
                },
            ],
            restrictions: vec![
                "Never use brew install — Homebrew is non-deterministic and lacks snapshots".into(),
                "Never use curl ... | sh — Piped remote scripts are unverifiable".into(),
                "Never use npm install -g or pip install --user — Global language installs fragment the environment".into(),
                "Never edit PATH manually — Root manages profile activation through Nix".into(),
                "If Root is unavailable, ask the user before falling back to another installer".into(),
            ],
        }
    }

    /// Format the protocol as a Markdown string suitable for embedding in agent instructions
    pub fn to_markdown(&self) -> String {
        let mut md = format!("# {}\n\n", self.name);
        md.push_str(&format!(
            "_Version {}_ — {}\n\n",
            self.version, self.description
        ));
        md.push_str("## Protocol Steps\n\n");
        for step in &self.steps {
            let badge = if step.required {
                "required"
            } else {
                "optional"
            };
            md.push_str(&format!(
                "{}. `{}` — {} _({})_\n",
                step.order, step.command, step.description, badge
            ));
        }
        md.push_str("\n## Restrictions\n\n");
        for r in &self.restrictions {
            md.push_str(&format!("- {}\n", r));
        }
        md
    }

    /// Serialize the protocol to a JSON string
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_default() {
        let protocol = AgentProtocol::new();
        assert_eq!(protocol.name, "Root Safe Install Protocol");
        assert_eq!(protocol.version, "1.0.0");
        assert_eq!(protocol.steps.len(), 5);
        assert_eq!(protocol.restrictions.len(), 5);
    }

    #[test]
    fn test_protocol_steps_ordered() {
        let protocol = AgentProtocol::new();
        for (i, step) in protocol.steps.iter().enumerate() {
            assert_eq!(step.order, (i + 1) as u32);
        }
    }

    #[test]
    fn test_protocol_to_markdown() {
        let protocol = AgentProtocol::new();
        let md = protocol.to_markdown();
        assert!(md.contains("Root Safe Install Protocol"));
        assert!(md.contains("root doctor --json"));
        assert!(md.contains("root rollback --last --json"));
        assert!(md.contains("Never use brew install"));
    }

    #[test]
    fn test_protocol_to_json() {
        let protocol = AgentProtocol::new();
        let json = protocol.to_json();
        assert!(json.contains("\"version\": \"1.0.0\""));
        assert!(json.contains("\"root doctor --json\""));
        let parsed: AgentProtocol = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.steps.len(), 5);
    }
}
