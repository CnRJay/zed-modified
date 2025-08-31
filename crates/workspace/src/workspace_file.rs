use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

/// Represents a folder entry in a multi-root workspace
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceFolder {
    /// The path to the folder. Can be absolute or relative to the workspace file.
    pub path: PathBuf,
    /// Optional display name for the folder in the UI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Workspace-specific settings that override global settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSettings {
    /// Editor settings specific to this workspace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<serde_json::Value>,
    /// Language-specific settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_specific: Option<serde_json::Value>,
    /// Other workspace-specific settings
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

/// Extension recommendations for this workspace
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionRecommendations {
    /// Extensions that should be installed and enabled for this workspace
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub recommendations: Vec<String>,
    /// Extensions that are unwanted in this workspace
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub unwanted_recommendations: Vec<String>,
}

/// Launch configurations for debugging
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchConfiguration {
    /// Type of the launch configuration (e.g., "node", "python", "cppdbg")
    #[serde(rename = "type")]
    pub type_: String,
    /// Name of the configuration
    pub name: String,
    /// Request type ("launch" or "attach")
    pub request: String,
    /// Other configuration properties
    #[serde(flatten)]
    pub properties: serde_json::Map<String, serde_json::Value>,
}

/// Tasks configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskConfiguration {
    /// Label for the task
    pub label: String,
    /// Type of the task ("shell", "process")
    #[serde(rename = "type")]
    pub type_: String,
    /// Command to run
    pub command: String,
    /// Arguments for the command
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub args: Vec<String>,
    /// Working directory for the task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,
    /// Other task properties
    #[serde(flatten)]
    pub properties: serde_json::Map<String, serde_json::Value>,
}

/// The main workspace configuration file structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFile {
    /// Folders to include in the workspace
    pub folders: Vec<WorkspaceFolder>,
    /// Workspace-specific settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<WorkspaceSettings>,
    /// Extension recommendations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<ExtensionRecommendations>,
    /// Launch configurations for debugging
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch: Option<LaunchConfiguration>,
    /// Tasks configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks: Option<TaskConfiguration>,
    /// Remote development configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_authority: Option<String>,
    /// Whether to trust this workspace automatically
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust: Option<bool>,
    /// Other custom properties
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

impl WorkspaceFile {
    /// Load a workspace file from disk
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read workspace file: {}", path.as_ref().display()))?;

        let workspace: WorkspaceFile = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse workspace file: {}", path.as_ref().display()))?;

        Ok(workspace)
    }

    /// Save the workspace file to disk
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize workspace configuration")?;

        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write workspace file: {}", path.as_ref().display()))?;

        Ok(())
    }

    /// Get all folder paths, resolving relative paths against the workspace file location
    pub fn resolved_folder_paths<P: AsRef<Path>>(&self, workspace_file_path: P) -> Vec<PathBuf> {
        let workspace_file_path = workspace_file_path.as_ref();

        // The workspace root is the parent of the .zed directory
        let base_path = workspace_file_path
            .parent() // Get .zed directory
            .and_then(|zed_dir| zed_dir.parent()) // Get workspace root
            .unwrap_or(Path::new(""));

        self.folders
            .iter()
            .map(|folder| {
                if folder.path.is_absolute() {
                    folder.path.clone()
                } else {
                    base_path.join(&folder.path)
                }
            })
            .collect()
    }

    /// Get display names for folders, falling back to folder names if not specified
    pub fn folder_display_names(&self) -> Vec<String> {
        self.folders
            .iter()
            .enumerate()
            .map(|(index, folder)| {
                folder.name.clone().unwrap_or_else(|| {
                    folder.path.file_name()
                        .and_then(|name| name.to_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| format!("Folder {}", index + 1))
                })
            })
            .collect()
    }

    /// Create a new workspace file with the given folders
    pub fn new(folders: Vec<WorkspaceFolder>) -> Self {
        Self {
            folders,
            settings: None,
            extensions: None,
            launch: None,
            tasks: None,
            remote_authority: None,
            trust: None,
            other: serde_json::Map::new(),
        }
    }

    /// Add a folder to the workspace
    pub fn add_folder(&mut self, path: PathBuf, name: Option<String>) {
        self.folders.push(WorkspaceFolder { path, name });
    }

    /// Remove a folder from the workspace by index
    pub fn remove_folder(&mut self, index: usize) {
        if index < self.folders.len() {
            self.folders.remove(index);
        }
    }

    /// Update a folder's path and/or name
    pub fn update_folder(&mut self, index: usize, path: PathBuf, name: Option<String>) {
        if index < self.folders.len() {
            self.folders[index] = WorkspaceFolder { path, name };
        }
    }
}

impl Default for WorkspaceFile {
    fn default() -> Self {
        Self::new(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_workspace_file_serialization() {
        let mut workspace = WorkspaceFile::new(vec![
            WorkspaceFolder {
                path: PathBuf::from("/path/to/project1"),
                name: Some("Project 1".to_string()),
            },
            WorkspaceFolder {
                path: PathBuf::from("./project2"),
                name: None,
            },
        ]);

        workspace.settings = Some(WorkspaceSettings {
            editor: Some(serde_json::json!({
                "fontSize": 14,
                "tabSize": 2
            })),
            language_specific: Some(serde_json::json!({
                "rust": {
                    "formatOnSave": true
                }
            })),
            other: serde_json::Map::new(),
        });

        workspace.extensions = Some(ExtensionRecommendations {
            recommendations: vec![
                "rust-lang.rust".to_string(),
                "ms-python.python".to_string(),
            ],
            unwanted_recommendations: vec![],
        });

        let json = serde_json::to_string_pretty(&workspace).unwrap();
        let deserialized: WorkspaceFile = serde_json::from_str(&json).unwrap();

        assert_eq!(workspace, deserialized);
    }

    #[test]
    fn test_resolved_folder_paths() {
        let workspace = WorkspaceFile::new(vec![
            WorkspaceFolder {
                path: PathBuf::from("/absolute/path"),
                name: Some("Absolute".to_string()),
            },
            WorkspaceFolder {
                path: PathBuf::from("relative/path"),
                name: Some("Relative".to_string()),
            },
        ]);

        let workspace_file_path = PathBuf::from("/workspace/.zed/workspace.json");
        let resolved = workspace.resolved_folder_paths(&workspace_file_path);

        assert_eq!(resolved[0], PathBuf::from("/absolute/path"));
        assert_eq!(resolved[1], PathBuf::from("/workspace/relative/path"));
    }

    #[test]
    fn test_folder_display_names() {
        let workspace = WorkspaceFile::new(vec![
            WorkspaceFolder {
                path: PathBuf::from("/path/to/project1"),
                name: Some("My Project".to_string()),
            },
            WorkspaceFolder {
                path: PathBuf::from("/path/to/project2"),
                name: None,
            },
        ]);

        let names = workspace.folder_display_names();
        assert_eq!(names[0], "My Project");
        assert_eq!(names[1], "project2");
    }
}
