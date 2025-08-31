//! Remote Workspace Integration
//!
//! Handles remote workspace configurations and VS Code compatibility.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Remote workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteWorkspaceConfig {
    /// Remote folders in the workspace
    pub folders: Vec<RemoteWorkspaceFolder>,
    /// Workspace settings
    pub settings: HashMap<String, serde_json::Value>,
    /// Workspace-specific extensions
    pub extensions: RemoteExtensionsConfig,
    /// Launch configurations for debugging
    pub launch: Option<serde_json::Value>,
    /// Task configurations
    pub tasks: Option<serde_json::Value>,
}

/// Remote workspace folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteWorkspaceFolder {
    /// Folder name (display name)
    pub name: Option<String>,
    /// Remote path
    pub path: String,
    /// URI scheme (e.g., "vscode-remote", "ssh-remote")
    pub uri: Option<String>,
}

/// Remote extensions configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteExtensionsConfig {
    /// Extensions to install on remote
    pub install: Vec<String>,
    /// Extensions to uninstall from remote
    pub uninstall: Vec<String>,
    /// Extensions to enable on remote
    pub enable: Vec<String>,
    /// Extensions to disable on remote
    pub disable: Vec<String>,
}

/// Remote workspace manager
pub struct RemoteWorkspaceManager {
    /// SSH connection
    connection: std::sync::Arc<std::sync::Mutex<super::ssh_connection::SshConnection>>,
    /// Remote workspace configuration
    config: Option<RemoteWorkspaceConfig>,
    /// Local workspace path (for VS Code compatibility)
    local_workspace_path: Option<PathBuf>,
}

impl RemoteWorkspaceManager {
    /// Create a new remote workspace manager
    pub fn new(connection: std::sync::Arc<std::sync::Mutex<super::ssh_connection::SshConnection>>) -> Self {
        Self {
            connection,
            config: None,
            local_workspace_path: None,
        }
    }

    /// Load workspace configuration from remote path
    pub async fn load_workspace(&mut self, workspace_path: &Path) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        if !conn.is_connected() {
            anyhow::bail!("Not connected to remote server");
        }

        // Try to load .zed/workspace.json first
        let workspace_file = workspace_path.join(".zed").join("workspace.json");

        let config_content = if self.remote_file_exists(&workspace_file).await? {
            match conn.execute(&format!("cat {}", shell_escape(workspace_file.to_string_lossy().as_ref()))).await {
                Ok(content) => content,
                Err(e) => return Err(anyhow::anyhow!("Failed to read workspace file {}: {}", workspace_file.display(), e)),
            }
        } else {
            // Fall back to .vscode/settings.json for VS Code compatibility
            let vscode_settings = workspace_path.join(".vscode").join("settings.json");
            if self.remote_file_exists(&vscode_settings).await? {
                let settings_content = match conn.execute(&format!("cat {}", shell_escape(vscode_settings.to_string_lossy().as_ref()))).await {
                    Ok(content) => content,
                    Err(e) => return Err(anyhow::anyhow!("Failed to read VS Code settings {}: {}", vscode_settings.display(), e)),
                };

                // Convert VS Code settings to Zed workspace format
                self.convert_vscode_settings(&settings_content)?
            } else {
                // Create default workspace configuration
                self.create_default_config(workspace_path)
            }
        };

        let config: RemoteWorkspaceConfig = serde_json::from_str(&config_content)
            .with_context(|| "Failed to parse workspace configuration")?;

        self.config = Some(config);
        self.local_workspace_path = Some(workspace_path.to_path_buf());

