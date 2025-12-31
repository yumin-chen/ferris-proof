use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct SandboxedExecutor {
    allowed_paths: Vec<PathBuf>,
    network_policy: NetworkPolicy,
    limits: ResourceLimits,
}

#[derive(Debug, Clone)]
pub enum NetworkPolicy {
    Denied,
    AllowList(Vec<String>),
    Unrestricted { user_consent: bool },
}

#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory: u64,
    pub max_cpu_time: u64,
    pub max_file_descriptors: u32,
    pub max_processes: u32,
}

impl SandboxedExecutor {
    pub fn new() -> Self {
        Self {
            allowed_paths: Vec::new(),
            network_policy: NetworkPolicy::Denied,
            limits: ResourceLimits::default(),
        }
    }

    pub fn with_allowed_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.allowed_paths = paths;
        self
    }

    pub fn with_network_policy(mut self, policy: NetworkPolicy) -> Self {
        self.network_policy = policy;
        self
    }

    pub fn with_limits(mut self, limits: ResourceLimits) -> Self {
        self.limits = limits;
        self
    }

    pub async fn execute(
        &self,
        command: &str,
        args: &[&str],
        env: HashMap<String, String>,
        working_dir: Option<&PathBuf>,
    ) -> Result<SandboxedOutput> {
        debug!("Executing sandboxed command: {} {:?}", command, args);

        let mut cmd = Command::new(command);
        cmd.args(args)
            .envs(&env)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        // Apply resource limits (platform-specific implementation would go here)
        self.apply_resource_limits(&mut cmd)?;

        let start_time = std::time::Instant::now();
        let output = cmd.output()?;
        let execution_time = start_time.elapsed();

        if execution_time > Duration::from_secs(self.limits.max_cpu_time) {
            warn!("Command exceeded time limit: {:?}", execution_time);
        }

        Ok(SandboxedOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            execution_time,
        })
    }

    fn apply_resource_limits(&self, _cmd: &mut Command) -> Result<()> {
        // TODO: Implement platform-specific resource limits
        // On Linux: use cgroups or ulimit
        // On macOS: use launchctl or ulimit
        // On Windows: use job objects
        debug!("Applying resource limits: {:?}", self.limits);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SandboxedOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub execution_time: Duration,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: 2 * 1024 * 1024 * 1024, // 2GB
            max_cpu_time: 300, // 5 minutes
            max_file_descriptors: 1024,
            max_processes: 10,
        }
    }
}

impl Default for SandboxedExecutor {
    fn default() -> Self {
        Self::new()
    }
}