use anyhow::Result;
use std::process::Command;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum SandboxError {
    #[error("Sandbox provider is not available: {0}")]
    NotAvailable(String),
    #[error("Sandbox '{0}' not found")]
    NotFound(String),
    #[error("Container '{0}' is not a Root-managed sandbox")]
    NotRootOwned(String),
    #[error("Sandbox operation failed: {0}")]
    Generic(String),
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SandboxInstance {
    pub id: String,
    pub name: String,
    pub status: String,
    pub created_at: String,
    pub image: String,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SandboxExecResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub trait SandboxProvider {
    fn check_availability(&self) -> Result<bool, SandboxError>;
    fn create(&self, name: &str, image: Option<&str>) -> Result<SandboxInstance, SandboxError>;
    fn run_command(&self, id: &str, command: &[&str]) -> Result<SandboxExecResult, SandboxError>;
    fn list(&self) -> Result<Vec<SandboxInstance>, SandboxError>;
    fn destroy(&self, id: &str) -> Result<(), SandboxError>;
}

pub struct RealSandboxProvider;

impl RealSandboxProvider {
    pub fn new() -> Self {
        Self
    }

    fn run_docker(args: &[&str]) -> Result<String, SandboxError> {
        let output = Command::new("docker").args(args).output().map_err(|_| {
            SandboxError::NotAvailable(
                "Docker is not available on PATH. Install Docker Desktop or the Docker CLI.".into(),
            )
        })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(SandboxError::Generic(stderr))
        }
    }
}

impl Default for RealSandboxProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SandboxProvider for RealSandboxProvider {
    fn check_availability(&self) -> Result<bool, SandboxError> {
        match Command::new("docker").arg("info").output() {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    fn create(&self, name: &str, image: Option<&str>) -> Result<SandboxInstance, SandboxError> {
        let image = image.unwrap_or("ubuntu:latest");
        let now = chrono::Utc::now().to_rfc3339();
        let container_name = format!("root-sandbox-{}", name);

        // Remove any existing container with same name
        let _ = Self::run_docker(&["rm", "-f", &container_name]);

        Self::run_docker(&[
            "run",
            "-d",
            "--name",
            &container_name,
            image,
            "sleep",
            "infinity",
        ])?;

        let inspect = Self::run_docker(&["inspect", "--format", "{{.Id}}", &container_name])?;

        Ok(SandboxInstance {
            id: inspect,
            name: container_name,
            status: "running".to_string(),
            created_at: now,
            image: image.to_string(),
        })
    }

    fn run_command(&self, id: &str, command: &[&str]) -> Result<SandboxExecResult, SandboxError> {
        let mut args = vec!["exec", id];
        args.extend(command);

        let output = Command::new("docker")
            .args(&args)
            .output()
            .map_err(|e| SandboxError::Generic(format!("Failed to execute in sandbox: {}", e)))?;

        Ok(SandboxExecResult {
            exit_code: output.status.code().unwrap_or(128),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    fn list(&self) -> Result<Vec<SandboxInstance>, SandboxError> {
        let output = Self::run_docker(&[
            "ps",
            "-a",
            "--filter",
            "name=root-sandbox-",
            "--format",
            "{{.ID}}\t{{.Names}}\t{{.Status}}\t{{.Image}}",
        ])?;

        if output.is_empty() {
            return Ok(Vec::new());
        }

        let mut instances = Vec::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 4 {
                let status = if parts[2].starts_with("Up") {
                    "running".to_string()
                } else {
                    parts[2].to_string()
                };
                instances.push(SandboxInstance {
                    id: parts[0].to_string(),
                    name: parts[1].to_string(),
                    status,
                    created_at: String::new(),
                    image: parts[3].to_string(),
                });
            }
        }

        Ok(instances)
    }

    fn destroy(&self, id: &str) -> Result<(), SandboxError> {
        // Inspect the container to verify it is Root-owned
        let inspect_output = Self::run_docker(&["inspect", "--format", "{{.Name}}", id])
            .map_err(|_| SandboxError::NotFound(id.to_string()))?;
        let container_name = inspect_output.trim_start_matches('/');
        if !container_name.starts_with("root-sandbox-") {
            return Err(SandboxError::NotRootOwned(id.to_string()));
        }
        Self::run_docker(&["rm", "-f", id])?;
        Ok(())
    }
}

pub struct MockSandboxProvider {
    pub available: bool,
    pub sandboxes: Mutex<Vec<SandboxInstance>>,
}

impl MockSandboxProvider {
    pub fn new(available: bool) -> Self {
        Self {
            available,
            sandboxes: Mutex::new(Vec::new()),
        }
    }
}

impl SandboxProvider for MockSandboxProvider {
    fn check_availability(&self) -> Result<bool, SandboxError> {
        Ok(self.available)
    }

    fn create(&self, name: &str, image: Option<&str>) -> Result<SandboxInstance, SandboxError> {
        if !self.available {
            return Err(SandboxError::NotAvailable(
                "Mock provider not available".into(),
            ));
        }

        let now = chrono::Utc::now().to_rfc3339();
        let instance = SandboxInstance {
            id: format!("mock-sandbox-{}", name),
            name: format!("root-sandbox-{}", name),
            status: "running".to_string(),
            created_at: now,
            image: image.unwrap_or("ubuntu:latest").to_string(),
        };

        self.sandboxes.lock().unwrap().push(instance.clone());
        Ok(instance)
    }

    fn run_command(&self, id: &str, command: &[&str]) -> Result<SandboxExecResult, SandboxError> {
        if !self.available {
            return Err(SandboxError::NotAvailable(
                "Mock provider not available".into(),
            ));
        }

        let sandboxes = self.sandboxes.lock().unwrap();
        if !sandboxes.iter().any(|s| s.id == id || s.name == id) {
            return Err(SandboxError::NotFound(id.to_string()));
        }

        Ok(SandboxExecResult {
            exit_code: 0,
            stdout: format!("mock output: {}\n", command.join(" ")),
            stderr: String::new(),
        })
    }

    fn list(&self) -> Result<Vec<SandboxInstance>, SandboxError> {
        if !self.available {
            return Err(SandboxError::NotAvailable(
                "Mock provider not available".into(),
            ));
        }

        let sandboxes = self.sandboxes.lock().unwrap();
        Ok(sandboxes.clone())
    }

    fn destroy(&self, id: &str) -> Result<(), SandboxError> {
        if !self.available {
            return Err(SandboxError::NotAvailable(
                "Mock provider not available".into(),
            ));
        }

        let mut sandboxes = self.sandboxes.lock().unwrap();
        // Find the sandbox first to verify it is Root-owned
        let matching = sandboxes.iter().find(|s| s.id == id || s.name == id);
        match matching {
            None => return Err(SandboxError::NotFound(id.to_string())),
            Some(s) => {
                if !s.name.starts_with("root-sandbox-") {
                    return Err(SandboxError::NotRootOwned(id.to_string()));
                }
            }
        }
        sandboxes.retain(|s| s.id != id && s.name != id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_availability() {
        let mock = MockSandboxProvider::new(true);
        assert!(mock.check_availability().unwrap());

        let mock_unavailable = MockSandboxProvider::new(false);
        assert!(!mock_unavailable.check_availability().unwrap());
    }

    #[test]
    fn test_mock_create_list_destroy() {
        let mock = MockSandboxProvider::new(true);

        let instance = mock.create("test-1", None).unwrap();
        assert_eq!(instance.name, "root-sandbox-test-1");
        assert_eq!(instance.status, "running");

        let instances = mock.list().unwrap();
        assert_eq!(instances.len(), 1);

        mock.destroy(&instance.id).unwrap();
        assert!(mock.list().unwrap().is_empty());
    }

    #[test]
    fn test_mock_run_command() {
        let mock = MockSandboxProvider::new(true);
        let instance = mock.create("test-run", None).unwrap();

        let result = mock.run_command(&instance.id, &["echo", "hello"]).unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("echo hello"));

        let result = mock.run_command(&instance.name, &["ls", "-la"]).unwrap();
        assert!(result.stdout.contains("ls -la"));
    }

    #[test]
    fn test_mock_destroy_not_found() {
        let mock = MockSandboxProvider::new(true);
        let err = mock.destroy("nonexistent").unwrap_err();
        assert!(matches!(err, SandboxError::NotFound(_)));
    }

    #[test]
    fn test_mock_destroy_root_owned_container() {
        let mock = MockSandboxProvider::new(true);
        mock.create("my-sandbox", None).unwrap();
        mock.destroy("root-sandbox-my-sandbox").unwrap();
        assert!(mock.list().unwrap().is_empty());
    }

    #[test]
    fn test_mock_destroy_rejects_non_root_container() {
        let mock = MockSandboxProvider::new(true);
        // Manually insert a container that does not have root-sandbox- prefix
        {
            let mut sandboxes = mock.sandboxes.lock().unwrap();
            sandboxes.push(SandboxInstance {
                id: "ext-123".to_string(),
                name: "external-container".to_string(),
                status: "running".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                image: "ubuntu:latest".to_string(),
            });
        }
        let err = mock.destroy("external-container").unwrap_err();
        assert!(matches!(err, SandboxError::NotRootOwned(_)));
    }

    #[test]
    fn test_mock_destroy_by_id_root_owned() {
        let mock = MockSandboxProvider::new(true);
        let instance = mock.create("test-1", None).unwrap();
        mock.destroy(&instance.id).unwrap();
        assert!(mock.list().unwrap().is_empty());
    }

    #[test]
    fn test_mock_unavailable_errors() {
        let mock = MockSandboxProvider::new(false);

        assert!(mock.create("x", None).is_err());
        assert!(mock.run_command("x", &["echo"]).is_err());
        assert!(mock.list().is_err());
        assert!(mock.destroy("x").is_err());
    }
}
