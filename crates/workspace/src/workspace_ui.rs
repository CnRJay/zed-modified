use std::path::PathBuf;

use gpui::{
    div, DismissEvent, App, Context, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement, Render,
    Styled, Subscription, WeakEntity, Window,
};
use crate::{Workspace, workspace_file::WorkspaceFile};

/// Action for workspace management
#[derive(Clone, PartialEq)]
pub struct OpenWorkspaceManager;

/// Simple workspace manager
pub struct WorkspaceManager {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    workspace_file: Option<WorkspaceFile>,
    workspace_path: Option<PathBuf>,
    _subscriptions: Vec<Subscription>,
}

impl WorkspaceManager {
    pub fn new(workspace: WeakEntity<Workspace>, _window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        let mut subscriptions = Vec::new();

        // Subscribe to workspace changes
        if let Some(workspace) = workspace.upgrade() {
            subscriptions.push(cx.subscribe(&workspace, |this, _, _, cx| {
                this.update_workspace_info(cx);
            }));
        }

        let mut this = Self {
            workspace,
            focus_handle,
            workspace_file: None,
            workspace_path: None,
            _subscriptions: subscriptions,
        };

        this.update_workspace_info(cx);

        this
    }

    fn update_workspace_info(&mut self, cx: &mut Context<Self>) {
        if let Some(workspace) = self.workspace.upgrade() {
            self.workspace_file = workspace.read(cx).workspace_file().cloned();
            self.workspace_path = workspace.read(cx).workspace_file_path().cloned();
        }
    }


}

impl EventEmitter<DismissEvent> for WorkspaceManager {}

impl Focusable for WorkspaceManager {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for WorkspaceManager {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut content = String::new();

        if let Some(workspace_file) = &self.workspace_file {
            content.push_str("Workspace Folders:\n");
            for folder in &workspace_file.folders {
                let folder_name = folder.name.clone().unwrap_or_else(|| {
                    folder.path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unnamed")
                        .to_string()
                });
                content.push_str(&format!("- {}: {}\n", folder_name, folder.path.display()));
            }
        } else {
            content.push_str("No workspace file loaded");
        }

        if let Some(workspace_path) = &self.workspace_path {
            content.push_str(&format!("\nWorkspace file: {}", workspace_path.display()));
        }

        div()
            .size_full()
            .p_4()
            .child(content)
            .into_element()
    }
}

/// Show workspace information (basic implementation)
pub fn show_workspace_manager(workspace: WeakEntity<Workspace>, _window: &mut Window, cx: &mut App) {
    if let Some(workspace) = workspace.upgrade() {
        let workspace_file = workspace.read(cx).workspace_file().cloned();
        let workspace_path = workspace.read(cx).workspace_file_path().cloned();
        let trust_level = workspace.read(cx).trust_level();
        let should_restrict = workspace.read(cx).should_restrict_functionality();

        log::info!("=== Workspace Manager ===");
        if let Some(workspace_file) = workspace_file {
            log::info!("Workspace Folders:");
            for folder in &workspace_file.folders {
                let folder_name = folder.name.clone().unwrap_or_else(|| {
                    folder.path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unnamed")
                        .to_string()
                });
                log::info!("- {}: {}", folder_name, folder.path.display());
            }
        } else {
            log::info!("No workspace file loaded");
        }

        if let Some(workspace_path) = workspace_path {
            log::info!("Workspace file: {}", workspace_path.display());
        }

        log::info!("Trust level: {:?}", trust_level);
        log::info!("Functionality restricted: {}", should_restrict);
        log::info!("=== End Workspace Manager ===");
    } else {
        log::warn!("Workspace entity is no longer available");
    }
}
