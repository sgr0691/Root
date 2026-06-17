use crate::{enforce_policy, events, get_or_create_rootfile};
use anyhow::{Context, Result};
use root_lockfile::get_root_dir;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum RunRequest {
    Task(String),
    Workflow(PathBuf),
    Command(Vec<OsString>),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowFile {
    #[serde(default = "default_workflow_version")]
    version: u32,
    #[serde(default)]
    name: Option<String>,
    command: String,
}

fn default_workflow_version() -> u32 {
    1
}

#[derive(Debug, Serialize)]
pub struct RunReport {
    pub success: bool,
    pub source: String,
    pub task: Option<String>,
    pub command: String,
    pub exit_code: i32,
    pub started_at: String,
    pub finished_at: String,
    pub duration_ms: u64,
    pub stdout_policy: &'static str,
    pub stderr_policy: &'static str,
    pub stdout: String,
    pub stderr: String,
    pub warnings: Vec<String>,
}

fn profile_path() -> Result<PathBuf> {
    Ok(get_root_dir()?.join("profiles").join("default").join("bin"))
}

fn configure_path(command: &mut Command) -> Result<()> {
    let mut paths = vec![profile_path()?];
    if let Some(current) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&current));
    }
    command.env(
        "PATH",
        std::env::join_paths(paths).context("Failed to construct Root execution PATH")?,
    );
    Ok(())
}

fn shell_command(command: &str) -> Command {
    let mut process = Command::new("/bin/sh");
    process.args(["-c", command]);
    process
}

fn workflow_from_file(path: &Path) -> Result<WorkflowFile> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read workflow file at {}", path.display()))?;
    let workflow: WorkflowFile = toml::from_str(&content)
        .with_context(|| format!("Failed to parse workflow file at {}", path.display()))?;
    if workflow.version != 1 {
        return Err(anyhow::anyhow!(
            "Unsupported workflow version {}. Expected 1.",
            workflow.version
        ));
    }
    if workflow.command.trim().is_empty() {
        return Err(anyhow::anyhow!("Workflow command cannot be empty."));
    }
    Ok(workflow)
}

pub fn run(request: RunRequest) -> Result<RunReport> {
    root_lockfile::init_root_dir()?;
    let (source, task, command_display, mut process) = match request {
        RunRequest::Task(task) => {
            let rootfile = get_or_create_rootfile()?;
            let command = rootfile.tasks.get(&task).ok_or_else(|| {
                anyhow::anyhow!(
                    "Task '{}' is not defined in Rootfile. Add it under [tasks].",
                    task
                )
            })?;
            (
                "task".to_string(),
                Some(task),
                command.clone(),
                shell_command(command),
            )
        }
        RunRequest::Workflow(path) => {
            let workflow = workflow_from_file(&path)?;
            let task = workflow.name.or_else(|| {
                path.file_stem()
                    .and_then(|name| name.to_str())
                    .map(ToString::to_string)
            });
            (
                "workflow".to_string(),
                task,
                workflow.command.clone(),
                shell_command(&workflow.command),
            )
        }
        RunRequest::Command(args) => {
            let (program, rest) = args
                .split_first()
                .ok_or_else(|| anyhow::anyhow!("No command provided after `root run --`."))?;
            let mut process = Command::new(program);
            process.args(rest);
            let display = args
                .iter()
                .map(|arg| arg.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");
            ("command".to_string(), None, display, process)
        }
    };

    enforce_policy(crate::policy::PolicyAction::Run, Some(&command_display))?;
    configure_path(&mut process)?;

    let started_at = chrono::Utc::now().to_rfc3339();
    let timer = Instant::now();
    let output = match process.output() {
        Ok(output) => output,
        Err(error) => {
            let duration_ms = timer.elapsed().as_millis().try_into().unwrap_or(u64::MAX);
            let finished_at = chrono::Utc::now().to_rfc3339();
            let _ = events::record_execution_event(
                &format!("root run {}", command_display),
                events::RootEventStatus::Failed,
                events::ExecutionEventDetails {
                    task_name: task,
                    exit_code: None,
                    started_at,
                    finished_at,
                    duration_ms,
                    message: Some(format!("Failed to start execution: {}", error)),
                },
            );
            return Err(error).with_context(|| format!("Failed to execute '{}'", command_display));
        }
    };
    let duration_ms = timer.elapsed().as_millis().try_into().unwrap_or(u64::MAX);
    let finished_at = chrono::Utc::now().to_rfc3339();
    let exit_code = output.status.code().unwrap_or(128);
    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let status = if success {
        events::RootEventStatus::Completed
    } else {
        events::RootEventStatus::Failed
    };
    let mut warnings = Vec::new();
    if let Err(error) = events::record_execution_event(
        &format!("root run {}", command_display),
        status,
        events::ExecutionEventDetails {
            task_name: task.clone(),
            exit_code: Some(exit_code),
            started_at: started_at.clone(),
            finished_at: finished_at.clone(),
            duration_ms,
            message: Some(format!("Execution source: {}", source)),
        },
    ) {
        warnings.push(format!("Failed to record execution history: {}", error));
    }

    Ok(RunReport {
        success,
        source,
        task,
        command: command_display,
        exit_code,
        started_at,
        finished_at,
        duration_ms,
        stdout_policy: "captured",
        stderr_policy: "captured",
        stdout,
        stderr,
        warnings,
    })
}
