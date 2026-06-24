use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SandboxState {
    Created,
    Running,
    Completed,
    Failed,
    Destroyed,
}

impl SandboxState {
    fn can_transition_to(&self, target: &SandboxState) -> bool {
        matches!(
            (self, target),
            (SandboxState::Created, SandboxState::Running)
                | (SandboxState::Created, SandboxState::Completed)
                | (SandboxState::Created, SandboxState::Destroyed)
                | (SandboxState::Created, SandboxState::Failed)
                | (SandboxState::Created, SandboxState::Created)
                | (SandboxState::Running, SandboxState::Completed)
                | (SandboxState::Running, SandboxState::Failed)
                | (SandboxState::Running, SandboxState::Destroyed)
                | (SandboxState::Running, SandboxState::Running)
                | (SandboxState::Completed, SandboxState::Destroyed)
                | (SandboxState::Failed, SandboxState::Destroyed)
        )
    }
}

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
    #[error("Docker is not available or the daemon is not running: {0}")]
    DockerUnavailable(String),
    #[error("Failed to pull Docker image: {0}")]
    ImagePullFailed(String),
    #[error("Container failed to start: {0}")]
    ContainerStartupFailed(String),
    #[error("Sandbox command timed out after {0} seconds")]
    TimeoutExceeded(String),
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Cleanup failed: {0}")]
    CleanupFailed(String),
    #[error("Invalid state transition: {0}")]
    LifecycleViolation(String),
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SandboxInstance {
    pub id: String,
    pub name: String,
    pub status: String,
    pub state: SandboxState,
    pub created_at: String,
    pub image: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpus: Option<String>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SandboxExecResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub trait SandboxProvider {
    fn check_availability(&self) -> Result<bool, SandboxError>;
    fn create(
        &self,
        name: &str,
        image: Option<&str>,
        memory: Option<&str>,
        cpus: Option<&str>,
    ) -> Result<SandboxInstance, SandboxError>;
    fn run_command(
        &self,
        id: &str,
        command: &[&str],
        timeout_secs: Option<u64>,
    ) -> Result<SandboxExecResult, SandboxError>;
    fn list(&self) -> Result<Vec<SandboxInstance>, SandboxError>;
    fn destroy(&self, id: &str) -> Result<(), SandboxError>;
    fn check_exists(&self, id: &str) -> Result<bool, SandboxError>;
    fn check_reachable(&self, id: &str) -> Result<bool, SandboxError>;
}

pub struct RealSandboxProvider {
    state: Mutex<HashMap<String, SandboxState>>,
}

impl RealSandboxProvider {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
        }
    }

    fn run_docker(args: &[&str]) -> Result<String, SandboxError> {
        let output = Command::new("docker").args(args).output().map_err(|e| {
            let msg = format!("Docker command failed: {}", e);
            if msg.contains("No such file or directory")
                || msg.contains("program not found")
                || msg.contains("not found")
            {
                SandboxError::DockerUnavailable(
                    "Docker is not available on PATH. Install Docker Desktop or the Docker CLI."
                        .into(),
                )
            } else {
                SandboxError::NotAvailable(msg)
            }
        })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(normalize_docker_error(&stderr))
        }
    }
}

