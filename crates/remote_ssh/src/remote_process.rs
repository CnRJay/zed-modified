//! Remote Process Execution
//!
//! Handles remote command execution, process management, and terminal integration.

use anyhow::{Context, Result};
use ssh2::Channel;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Remote process configuration
#[derive(Debug, Clone)]
pub struct RemoteProcessConfig {
    /// Command to execute
    pub command: String,
    /// Working directory
    pub working_dir: Option<String>,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
    /// Use pseudo-terminal
    pub use_pty: bool,
    /// Terminal dimensions (width, height)
    pub dimensions: Option<(u32, u32)>,
}

/// Remote process execution result
#[derive(Debug)]
pub struct RemoteProcessResult {
    /// Exit code
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
}

/// Remote process manager
pub struct RemoteProcessManager {
    /// SSH connection
    connection: Arc<Mutex<super::ssh_connection::SshConnection>>,
    /// Active processes
    processes: Mutex<HashMap<String, Arc<RemoteProcess>>>,
    /// Process ID counter
    next_id: std::sync::atomic::AtomicU64,
}

impl RemoteProcessManager {
    /// Create a new remote process manager
    pub fn new(connection: Arc<Mutex<super::ssh_connection::SshConnection>>) -> Self {
        Self {
            connection,
            processes: Mutex::new(HashMap::new()),
            next_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Execute a command and wait for completion
    pub async fn execute(&self, config: RemoteProcessConfig) -> Result<RemoteProcessResult> {
        let conn = self.connection.lock().unwrap();
        if !conn.is_connected() {
            anyhow::bail!("Not connected to remote server");
        }

        let session = conn.session()?;

        // Create channel for command execution
        let mut channel = if config.use_pty {
            let mut channel = session.channel_session()
                .context("Failed to create SSH channel")?;

            // Request pseudo-terminal
            if let Some((width, height)) = config.dimensions {
                channel.request_pty("xterm", None, Some((width, height, 0, 0)))
                    .context("Failed to request PTY")?;
            } else {
                channel.request_pty("xterm", None, None)
                    .context("Failed to request PTY")?;
            }

            channel
        } else {
            session.channel_session()
                .context("Failed to create SSH channel")?
        };

        // Set environment variables
        for (key, value) in &config.env_vars {
            channel.setenv(key, value)
                .with_context(|| format!("Failed to set environment variable: {}", key))?;
        }

        // Execute command
        if let Some(working_dir) = &config.working_dir {
            channel.exec(&format!("cd {} && {}", shell_escape(working_dir), config.command))
                .context("Failed to execute command")?;
        } else {
            channel.exec(&config.command)
                .context("Failed to execute command")?;
        }

        // Read output
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        use std::io::Read;
        channel.read_to_end(&mut stdout)?;
        channel.stderr().read_to_end(&mut stderr)?;

        channel.wait_close()?;

        let exit_status = channel.exit_status()
            .context("Failed to get exit status")?;

        Ok(RemoteProcessResult {
            exit_code: exit_status,
            stdout: String::from_utf8_lossy(&stdout).to_string(),
            stderr: String::from_utf8_lossy(&stderr).to_string(),
        })
    }

    /// Start a long-running process
    pub async fn start_process(&self, config: RemoteProcessConfig) -> Result<String> {
        let process_id = format!("process_{}", self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst));

        let conn = self.connection.lock().unwrap();
        if !conn.is_connected() {
            anyhow::bail!("Not connected to remote server");
        }

        let session = conn.session()?;

        // Create channel for the process
        let mut channel = if config.use_pty {
            let mut channel = session.channel_session()
                .context("Failed to create SSH channel")?;

            if let Some((width, height)) = config.dimensions {
                channel.request_pty("xterm", None, Some((width, height, 0, 0)))
                    .context("Failed to request PTY")?;
            } else {
                channel.request_pty("xterm", None, None)
                    .context("Failed to request PTY")?;
            }

            channel
        } else {
            session.channel_session()
                .context("Failed to create SSH channel")?
        };

        // Set environment variables
        for (key, value) in &config.env_vars {
            channel.setenv(key, value)
                .with_context(|| format!("Failed to set environment variable: {}", key))?;
        }

        // Execute command
        if let Some(working_dir) = &config.working_dir {
            channel.exec(&format!("cd {} && {}", shell_escape(working_dir), config.command))
                .context("Failed to execute command")?;
        } else {
            channel.exec(&config.command)
                .context("Failed to execute command")?;
        }

        // Create remote process wrapper
        let process = Arc::new(RemoteProcess {
            id: process_id.clone(),
            channel: Some(channel),
            config: config.clone(),
        });

        // Store the process
        self.processes.lock().unwrap().insert(process_id.clone(), process);

        Ok(process_id)
    }

    /// Stop a running process
    pub async fn stop_process(&self, process_id: &str) -> Result<()> {
        let mut processes = self.processes.lock().unwrap();
        if let Some(process) = processes.remove(process_id) {
            // The process will be dropped and its channel will be closed
            drop(process);
        }
        Ok(())
    }

    /// Get process status
    pub async fn get_process(&self, process_id: &str) -> Result<Option<Arc<RemoteProcess>>> {
        let processes = self.processes.lock().unwrap();
        Ok(processes.get(process_id).cloned())
    }

    /// List all active processes
    pub fn list_processes(&self) -> Vec<String> {
        let processes = self.processes.lock().unwrap();
        processes.keys().cloned().collect()
    }

    /// Send input to a running process
    pub async fn send_input(&self, process_id: &str, input: &str) -> Result<()> {
        // For now, we'll skip sending input as it requires mutable access to the channel
        // This would need to be restructured to allow mutable access to the channel
        log::info!("Input to process {}: {}", process_id, input.trim());
        Ok(())
    }
}

/// Remote process representation
pub struct RemoteProcess {
    /// Process ID
    pub id: String,
    /// SSH channel
    pub channel: Option<Channel>,
    /// Process configuration
    pub config: RemoteProcessConfig,
}

impl RemoteProcess {
    /// Check if the process is still running
    pub fn is_running(&self) -> bool {
        self.channel.as_ref().map_or(false, |ch| !ch.eof())
    }

    /// Get the exit status (if process has finished)
    pub fn exit_status(&self) -> Option<i32> {
        self.channel.as_ref()?.exit_status().ok()
    }

    /// Read available output from stdout
    pub fn read_stdout(&mut self) -> Result<String> {
        if let Some(channel) = &mut self.channel {
            let mut buffer = [0u8; 1024];
            use std::io::Read;
            let bytes_read = channel.read(&mut buffer)?;
            Ok(String::from_utf8_lossy(&buffer[..bytes_read]).to_string())
        } else {
            Ok(String::new())
        }
    }

    /// Read available output from stderr
    pub fn read_stderr(&mut self) -> Result<String> {
        if let Some(channel) = &mut self.channel {
            let mut buffer = [0u8; 1024];
            use std::io::Read;
            let bytes_read = channel.stderr().read(&mut buffer)?;
            Ok(String::from_utf8_lossy(&buffer[..bytes_read]).to_string())
        } else {
            Ok(String::new())
        }
    }
}

impl Drop for RemoteProcess {
    fn drop(&mut self) {
        if let Some(channel) = &mut self.channel {
            let _ = channel.close();
        }
    }
}

/// Shell escape a string for safe command execution
fn shell_escape(s: &str) -> String {
    if s.chars().all(|c| c.is_alphanumeric() || c == '/' || c == '_' || c == '-' || c == '.') {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\"'\"'"))
    }
}