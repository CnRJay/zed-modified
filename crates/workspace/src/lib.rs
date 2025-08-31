pub mod dock;
pub mod history_manager;
pub mod invalid_buffer_view;
pub mod item;
mod modal_layer;
pub mod notifications;
pub mod pane;
pub mod pane_group;
mod path_list;
mod persistence;
pub mod searchable;
pub mod shared_screen;
mod status_bar;
pub mod tasks;
mod theme_preview;
mod toast_layer;
mod toolbar;
pub mod workspace_file;
pub mod workspace_trust;
pub mod workspace_ui;
mod workspace_settings;

pub use crate::notifications::NotificationFrame;
pub use dock::Panel;
pub use path_list::PathList;
pub use toast_layer::{ToastAction, ToastLayer, ToastView};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_workspace_file_loading() {
        let workspace_path = PathBuf::from("../../test-workspace/.zed/workspace.json");
        match workspace_file::WorkspaceFile::load(&workspace_path) {
            Ok(workspace_file) => {
                assert_eq!(workspace_file.folders.len(), 3);

                // Check folder names
                assert_eq!(workspace_file.folders[0].name.as_deref(), Some("Test Workspace Root"));
                assert_eq!(workspace_file.folders[1].name.as_deref(), Some("Zed Source Code"));
                assert_eq!(workspace_file.folders[2].name.as_deref(), Some("Example Project"));

                // Check trust setting
                assert_eq!(workspace_file.trust, Some(true));

                // Check display names
                let display_names = workspace_file.folder_display_names();
                assert_eq!(display_names.len(), 3);
                assert_eq!(display_names[0], "Test Workspace Root");
                assert_eq!(display_names[1], "Zed Source Code");
                assert_eq!(display_names[2], "Example Project");

                println!("✅ Workspace file loading test passed!");
            }
            Err(err) => {
                panic!("Failed to load workspace file: {}", err);
            }
        }
    }

    #[test]
    fn test_trust_system() {
        let workspace_path = PathBuf::from("../../test-workspace/.zed/workspace.json");

        match workspace_file::WorkspaceFile::load(&workspace_path) {
            Ok(workspace_file) => {
                // Test auto-trust for explicitly trusted workspace
                let should_trust = workspace_trust::WorkspaceTrust::should_auto_trust(
                    &workspace_file,
                    &workspace_path
                );
                assert!(should_trust);

                // Test trust level determination
                let trust_level = workspace_trust::WorkspaceTrust::should_restrict_functionality(
                    workspace_trust::TrustLevel::Trusted
                );
                assert!(!trust_level);

                let trust_level = workspace_trust::WorkspaceTrust::should_restrict_functionality(
                    workspace_trust::TrustLevel::Untrusted
                );
                assert!(trust_level);

                println!("✅ Trust system test passed!");
            }
            Err(err) => {
                panic!("Failed to load workspace file: {}", err);
            }
        }
    }
}
