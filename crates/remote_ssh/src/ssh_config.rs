//! SSH Configuration Management
//!
//! Handles SSH configuration files, connection settings, and VS Code compatibility.

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// SSH configuration entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshHostConfig {
    /// Host pattern (can include wildcards)
    pub host: String,
    /// Actual hostname or IP
    pub hostname: Option<String>,
    /// SSH port
    pub port: Option<u16>,
    /// Username
    pub user: Option<String>,
    /// SSH key file
    pub identity_file: Option<PathBuf>,
    /// Additional options
    pub options: HashMap<String, String>,
}

impl Default for SshHostConfig {
    fn default() -> Self {
        Self {
            host: "*".to_string(),
            hostname: None,
            port: Some(22),
            user: None,
            identity_file: None,
            options: HashMap::new(),
        }
    }
}

/// SSH configuration file parser
pub struct SshConfigParser;

impl SshConfigParser {
    /// Parse SSH config file
    pub fn parse_file(path: &Path) -> Result<Vec<SshHostConfig>> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read SSH config file: {}", path.display()))?;

        Self::parse_content(&content)
    }

    /// Parse SSH config content
    pub fn parse_content(content: &str) -> Result<Vec<SshHostConfig>> {
        let mut configs = Vec::new();
        let mut current_config: Option<SshHostConfig> = None;

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse key-value pairs
            if let Some((key, value)) = Self::parse_line(line) {
                match key.to_lowercase().as_str() {
                    "host" => {
                        // Save previous config if exists
                        if let Some(config) = current_config.take() {
                            configs.push(config);
                        }

                        // Start new config
                        current_config = Some(SshHostConfig {
                            host: value,
                            hostname: None,
                            port: None,
                            user: None,
                            identity_file: None,
                            options: HashMap::new(),
                        });
                    }
                    "hostname" => {
                        if let Some(ref mut config) = current_config {
                            config.hostname = Some(value);
                        }
                    }
                    "port" => {
                        if let Some(ref mut config) = current_config {
                            if let Ok(port) = value.parse::<u16>() {
                                config.port = Some(port);
                            }
                        }
                    }
                    "user" => {
                        if let Some(ref mut config) = current_config {
                            config.user = Some(value);
                        }
                    }
                    "identityfile" => {
                        if let Some(ref mut config) = current_config {
                            config.identity_file = Some(PathBuf::from(value));
                        }
                    }
                    _ => {
                        // Store other options
                        if let Some(ref mut config) = current_config {
                            config.options.insert(key.to_string(), value);
                        }
                    }
                }
            }
        }

        // Save the last config
        if let Some(config) = current_config {
            configs.push(config);
        }

        Ok(configs)
    }

    /// Parse a single line into key-value pair
    fn parse_line(line: &str) -> Option<(String, String)> {
        let mut parts = line.splitn(2, char::is_whitespace);
        let key = parts.next()?.trim();
        let value = parts.next()?.trim().trim_matches('"').trim_matches('\'');
        Some((key.to_string(), value.to_string()))
    }
}

/// VS Code remote SSH configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsCodeRemoteConfig {
    /// Remote host configuration
    pub host: String,
    /// Platform type
    pub platform: String,
    /// Remote architecture
    pub arch: Option<String>,
    /// Remote OS details
    pub os: Option<String>,
    /// Remote workspace path
    pub workspace_path: Option<PathBuf>,
}

/// VS Code remote SSH configuration manager
pub struct VsCodeRemoteManager {
    /// SSH configurations
    ssh_configs: HashMap<String, SshHostConfig>,
    /// VS Code remote configurations
    remote_configs: HashMap<String, VsCodeRemoteConfig>,
}

impl VsCodeRemoteManager {
    /// Create a new VS Code remote manager
    pub fn new() -> Self {
        Self {
            ssh_configs: HashMap::new(),
            remote_configs: HashMap::new(),
        }
    }

    /// Load SSH configurations from standard locations
    pub fn load_ssh_configs(&mut self) -> Result<()> {
        let config_paths = Self::get_ssh_config_paths();

        for path in config_paths {
            if path.exists() {
                let configs = SshConfigParser::parse_file(&path)
                    .with_context(|| format!("Failed to parse SSH config: {}", path.display()))?;

                for config in configs {
                    self.ssh_configs.insert(config.host.clone(), config);
                }
            }
        }

        Ok(())
    }

    /// Get SSH configuration for a host
    pub fn get_ssh_config(&self, host: &str) -> Option<&SshHostConfig> {
        // Try exact match first
        if let Some(config) = self.ssh_configs.get(host) {
            return Some(config);
        }

        // Try pattern matching (basic implementation)
        for (pattern, config) in &self.ssh_configs {
            if Self::matches_pattern(host, pattern) {
                return Some(config);
            }
        }

        None
    }

    /// Create SSH connection config from host name
    pub fn create_connection_config(&self, host: &str) -> Result<super::ssh_connection::SshConfig> {
        let ssh_config = self.get_ssh_config(host)
            .with_context(|| format!("No SSH configuration found for host: {}", host))?;

        let hostname = ssh_config.hostname.as_deref().unwrap_or(host);
        let port = ssh_config.port.unwrap_or(22);
        let user = ssh_config.user.as_deref()
            .context("SSH user not specified")?;

        // Determine authentication method
        let auth = if let Some(identity_file) = &ssh_config.identity_file {
            super::ssh_connection::SshAuth::PublicKey {
                public_key: identity_file.with_extension("pub"),
                private_key: identity_file.clone(),
                passphrase: ssh_config.options.get("passphrase").cloned(),
            }
        } else {
            // Default to agent authentication
            super::ssh_connection::SshAuth::Agent
        };

        Ok(super::ssh_connection::SshConfig {
            host: hostname.to_string(),
            port,
            user: user.to_string(),
            auth,
            timeout: 30,
            key_file: ssh_config.identity_file.clone(),
            known_hosts_file: Self::get_known_hosts_path(),
            strict_host_key_checking: true,
        })
    }

    /// Get standard SSH config file paths
    fn get_ssh_config_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Some(home) = dirs::home_dir() {
            paths.push(home.join(".ssh").join("config"));
        }

        // System-wide SSH config
        paths.push(PathBuf::from("/etc/ssh/ssh_config"));

        paths
    }

    /// Get SSH known hosts file path
    fn get_known_hosts_path() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".ssh").join("known_hosts"))
    }

    /// Check if a host matches a pattern (basic implementation)
    fn matches_pattern(host: &str, pattern: &str) -> bool {
        // Simple wildcard matching
        if pattern.contains('*') {
            let regex_pattern = pattern.replace('*', ".*");
            if let Ok(regex) = Regex::new(&format!("^{}$", regex_pattern)) {
                return regex.is_match(host);
            }
        }

        false
    }

    /// Save VS Code remote configuration
    pub fn save_remote_config(&mut self, host: String, config: VsCodeRemoteConfig) -> Result<()> {
        self.remote_configs.insert(host, config);
        Ok(())
    }

    /// Get VS Code remote configuration
    pub fn get_remote_config(&self, host: &str) -> Option<&VsCodeRemoteConfig> {
        self.remote_configs.get(host)
    }
}

impl Default for VsCodeRemoteManager {
    fn default() -> Self {
        Self::new()
    }
}