        Ok(())
    }

    /// Save workspace configuration to remote
    pub async fn save_workspace(&self) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        if !conn.is_connected() {
            anyhow::bail!("Not connected to remote server");
        }

        let config = self.config.as_ref()
            .context("No workspace configuration loaded")?;

        let workspace_path = self.local_workspace_path.as_ref()
            .context("No workspace path set")?;

        let workspace_dir = workspace_path.join(".zed");
        let workspace_file = workspace_dir.join("workspace.json");

        // Create .zed directory if it doesn't exist
        let mkdir_cmd = format!("mkdir -p {}", shell_escape(workspace_dir.to_string_lossy().as_ref()));
        conn.execute(&mkdir_cmd).await
            .map_err(|e| anyhow::anyhow!("Failed to create workspace directory {}: {}", workspace_dir.display(), e))?;

        // Write workspace configuration
        let config_json = serde_json::to_string_pretty(config)
            .context("Failed to serialize workspace configuration")?;

        let temp_file = format!("/tmp/zed_workspace_{}.json", std::process::id());
        let write_cmd = format!("cat > {} << 'EOF'\n{}\nEOF", shell_escape(&temp_file), config_json);
        conn.execute(&write_cmd).await
            .map_err(|e| anyhow::anyhow!("Failed to write temporary workspace file: {}", e))?;

        let move_cmd = format!("mv {} {}", shell_escape(&temp_file), shell_escape(workspace_file.to_string_lossy().as_ref()));
        conn.execute(&move_cmd).await
            .map_err(|e| anyhow::anyhow!("Failed to move workspace file to final location: {}", e))?;

        Ok(())
    }

    /// Get workspace configuration
    pub fn get_config(&self) -> Option<&RemoteWorkspaceConfig> {
        self.config.as_ref()
    }

    /// Update workspace configuration
    pub fn update_config(&mut self, config: RemoteWorkspaceConfig) {
        self.config = Some(config);
    }

    /// Get remote workspace folders
    pub fn get_folders(&self) -> Vec<&RemoteWorkspaceFolder> {
        self.config.as_ref()
            .map(|c| c.folders.iter().collect())
            .unwrap_or_default()
    }

    /// Add a folder to the workspace
    pub fn add_folder(&mut self, folder: RemoteWorkspaceFolder) -> Result<()> {
        if let Some(config) = &mut self.config {
            config.folders.push(folder);
            Ok(())
        } else {
            anyhow::bail!("No workspace configuration loaded");
        }
    }

    /// Remove a folder from the workspace
    pub fn remove_folder(&mut self, path: &str) -> Result<()> {
        if let Some(config) = &mut self.config {
            config.folders.retain(|f| f.path != path);
            Ok(())
        } else {
            anyhow::bail!("No workspace configuration loaded");
        }
    }

    /// Get workspace settings
    pub fn get_setting(&self, key: &str) -> Option<&serde_json::Value> {
        self.config.as_ref()?
            .settings.get(key)
    }

    /// Set workspace setting
    pub fn set_setting(&mut self, key: String, value: serde_json::Value) -> Result<()> {
        if let Some(config) = &mut self.config {
            config.settings.insert(key, value);
            Ok(())
        } else {
            anyhow::bail!("No workspace configuration loaded");
        }
    }

    /// Convert VS Code settings to Zed workspace format
    fn convert_vscode_settings(&self, vscode_settings: &str) -> Result<String> {
        let vscode_config: serde_json::Value = serde_json::from_str(vscode_settings)
            .context("Failed to parse VS Code settings")?;

        // Extract remote-specific settings
        let mut zed_config = RemoteWorkspaceConfig {
            folders: Vec::new(), // Will be populated by caller
            settings: HashMap::new(),
            extensions: RemoteExtensionsConfig {
                install: Vec::new(),
                uninstall: Vec::new(),
                enable: Vec::new(),
                disable: Vec::new(),
            },
            launch: None,
            tasks: None,
        };

        // Copy relevant settings
        if let Some(settings) = vscode_config.as_object() {
            for (key, value) in settings {
                // Convert VS Code specific settings to Zed equivalents
                match key.as_str() {
                    "remote.SSH.configFile" => {
                        // Handle SSH config file setting
                        zed_config.settings.insert("remote.ssh.configFile".to_string(), value.clone());
                    }
                    "remote.SSH.connectTimeout" => {
                        zed_config.settings.insert("remote.ssh.connectTimeout".to_string(), value.clone());
                    }
                    _ => {
                        // Keep other settings as-is
                        zed_config.settings.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        serde_json::to_string_pretty(&zed_config)
            .context("Failed to serialize Zed workspace configuration")
    }

    /// Create default workspace configuration
    fn create_default_config(&self, workspace_path: &Path) -> String {
        let config = RemoteWorkspaceConfig {
            folders: vec![RemoteWorkspaceFolder {
                name: Some(workspace_path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Remote Workspace".to_string())),
                path: workspace_path.to_string_lossy().to_string(),
                uri: Some("vscode-remote".to_string()),
            }],
            settings: HashMap::new(),
            extensions: RemoteExtensionsConfig {
                install: Vec::new(),
                uninstall: Vec::new(),
                enable: Vec::new(),
                disable: Vec::new(),
            },
            launch: None,
            tasks: None,
        };

        serde_json::to_string_pretty(&config).unwrap_or_else(|_| "{}".to_string())
    }

    /// Check if a remote file exists
    async fn remote_file_exists(&self, path: &Path) -> Result<bool> {
        let conn = self.connection.lock().unwrap();
        if !conn.is_connected() {
            return Ok(false);
        }

        let result = conn.execute(&format!("test -f {} && echo 'exists'", shell_escape(path.to_string_lossy().as_ref())))
            .await;

        match result {
            Ok(output) => Ok(output.contains("exists")),
            Err(_) => Ok(false),
        }
    }
}

/// Shell escape helper
fn shell_escape(s: impl AsRef<str>) -> String {
    let s = s.as_ref();
    if s.chars().all(|c| c.is_alphanumeric() || c == '/' || c == '_' || c == '-' || c == '.') {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\"'\"'"))
    }
}