fn normalize_docker_error(stderr: &str) -> SandboxError {
    let lower = stderr.to_lowercase();
    if lower.contains("permission denied") || lower.contains("permission_denied") {
        return SandboxError::PermissionDenied(stderr.to_string());
    }
    if lower.contains("image") && (lower.contains("pull") || lower.contains("not found")) {
        return SandboxError::ImagePullFailed(stderr.to_string());
    }
    if lower.contains("oom") || lower.contains("memory") || lower.contains("cpuset") {
        return SandboxError::ResourceLimitExceeded(stderr.to_string());
    }
    SandboxError::Generic(stderr.to_string())
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

    fn create(
        &self,
        name: &str,
        image: Option<&str>,
        memory: Option<&str>,
        cpus: Option<&str>,
    ) -> Result<SandboxInstance, SandboxError> {
        let image = image.unwrap_or("ubuntu:latest");
        let now = chrono::Utc::now().to_rfc3339();
        let container_name = format!("root-sandbox-{}", name);
        let mem = memory.unwrap_or("2g");
        let cpu = cpus.unwrap_or("2.0");

        let _ = Self::run_docker(&["rm", "-f", &container_name]);

        let args = vec![
            "run",
            "-d",
            "--name",
            &container_name,
            "--memory",
            mem,
            "--cpus",
            cpu,
            image,
            "sleep",
            "infinity",
        ];
        Self::run_docker(&args).map_err(|e| match &e {
            SandboxError::Generic(msg) if msg.contains("image") || msg.contains("pull") => {
                SandboxError::ImagePullFailed(format!(
                    "Failed to pull/start image '{}': {}",
                    image, msg
                ))
            }
            SandboxError::Generic(msg)
                if msg.contains("cannot start") || msg.contains("startup") =>
            {
                SandboxError::ContainerStartupFailed(format!("Container failed to start: {}", msg))
            }
            _ => e,
        })?;

        let inspect = Self::run_docker(&["inspect", "--format", "{{.Id}}", &container_name])?;

        let instance = SandboxInstance {
            id: inspect.clone(),
            name: container_name,
            status: "running".to_string(),
            state: SandboxState::Running,
            created_at: now,
            image: image.to_string(),
            memory: Some(mem.to_string()),
            cpus: Some(cpu.to_string()),
        };

        self.state
            .lock()
            .unwrap()
            .insert(inspect, SandboxState::Running);
        Ok(instance)
    }

    fn run_command(
        &self,
        id: &str,
        command: &[&str],
        timeout_secs: Option<u64>,
    ) -> Result<SandboxExecResult, SandboxError> {
        {
            let state_map = self.state.lock().unwrap();
            if let Some(st) = state_map.get(id) {
                if *st == SandboxState::Destroyed {
                    return Err(SandboxError::LifecycleViolation(format!(
                        "Cannot run command in destroyed sandbox '{}'",
                        id
                    )));
                }
            }
        }

        let timeout = timeout_secs.unwrap_or(300);
        let timeout_str = timeout.to_string();
        let mut all_args: Vec<&str> = vec!["exec"];

        if timeout > 0 {
            all_args.push(id);
            all_args.push("timeout");
            all_args.push(&timeout_str);
        } else {
            all_args.push(id);
        }

        all_args.extend(command.iter());

        let output = Command::new("docker")
            .args(&all_args)
            .output()
            .map_err(|e| SandboxError::Generic(format!("Failed to exec in container: {}", e)))?;

        let exit_code = output.status.code().unwrap_or(128);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if exit_code == 124 {
            Ok(SandboxExecResult {
                exit_code: 124,
                stdout,
                stderr: format!("Command timed out after {} seconds\n{}", timeout, stderr),
            })
        } else {
            Ok(SandboxExecResult {
                exit_code,
                stdout,
                stderr,
            })
        }
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
                let is_running = parts[2].starts_with("Up");
                let status = if is_running {
                    "running".to_string()
                } else {
                    parts[2].to_string()
                };
                let state = if is_running {
                    SandboxState::Running
                } else {
                    SandboxState::Completed
                };
                instances.push(SandboxInstance {
                    id: parts[0].to_string(),
                    name: parts[1].to_string(),
                    status,
                    state,
                    created_at: String::new(),
                    image: parts[3].to_string(),
                    memory: None,
                    cpus: None,
                });
            }
        }

        Ok(instances)
    }

    fn destroy(&self, id: &str) -> Result<(), SandboxError> {
        let state_before = {
            let state_map = self.state.lock().unwrap();
            state_map.get(id).cloned()
        };

        if let Some(ref st) = state_before {
            if *st == SandboxState::Destroyed {
                let _ = Self::run_docker(&["rm", "-f", id]);
                return Err(SandboxError::LifecycleViolation(format!(
                    "Sandbox '{}' is already destroyed",
                    id
                )));
            }
        }

        let destroy_result = (|| -> Result<(), SandboxError> {
            let inspect_output = Self::run_docker(&["inspect", "--format", "{{.Name}}", id])
                .map_err(|_| SandboxError::NotFound(id.to_string()))?;
            let container_name = inspect_output.trim_start_matches('/');
            if !container_name.starts_with("root-sandbox-") {
                return Err(SandboxError::NotRootOwned(id.to_string()));
            }
            Self::run_docker(&["rm", "-f", id])?;
            Ok(())
        })();

        {
            let mut state_map = self.state.lock().unwrap();
            state_map.insert(id.to_string(), SandboxState::Destroyed);
        }

        match destroy_result {
            Ok(()) => Ok(()),
            Err(e) => {
                let _ = Self::run_docker(&["rm", "-f", id]);
                Err(e)
            }
        }
    }

    fn check_exists(&self, id: &str) -> Result<bool, SandboxError> {
        match Self::run_docker(&["inspect", "--format", "{{.Id}}", id]) {
            Ok(_) => Ok(true),
            Err(SandboxError::NotFound(_)) => Ok(false),
            Err(SandboxError::Generic(msg)) if msg.contains("No such object") => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn check_reachable(&self, id: &str) -> Result<bool, SandboxError> {
        match Self::run_docker(&["exec", id, "echo", "reachable"]) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

pub struct MockSandboxProvider {
    pub available: bool,
    pub sandboxes: Mutex<Vec<SandboxInstance>>,
    pub states: Mutex<HashMap<String, SandboxState>>,
    pub cleanup_attempts: Mutex<u32>,
    pub simulate_cleanup_failure: Mutex<bool>,
    pub simulate_timeout: Mutex<bool>,
    pub simulate_destroy_unavailable: Mutex<bool>,
}

impl MockSandboxProvider {
    pub fn new(available: bool) -> Self {
        Self {
            available,
            sandboxes: Mutex::new(Vec::new()),
            states: Mutex::new(HashMap::new()),
            cleanup_attempts: Mutex::new(0),
            simulate_cleanup_failure: Mutex::new(false),
            simulate_timeout: Mutex::new(false),
            simulate_destroy_unavailable: Mutex::new(false),
        }
    }

    fn validate_transition(&self, id: &str, target: &SandboxState) -> Result<(), SandboxError> {
        let states = self.states.lock().unwrap();
        if let Some(current) = states.get(id) {
            if !current.can_transition_to(target) {
                return Err(SandboxError::LifecycleViolation(format!(
                    "Invalid state transition: {:?} -> {:?} for sandbox '{}'",
                    current, target, id
                )));
            }
        }
        Ok(())
    }

    fn set_state(&self, id: &str, target: SandboxState) {
        let mut states = self.states.lock().unwrap();
        states.insert(id.to_string(), target);
    }
}

impl SandboxProvider for MockSandboxProvider {
    fn check_availability(&self) -> Result<bool, SandboxError> {
        Ok(self.available)
    }

    fn create(
        &self,
        name: &str,
        image: Option<&str>,
        memory: Option<&str>,
        cpus: Option<&str>,
    ) -> Result<SandboxInstance, SandboxError> {
        if !self.available {
            return Err(SandboxError::NotAvailable(
                "Mock provider not available".into(),
            ));
        }

        let now = chrono::Utc::now().to_rfc3339();
        let instance = SandboxInstance {
            id: format!("mock-sandbox-{}", name),
            name: format!("root-sandbox-{}", name),
            status: "created".to_string(),
            state: SandboxState::Created,
            created_at: now,
            image: image.unwrap_or("ubuntu:latest").to_string(),
            memory: memory.map(|s| s.to_string()),
            cpus: cpus.map(|s| s.to_string()),
        };

        self.set_state(&instance.id, SandboxState::Created);
        self.set_state(&instance.name, SandboxState::Created);
        self.sandboxes.lock().unwrap().push(instance.clone());
        Ok(instance)
    }

    fn run_command(
        &self,
        id: &str,
        command: &[&str],
        _timeout_secs: Option<u64>,
    ) -> Result<SandboxExecResult, SandboxError> {
        if !self.available {
            return Err(SandboxError::NotAvailable(
                "Mock provider not available".into(),
            ));
        }

        let is_destroyed = {
            let states = self.states.lock().unwrap();
            matches!(states.get(id), Some(SandboxState::Destroyed))
        };
        if is_destroyed {
            return Err(SandboxError::LifecycleViolation(format!(
                "Cannot run command in destroyed sandbox '{}'",
                id
            )));
        }

        let sandboxes = self.sandboxes.lock().unwrap();
        let matching = sandboxes.iter().find(|s| s.id == id || s.name == id);
        match matching {
            None => return Err(SandboxError::NotFound(id.to_string())),
            Some(s) => {
                self.validate_transition(&s.id, &SandboxState::Running)?;
                self.set_state(&s.id, SandboxState::Running);
                self.set_state(&s.name, SandboxState::Running);
            }
        }

        if *self.simulate_timeout.lock().unwrap() {
            return Ok(SandboxExecResult {
                exit_code: 124,
                stdout: String::new(),
                stderr: "Command timed out".to_string(),
            });
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
        if !self.available && *self.simulate_destroy_unavailable.lock().unwrap() {
            return Err(SandboxError::NotAvailable(
                "Mock provider not available".into(),
            ));
        }

        let mut sandboxes = self.sandboxes.lock().unwrap();
        let matching_idx = sandboxes.iter().position(|s| s.id == id || s.name == id);

        match matching_idx {
            None => {
                self.set_state(id, SandboxState::Destroyed);
                return Err(SandboxError::NotFound(id.to_string()));
            }
            Some(idx) => {
                let s = &sandboxes[idx];
                if !s.name.starts_with("root-sandbox-") {
                    return Err(SandboxError::NotRootOwned(id.to_string()));
                }
                self.validate_transition(&s.id, &SandboxState::Destroyed)?;
                self.set_state(&s.id, SandboxState::Destroyed);
                self.set_state(&s.name, SandboxState::Destroyed);
                sandboxes.remove(idx);
            }
        }

        {
            let mut attempts = self.cleanup_attempts.lock().unwrap();
            *attempts += 1;
        }

        Ok(())
    }

    fn check_exists(&self, id: &str) -> Result<bool, SandboxError> {
        let states = self.states.lock().unwrap();
        Ok(states.contains_key(id) && states.get(id) != Some(&SandboxState::Destroyed))
    }

    fn check_reachable(&self, id: &str) -> Result<bool, SandboxError> {
        let states = self.states.lock().unwrap();
        Ok(matches!(
            states.get(id),
            Some(&SandboxState::Running) | Some(&SandboxState::Created)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_state(mock: &MockSandboxProvider, id: &str, expected: &SandboxState) {
        let states = mock.states.lock().unwrap();
        assert_eq!(
            states.get(id),
            Some(expected),
            "Expected sandbox '{}' to be in state {:?}, but got {:?}",
            id,
            expected,
            states.get(id)
        );
    }

    mod phase2_lifecycle {
        use super::*;

        #[test]
        fn test_state_enum_serialization() {
            let state = SandboxState::Running;
            let json = serde_json::to_string(&state).unwrap();
            assert_eq!(json, "\"Running\"");
            let deserialized: SandboxState = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, SandboxState::Running);
        }

        #[test]
        fn test_sandbox_instance_uses_state() {
            let inst = SandboxInstance {
                id: "test-id".into(),
                name: "root-sandbox-test".into(),
                status: "created".into(),
                state: SandboxState::Created,
                created_at: "now".into(),
                image: "ubuntu:latest".into(),
                memory: None,
                cpus: None,
            };
            assert_eq!(inst.state, SandboxState::Created);
            let json = serde_json::to_string(&inst).unwrap();
            assert!(json.contains("\"Created\""));
        }

        #[test]
        fn test_valid_transitions() {
            let mock = MockSandboxProvider::new(true);
            let instance = mock.create("test-transitions", None, None, None).unwrap();
            let id = &instance.id;

            assert_state(&mock, id, &SandboxState::Created);

            mock.run_command(id, &["echo", "hi"], None).unwrap();
            assert_state(&mock, id, &SandboxState::Running);

            mock.destroy(id).unwrap();
            assert_state(&mock, id, &SandboxState::Destroyed);
        }

        #[test]
        fn test_invalid_transition_destroyed_to_anything() {
            let mock = MockSandboxProvider::new(true);
            let instance = mock.create("test-invalid", None, None, None).unwrap();
            let id = &instance.id;

            mock.destroy(id).unwrap();
            assert_state(&mock, id, &SandboxState::Destroyed);

            let err = mock.run_command(id, &["echo", "hi"], None).unwrap_err();
            assert!(matches!(err, SandboxError::LifecycleViolation(_)));
            assert!(err.to_string().contains("Invalid state transition"));
        }

        #[test]
        fn test_invalid_transition_completed_to_running() {
            let mock = MockSandboxProvider::new(true);
            let instance = mock.create("test-completed", None, None, None).unwrap();
            let id = &instance.id;

            mock.set_state(id, SandboxState::Completed);

            let err = mock.run_command(id, &["echo", "hi"], None).unwrap_err();
            assert!(matches!(err, SandboxError::LifecycleViolation(_)));
        }

        #[test]
        fn test_repeated_destroy() {
            let mock = MockSandboxProvider::new(true);
            let instance = mock.create("test-repeated", None, None, None).unwrap();
            let id = &instance.id;

            mock.destroy(id).unwrap();
            let err = mock.destroy(id).unwrap_err();
            assert!(matches!(err, SandboxError::NotFound(_)));
        }

        #[test]
        fn test_destroy_missing_sandbox() {
            let mock = MockSandboxProvider::new(true);
            let err = mock.destroy("nonexistent").unwrap_err();
            assert!(matches!(err, SandboxError::NotFound(_)));
        }

        #[test]
        fn test_run_destroyed_sandbox() {
            let mock = MockSandboxProvider::new(true);
            let instance = mock.create("test-run-destroyed", None, None, None).unwrap();
            let id = &instance.id;

            mock.destroy(id).unwrap();
            let err = mock.run_command(id, &["echo", "hi"], None).unwrap_err();
            assert!(matches!(err, SandboxError::LifecycleViolation(_)));
        }

        #[test]
        fn test_invalid_transition_failed_to_running() {
            let mock = MockSandboxProvider::new(true);
            let instance = mock.create("test-failed", None, None, None).unwrap();
            let id = &instance.id;

            mock.set_state(id, SandboxState::Failed);

            let err = mock.run_command(id, &["echo", "hi"], None).unwrap_err();
            assert!(matches!(err, SandboxError::LifecycleViolation(_)));
        }
    }

    mod phase3_cleanup {
        use super::*;

        #[test]
        fn test_cleanup_attempts_tracked() {
            let mock = MockSandboxProvider::new(true);
            let instance = mock.create("test-cleanup", None, None, None).unwrap();
            assert_eq!(*mock.cleanup_attempts.lock().unwrap(), 0);

            mock.destroy(&instance.id).unwrap();
            assert_eq!(*mock.cleanup_attempts.lock().unwrap(), 1);

            let instance2 = mock.create("test-cleanup-2", None, None, None).unwrap();
            mock.destroy(&instance2.id).unwrap();
            assert_eq!(*mock.cleanup_attempts.lock().unwrap(), 2);
        }

        #[test]
        fn test_double_cleanup() {
            let mock = MockSandboxProvider::new(true);
            let instance = mock.create("test-double", None, None, None).unwrap();
            mock.destroy(&instance.id).unwrap();
            assert_eq!(*mock.cleanup_attempts.lock().unwrap(), 1);
            let err = mock.destroy(&instance.id).unwrap_err();
            assert!(matches!(err, SandboxError::NotFound(_)));
            assert_eq!(*mock.cleanup_attempts.lock().unwrap(), 1);
        }

        #[test]
        fn test_failed_run_cleanup_tracking() {
            let mock = MockSandboxProvider::new(true);
            let instance = mock.create("test-fail-cleanup", None, None, None).unwrap();
            assert_eq!(*mock.cleanup_attempts.lock().unwrap(), 0);
            mock.destroy(&instance.id).unwrap();
            assert_eq!(*mock.cleanup_attempts.lock().unwrap(), 1);
        }
    }

    mod phase4_resource_limits {
        use super::*;

        #[test]
        fn test_create_with_resource_limits_mock() {
            let mock = MockSandboxProvider::new(true);
            let instance = mock
                .create("test-resources", None, Some("4g"), Some("4.0"))
                .unwrap();
            assert_eq!(instance.memory.as_deref(), Some("4g"));
            assert_eq!(instance.cpus.as_deref(), Some("4.0"));
        }

        #[test]
        fn test_create_with_default_resources_mock() {
            let mock = MockSandboxProvider::new(true);
            let instance = mock.create("test-defaults", None, None, None).unwrap();
            assert_eq!(instance.memory, None);
            assert_eq!(instance.cpus, None);
        }

        #[test]
        fn test_resource_limits_serialized() {
            let instance = SandboxInstance {
                id: "test-id".into(),
                name: "root-sandbox-test".into(),
                status: "created".into(),
                state: SandboxState::Created,
                created_at: "now".into(),
                image: "ubuntu:latest".into(),
                memory: Some("4g".into()),
                cpus: Some("4.0".into()),
            };
            let json = serde_json::to_string(&instance).unwrap();
            assert!(json.contains("\"4g\""));
            assert!(json.contains("\"4.0\""));
        }
    }

    mod phase5_timeout {
        use super::*;

        #[test]
        fn test_timeout_returns_exit_code_124() {
            let mock = MockSandboxProvider::new(true);
            *mock.simulate_timeout.lock().unwrap() = true;
            let instance = mock.create("test-timeout", None, None, None).unwrap();

            let result = mock
                .run_command(&instance.id, &["sleep", "10"], Some(1))
                .unwrap();
            assert_eq!(result.exit_code, 124);
            assert!(result.stderr.contains("timed out"));
        }

        #[test]
        fn test_timeout_not_triggered_with_sufficient_time() {
            let mock = MockSandboxProvider::new(true);
            let instance = mock.create("test-no-timeout", None, None, None).unwrap();

            let result = mock
                .run_command(&instance.id, &["echo", "hello"], Some(300))
                .unwrap();
            assert_eq!(result.exit_code, 0);
        }

        #[test]
        fn test_default_timeout_300() {
            let mock = MockSandboxProvider::new(true);
            let instance = mock
                .create("test-default-timeout", None, None, None)
                .unwrap();

            let result = mock
                .run_command(&instance.id, &["echo", "hello"], None)
                .unwrap();
            assert_eq!(result.exit_code, 0);
        }
    }

    mod phase6_validation {
        use super::*;

        #[test]
        fn test_check_exists_returns_true_for_existing() {
            let mock = MockSandboxProvider::new(true);
            let instance = mock.create("test-exists", None, None, None).unwrap();
            assert!(mock.check_exists(&instance.id).unwrap());
        }

        #[test]
        fn test_check_exists_returns_false_for_destroyed() {
            let mock = MockSandboxProvider::new(true);
            let instance = mock.create("test-gone", None, None, None).unwrap();
            mock.destroy(&instance.id).unwrap();
            assert!(!mock.check_exists(&instance.id).unwrap());
        }

        #[test]
        fn test_check_exists_returns_false_for_unknown() {
            let mock = MockSandboxProvider::new(true);
            assert!(!mock.check_exists("unknown").unwrap());
        }

        #[test]
        fn test_check_reachable_returns_true_for_running() {
            let mock = MockSandboxProvider::new(true);
            let instance = mock.create("test-reachable", None, None, None).unwrap();
            assert!(mock.check_reachable(&instance.id).unwrap());
        }

        #[test]
        fn test_check_reachable_returns_false_for_destroyed() {
            let mock = MockSandboxProvider::new(true);
            let instance = mock.create("test-unreachable", None, None, None).unwrap();
            mock.destroy(&instance.id).unwrap();
            assert!(!mock.check_reachable(&instance.id).unwrap());
        }
    }

    mod phase8_error_normalization {
        use super::*;

        #[test]
        fn test_error_display_not_available() {
            let err = SandboxError::NotAvailable("Docker not found".into());
            let msg = format!("{}", err);
            assert!(msg.contains("not available"));
            assert!(msg.contains("Docker not found"));
        }

        #[test]
        fn test_error_display_not_found() {
            let err = SandboxError::NotFound("sandbox-123".into());
            let msg = format!("{}", err);
            assert!(msg.contains("not found"));
            assert!(msg.contains("sandbox-123"));
        }

        #[test]
        fn test_error_display_lifecycle_violation() {
            let err = SandboxError::LifecycleViolation(
                "Invalid state transition: Destroyed -> Running".into(),
            );
            let msg = format!("{}", err);
            assert!(msg.contains("Invalid state transition"));
        }

        #[test]
        fn test_error_display_timeout() {
            let err = SandboxError::TimeoutExceeded("300".into());
            let msg = format!("{}", err);
            assert!(msg.contains("timed out"));
            assert!(msg.contains("300"));
        }

        #[test]
        fn test_error_display_cleanup_failed() {
            let err = SandboxError::CleanupFailed("rm failed".into());
            let msg = format!("{}", err);
            assert!(msg.contains("Cleanup failed"));
            assert!(msg.contains("rm failed"));
        }

        #[test]
        fn test_error_display_permission_denied() {
            let err = SandboxError::PermissionDenied("access denied".into());
            let msg = format!("{}", err);
            assert!(msg.contains("Permission denied"));
            assert!(msg.contains("access denied"));
        }

        #[test]
        fn test_error_display_resource_limit() {
            let err = SandboxError::ResourceLimitExceeded("OOM killed".into());
            let msg = format!("{}", err);
            assert!(msg.contains("Resource limit exceeded"));
            assert!(msg.contains("OOM killed"));
        }
    }

    mod existing_tests_migrated {
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

            let instance = mock.create("test-1", None, None, None).unwrap();
            assert_eq!(instance.name, "root-sandbox-test-1");
            assert_eq!(instance.state, SandboxState::Created);

            let instances = mock.list().unwrap();
            assert_eq!(instances.len(), 1);

            mock.destroy(&instance.id).unwrap();
            assert!(mock.list().unwrap().is_empty());
        }

        #[test]
        fn test_mock_run_command() {
            let mock = MockSandboxProvider::new(true);
            let instance = mock.create("test-run-migrated", None, None, None).unwrap();

            let result = mock
                .run_command(&instance.id, &["echo", "hello"], None)
                .unwrap();
            assert_eq!(result.exit_code, 0);
            assert!(result.stdout.contains("echo hello"));

            let result = mock
                .run_command(&instance.name, &["ls", "-la"], None)
                .unwrap();
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
            mock.create("my-sandbox", None, None, None).unwrap();

            let result = mock.destroy("root-sandbox-my-sandbox");
            assert!(result.is_ok(), "expected ok: {:?}", result);
            assert!(mock.list().unwrap().is_empty());
        }

        #[test]
        fn test_mock_destroy_rejects_non_root_container() {
            let mock = MockSandboxProvider::new(true);
            {
                let mut sandboxes = mock.sandboxes.lock().unwrap();
                sandboxes.push(SandboxInstance {
                    id: "ext-123".to_string(),
                    name: "external-container".to_string(),
                    status: "running".to_string(),
                    state: SandboxState::Running,
                    created_at: chrono::Utc::now().to_rfc3339(),
                    image: "ubuntu:latest".to_string(),
                    memory: None,
                    cpus: None,
                });
            }
            let err = mock.destroy("external-container").unwrap_err();
            assert!(matches!(err, SandboxError::NotRootOwned(_)));
        }

        #[test]
        fn test_mock_destroy_by_id_root_owned() {
            let mock = MockSandboxProvider::new(true);
            let instance = mock.create("test-1", None, None, None).unwrap();
            mock.destroy(&instance.id).unwrap();
            assert!(mock.list().unwrap().is_empty());
        }

        #[test]
        fn test_mock_unavailable_errors() {
            let mock = MockSandboxProvider::new(false);

            assert!(mock.create("x", None, None, None).is_err());
            assert!(mock.run_command("x", &["echo"], None).is_err());
            assert!(mock.list().is_err());
            assert!(mock.destroy("x").is_err());
        }
    }
}
