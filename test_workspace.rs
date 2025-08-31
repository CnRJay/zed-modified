use std::path::PathBuf;
use workspace::{workspace_file::WorkspaceFile, workspace_trust};

fn main() {
    println!("=== Testing Workspace File Loading ===");

    // Test loading the workspace file
    let workspace_path = PathBuf::from("test-workspace/.zed/workspace.json");
    match WorkspaceFile::load(&workspace_path) {
        Ok(workspace_file) => {
            println!("✅ Successfully loaded workspace file!");
            println!("📁 Folders:");
            for folder in &workspace_file.folders {
                let name = folder.name.clone().unwrap_or_else(|| "Unnamed".to_string());
                println!("  - {}: {}", name, folder.path.display());
            }

            // Test trust checking
            let trust_level = workspace_trust::WorkspaceTrust::should_auto_trust(
                &workspace_file,
                &workspace_path
            );

            println!("🔒 Trust level: {:?}", trust_level);
            println!("🚫 Should restrict functionality: {}", workspace_trust::WorkspaceTrust::should_restrict_functionality(trust_level));

            // Test folder display names
            let display_names = workspace_file.folder_display_names();
            println!("🏷️  Folder display names: {:?}", display_names);

            // Test resolved folder paths
            let resolved_paths = workspace_file.resolved_folder_paths(&workspace_path);
            println!("📂 Resolved paths:");
            for path in resolved_paths {
                println!("  - {}", path.display());
            }

        }
        Err(err) => {
            println!("❌ Failed to load workspace file: {}", err);
        }
    }

    println!("\n=== Test Complete ===");
}
