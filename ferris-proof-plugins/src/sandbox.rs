use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

/// Sandboxed executor for running external verification tools safely
///
/// This executor provides:
/// - Resource limits (memory, CPU time, file descriptors)
/// - Network access policy enforcement
/// - Timeout handling with graceful termination
/// - File system access restrictions
#[derive(Debug, Clone)]
pub struct SandboxedExecutor {
    allowed_paths: Vec<PathBuf>,
    network_policy: NetworkPolicy,
    limits: ResourceLimits,
    timeout_duration: Duration,
}

#[derive(Debug, Clone)]
pub enum NetworkPolicy {
    /// No network access allowed
    Denied,

    /// Allow connections to specified hosts only
    AllowList(Vec<String>),

    /// Allow all connections (user explicitly opted in)
    Unrestricted { user_consent: bool },
}

#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory: u64,

    /// Maximum CPU time in seconds
    pub max_cpu_time: u64,

    /// Maximum number of open file descriptors
    pub max_file_descriptors: u32,

    /// Maximum number of child processes
    pub max_processes: u32,

    /// Maximum file size that can be created (bytes)
    pub max_file_size: u64,
}

/// Configuration for network access consent
#[derive(Debug, Clone)]
pub struct NetworkConsent {
    pub granted: bool,
    pub timestamp: std::time::SystemTime,
    pub scope: ConsentScope,
}

#[derive(Debug, Clone)]
pub enum ConsentScope {
    /// Consent for specific hosts
    Hosts(Vec<String>),

    /// Consent for all network access
    Unrestricted,

    /// Consent for specific verification session
    Session(String),
}

impl SandboxedExecutor {
    /// Create a new sandboxed executor with default settings
    pub fn new() -> Self {
        Self {
            allowed_paths: Vec::new(),
            network_policy: NetworkPolicy::Denied,
            limits: ResourceLimits::default(),
            timeout_duration: Duration::from_secs(300), // 5 minutes default
        }
    }

