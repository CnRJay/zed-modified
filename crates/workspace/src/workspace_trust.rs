use std::collections::HashSet;
use std::path::{Path, PathBuf};

use gpui::{App, Global, SharedString, Task};
use serde::{Deserialize, Serialize};

/// Workspace trust levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    /// Workspace is explicitly trusted
    Trusted,
    /// Workspace is explicitly untrusted
    Untrusted,
    /// Trust level not yet determined
    Unknown,
}

/// Trust decision for a workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustDecision {
    /// Absolute path to the workspace file or directory
    pub path: PathBuf,
    /// Trust level
    pub level: TrustLevel,
    /// When the decision was made (timestamp)
    pub timestamp: u64,
}

/// Global workspace trust store
#[derive(Debug, Clone)]
pub struct WorkspaceTrustStore {
    /// Set of trusted workspace paths
    trusted_paths: HashSet<PathBuf>,
    /// Set of explicitly untrusted workspace paths
    untrusted_paths: HashSet<PathBuf>,
}

impl Global for WorkspaceTrustStore {}

impl WorkspaceTrustStore {
    /// Initialize the trust store
    pub fn init(cx: &mut App) {
        let store = Self {
            trusted_paths: HashSet::new(),
            untrusted_paths: HashSet::new(),
        };
        cx.set_global(store);
    }

    /// Get the global trust store
    pub fn global(cx: &App) -> &WorkspaceTrustStore {
        cx.global::<WorkspaceTrustStore>()
    }

    /// Check if a workspace path is trusted
    pub fn is_trusted(&self, path: &Path) -> TrustLevel {
        // Check for exact path match first
        if self.trusted_paths.contains(path) {
            return TrustLevel::Trusted;
        }

        if self.untrusted_paths.contains(path) {
            return TrustLevel::Untrusted;
        }

        // Check if any parent directory is trusted/untrusted
        for trusted_path in &self.trusted_paths {
            if path.starts_with(trusted_path) {
                return TrustLevel::Trusted;
            }
        }

        for untrusted_path in &self.untrusted_paths {
            if path.starts_with(untrusted_path) {
                return TrustLevel::Untrusted;
            }
        }

        TrustLevel::Unknown
    }

    /// Trust a workspace path
    pub fn trust_path(&mut self, path: PathBuf) {
        self.trusted_paths.insert(path);
    }

    /// Untrust a workspace path
    pub fn untrust_path(&mut self, path: PathBuf) {
        self.untrusted_paths.insert(path.clone());
        self.trusted_paths.remove(&path);
    }

    /// Remove trust decision for a path
    pub fn remove_decision(&mut self, path: &PathBuf) {
        self.trusted_paths.remove(path);
        self.untrusted_paths.remove(path);
    }
}

/// Trust prompt dialog
pub struct WorkspaceTrustPrompt;

impl WorkspaceTrustPrompt {
    /// Show trust prompt for an untrusted workspace
    pub fn show(workspace_path: PathBuf, workspace_name: SharedString, cx: &mut App) -> Task<()> {
        cx.spawn(async move |_cx| {
            // TODO: Show actual UI dialog
            // For now, we'll just log the prompt
            log::info!("Workspace trust prompt for: {} at {}", workspace_name, workspace_path.display());

            // In a real implementation, this would show a modal dialog
            // asking the user to trust or reject the workspace
        })
    }
}

/// Trust utilities
pub struct WorkspaceTrust;

impl WorkspaceTrust {
    /// Check if a workspace should be trusted based on its configuration
    pub fn should_auto_trust(workspace_file: &super::workspace_file::WorkspaceFile, workspace_path: &Path) -> bool {
        // Trust if explicitly marked as trusted
        if workspace_file.trust == Some(true) {
            return true;
        }

        // Trust if the workspace file is in a well-known safe location
        if let Some(parent) = workspace_path.parent() {
            let safe_dirs = ["~/Documents", "~/Projects", "~/src", "~/work"];
            if safe_dirs.iter().any(|safe_dir| parent.to_string_lossy().contains(safe_dir)) {
                return true;
            }
        }

        false
    }

    /// Determine if workspace functionality should be restricted
    pub fn should_restrict_functionality(trust_level: TrustLevel) -> bool {
        matches!(trust_level, TrustLevel::Untrusted)
    }

    /// Get restricted features for untrusted workspaces
    pub fn get_restricted_features() -> Vec<&'static str> {
        vec![
            "Task execution",
            "Terminal commands",
            "Extension auto-installation",
            "File system operations outside workspace",
            "Network requests",
        ]
    }
}