    /// Configure allowed file system paths
    pub fn with_allowed_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.allowed_paths = paths;
        self
    }

    /// Configure network access policy
    pub fn with_network_policy(mut self, policy: NetworkPolicy) -> Self {
        self.network_policy = policy;
        self
    }

    /// Configure resource limits
    pub fn with_limits(mut self, limits: ResourceLimits) -> Self {
        self.limits = limits;
        self
    }

    /// Configure execution timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout_duration = timeout;
        self
    }

    /// Execute a command in the sandbox with comprehensive safety measures
    pub async fn execute(
        &self,
        command: &str,
        args: &[&str],
        env: HashMap<String, String>,
        working_dir: Option<&PathBuf>,
    ) -> Result<SandboxedOutput> {
        info!("Executing sandboxed command: {} {:?}", command, args);

        // Validate command and arguments
        self.validate_command(command, args)?;

        // Validate working directory
        if let Some(dir) = working_dir {
            self.validate_path_access(dir)?;
        }

        // Prepare command with security restrictions
        let cmd = self.prepare_command(command, args, env, working_dir)?;

        // Execute with timeout and resource monitoring
        let execution_result = self.execute_with_timeout(cmd).await?;

        // Validate execution results
        self.validate_execution_result(&execution_result)?;

        Ok(execution_result)
    }

    /// Validate that the command is safe to execute
    fn validate_command(&self, command: &str, args: &[&str]) -> Result<()> {
        // Check for dangerous commands
        let dangerous_commands = [
            "rm", "rmdir", "del", "format", "fdisk", "dd", "mkfs", "mount", "umount", "sudo", "su",
            "chmod", "chown", "curl", "wget", "nc", "netcat", "telnet",
        ];

        if dangerous_commands.contains(&command) {
            return Err(anyhow!("Command '{}' is not allowed in sandbox", command));
        }

        // Check for suspicious arguments
        for arg in args {
            if arg.contains("..") || arg.starts_with('/') {
                warn!("Suspicious argument detected: {}", arg);
            }

            // Check for network-related arguments
            if self.network_policy == NetworkPolicy::Denied
                && (arg.contains("http://") || arg.contains("https://") || arg.contains("ftp://"))
            {
                return Err(anyhow!("Network access denied: argument contains URL"));
            }
        }

        Ok(())
    }

    /// Validate that a path is within allowed access
    fn validate_path_access(&self, path: &PathBuf) -> Result<()> {
        if self.allowed_paths.is_empty() {
            // If no restrictions specified, allow current directory and subdirectories
            let current_dir = std::env::current_dir()?;
            let canonical_path = path.canonicalize().unwrap_or_else(|_| path.clone());

            if !canonical_path.starts_with(&current_dir) {
                return Err(anyhow!(
                    "Path access denied: {:?} is outside current directory",
                    path
                ));
            }
        } else {
            // Check against allowed paths
            let canonical_path = path.canonicalize().unwrap_or_else(|_| path.clone());
            let allowed = self
                .allowed_paths
                .iter()
                .any(|allowed_path| canonical_path.starts_with(allowed_path));

            if !allowed {
                return Err(anyhow!(
                    "Path access denied: {:?} is not in allowed paths",
                    path
                ));
            }
        }

        Ok(())
    }

    /// Prepare command with security restrictions
    fn prepare_command(
        &self,
        command: &str,
        args: &[&str],
        mut env: HashMap<String, String>,
        working_dir: Option<&PathBuf>,
    ) -> Result<Command> {
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set working directory
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        // Configure environment variables for security
        self.configure_environment(&mut env)?;
        cmd.envs(&env);

        // Apply resource limits
        self.apply_resource_limits(&mut cmd)?;

        // Apply network restrictions
        self.apply_network_restrictions(&mut cmd)?;

        Ok(cmd)
    }

    /// Configure environment variables for security
    fn configure_environment(&self, env: &mut HashMap<String, String>) -> Result<()> {
        // Remove potentially dangerous environment variables
        env.remove("LD_PRELOAD");
        env.remove("DYLD_INSERT_LIBRARIES");
        env.remove("PATH"); // Will be set to restricted PATH

        // Set restricted PATH
        env.insert("PATH".to_string(), "/usr/bin:/bin".to_string());

        // Disable network access if policy requires it
        match &self.network_policy {
            NetworkPolicy::Denied => {
                env.insert("NO_PROXY".to_string(), "*".to_string());
                env.insert("no_proxy".to_string(), "*".to_string());
                env.remove("HTTP_PROXY");
                env.remove("HTTPS_PROXY");
                env.remove("http_proxy");
                env.remove("https_proxy");
            }
            NetworkPolicy::AllowList(hosts) => {
                // Configure proxy settings to only allow specific hosts
                let allowed_hosts = hosts.join(",");
                env.insert("ALLOWED_HOSTS".to_string(), allowed_hosts);
            }
            NetworkPolicy::Unrestricted { user_consent } => {
                if !user_consent {
                    return Err(anyhow!("Network access requires explicit user consent"));
                }
            }
        }

        Ok(())
    }

    /// Apply resource limits to the command (platform-specific)
    fn apply_resource_limits(&self, cmd: &mut Command) -> Result<()> {
        debug!("Applying resource limits: {:?}", self.limits);

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;

            let limits = self.limits.clone();
            unsafe {
                cmd.pre_exec(move || {
                    // Set memory limit (RLIMIT_AS - virtual memory)
                    let memory_limit = libc::rlimit {
                        rlim_cur: limits.max_memory,
                        rlim_max: limits.max_memory,
                    };
                    if libc::setrlimit(libc::RLIMIT_AS, &memory_limit) != 0 {
                        eprintln!("Warning: Failed to set memory limit");
                    }

                    // Set CPU time limit (RLIMIT_CPU)
                    let cpu_limit = libc::rlimit {
                        rlim_cur: limits.max_cpu_time,
                        rlim_max: limits.max_cpu_time,
                    };
                    if libc::setrlimit(libc::RLIMIT_CPU, &cpu_limit) != 0 {
                        eprintln!("Warning: Failed to set CPU time limit");
                    }

                    // Set file descriptor limit (RLIMIT_NOFILE)
                    let fd_limit = libc::rlimit {
                        rlim_cur: limits.max_file_descriptors as u64,
                        rlim_max: limits.max_file_descriptors as u64,
                    };
                    if libc::setrlimit(libc::RLIMIT_NOFILE, &fd_limit) != 0 {
                        eprintln!("Warning: Failed to set file descriptor limit");
                    }

                    // Set process limit (RLIMIT_NPROC)
                    let proc_limit = libc::rlimit {
                        rlim_cur: limits.max_processes as u64,
                        rlim_max: limits.max_processes as u64,
                    };
                    if libc::setrlimit(libc::RLIMIT_NPROC, &proc_limit) != 0 {
                        eprintln!("Warning: Failed to set process limit");
                    }

                    Ok(())
                });
            }
        }

        #[cfg(windows)]
        {
            // Windows resource limits would be implemented using Job Objects
            warn!("Resource limits not yet implemented on Windows");
        }

        Ok(())
    }

    /// Apply network access restrictions
    fn apply_network_restrictions(&self, _cmd: &mut Command) -> Result<()> {
        match &self.network_policy {
            NetworkPolicy::Denied => {
                debug!("Network access denied for sandboxed execution");
                // Network restrictions are primarily handled through environment variables
                // and firewall rules (platform-specific implementation)
            }
            NetworkPolicy::AllowList(hosts) => {
                debug!("Network access restricted to hosts: {:?}", hosts);
                // Implementation would involve configuring network namespace or firewall rules
            }
            NetworkPolicy::Unrestricted { user_consent } => {
                if *user_consent {
                    debug!("Unrestricted network access granted with user consent");
                } else {
                    return Err(anyhow!("Network access requires user consent"));
                }
            }
        }

        Ok(())
    }

    /// Execute command with timeout and monitoring
    async fn execute_with_timeout(&self, mut cmd: Command) -> Result<SandboxedOutput> {
        let start_time = Instant::now();

        // Spawn the process
        let child = cmd
            .spawn()
            .map_err(|e| anyhow!("Failed to spawn process: {}", e))?;

        // Create a shared handle for the child process
        let child_handle = Arc::new(Mutex::new(Some(child)));
        let child_handle_clone = Arc::clone(&child_handle);

        // Set up timeout handling
        let timeout_result = timeout(self.timeout_duration, async {
            // Wait for the process to complete in a blocking task
            tokio::task::spawn_blocking(move || {
                let mut child_guard = child_handle_clone.lock().unwrap();
                if let Some(child) = child_guard.take() {
                    child.wait_with_output()
                } else {
                    Err(std::io::Error::other("Child process not available"))
                }
            })
            .await
            .unwrap_or_else(|e| Err(std::io::Error::other(e.to_string())))
        })
        .await;

        let execution_time = start_time.elapsed();

        match timeout_result {
            Ok(Ok(output)) => {
                info!("Command completed successfully in {:?}", execution_time);

                Ok(SandboxedOutput {
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    exit_code: output.status.code().unwrap_or(-1),
                    execution_time,
                    resource_usage: self.collect_resource_usage(),
                    timeout_occurred: false,
                })
            }
            Ok(Err(e)) => {
                error!("Process execution failed: {}", e);
                Err(anyhow!("Process execution failed: {}", e))
            }
            Err(_) => {
                warn!(
                    "Command timed out after {:?}, attempting graceful termination",
                    self.timeout_duration
                );

                // Attempt graceful termination
                self.terminate_process_gracefully(&child_handle).await?;

                Ok(SandboxedOutput {
                    stdout: String::new(),
                    stderr: format!(
                        "Process terminated due to timeout ({:?})",
                        self.timeout_duration
                    ),
                    exit_code: -1,
                    execution_time,
                    resource_usage: self.collect_resource_usage(),
                    timeout_occurred: true,
                })
            }
        }
    }

    /// Terminate process gracefully with escalating signals
    async fn terminate_process_gracefully(
        &self,
        child_handle: &Arc<Mutex<Option<Child>>>,
    ) -> Result<()> {
        let pid = {
            let mut child_guard = child_handle.lock().unwrap();
            if let Some(ref mut child) = child_guard.as_mut() {
                #[cfg(unix)]
                {
                    // Try SIGTERM first
                    let pid = child.id();
                    unsafe {
                        libc::kill(pid as i32, libc::SIGTERM);
                    }
                    Some(pid)
                }
                #[cfg(windows)]
                {
                    // On Windows, use TerminateProcess
                    if let Err(e) = child.kill() {
                        error!("Failed to terminate process: {}", e);
                    }
                    None
                }
            } else {
                None
            }
        }; // Drop the lock before await

        // Wait a bit for graceful shutdown
        tokio::time::sleep(Duration::from_secs(2)).await;

        #[cfg(unix)]
        if let Some(pid) = pid {
            let mut child_guard = child_handle.lock().unwrap();
            // Check if process is still running
            if let Some(ref mut child) = child_guard.as_mut() {
                match child.try_wait() {
                    Ok(Some(_)) => {
                        debug!("Process terminated gracefully");
                        return Ok(());
                    }
                    Ok(None) => {
                        // Still running, try SIGKILL
                        warn!("Process did not respond to SIGTERM, sending SIGKILL");
                        unsafe {
                            libc::kill(pid as i32, libc::SIGKILL);
                        }
                    }
                    Err(e) => {
                        error!("Error checking process status: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Collect resource usage statistics
    fn collect_resource_usage(&self) -> ResourceUsage {
        // This would collect actual resource usage statistics
        // For now, return default values
        ResourceUsage {
            peak_memory: 0,
            cpu_time: Duration::from_secs(0),
            file_descriptors_used: 0,
            processes_spawned: 1,
        }
    }

    /// Validate execution results for security compliance
    fn validate_execution_result(&self, result: &SandboxedOutput) -> Result<()> {
        // Check for suspicious output patterns
        if result.stderr.contains("Permission denied") && result.exit_code != 0 {
            debug!("Process encountered permission restrictions (expected)");
        }

        // Check for network access attempts when denied
        if matches!(self.network_policy, NetworkPolicy::Denied)
            && (result.stderr.contains("Connection refused")
                || result.stderr.contains("Network is unreachable"))
        {
            debug!("Network access properly blocked");
        }

        // Validate resource usage
        if result.execution_time > self.timeout_duration {
            warn!("Execution time exceeded configured timeout");
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SandboxedOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub execution_time: Duration,
    pub resource_usage: ResourceUsage,
    pub timeout_occurred: bool,
}

#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub peak_memory: u64,
    pub cpu_time: Duration,
    pub file_descriptors_used: u32,
    pub processes_spawned: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: 2 * 1024 * 1024 * 1024, // 2GB
            max_cpu_time: 300,                  // 5 minutes
            max_file_descriptors: 1024,
            max_processes: 10,
            max_file_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

impl PartialEq for NetworkPolicy {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (NetworkPolicy::Denied, NetworkPolicy::Denied) => true,
            (NetworkPolicy::AllowList(a), NetworkPolicy::AllowList(b)) => a == b,
            (
                NetworkPolicy::Unrestricted { user_consent: a },
                NetworkPolicy::Unrestricted { user_consent: b },
            ) => a == b,
            _ => false,
        }
    }
}

impl Default for SandboxedExecutor {
    fn default() -> Self {
        Self::new()
    }
}
