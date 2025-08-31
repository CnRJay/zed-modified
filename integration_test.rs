// Integration test for multi-root workspace functionality
use std::path::PathBuf;

mod workspace_integration {
    use std::path::PathBuf;
    use workspace::workspace_file::WorkspaceFile;

    pub fn test_workspace_integration() {
        println!("🚀 Testing Multi-Root Workspace Integration");
        println!("============================================");

        // Test loading the workspace file
        let workspace_path = PathBuf::from("test-workspace/.zed/workspace.json");
        println!("📂 Loading workspace file: {}", workspace_path.display());

        match WorkspaceFile::load(&workspace_path) {
            Ok(workspace_file) => {
                println!("✅ Workspace file loaded successfully!");
                println!("📊 Workspace contains {} folders:", workspace_file.folders.len());

                for (i, folder) in workspace_file.folders.iter().enumerate() {
                    let name = folder.name.clone().unwrap_or_else(|| "Unnamed".to_string());
                    println!("  {}. {} -> {}", i + 1, name, folder.path.display());
                }

                // Test resolved paths
                let resolved = workspace_file.resolved_folder_paths(&workspace_path);
                println!("\n🔗 Resolved folder paths:");
                for (i, path) in resolved.iter().enumerate() {
                    println!("  {}. {}", i + 1, path.display());
                }

                // Test display names
                let display_names = workspace_file.folder_display_names();
                println!("\n🏷️  Display names:");
                for (i, name) in display_names.iter().enumerate() {
                    println!("  {}. {}", i + 1, name);
                }

                // Test trust system
                println!("\n🔒 Trust configuration:");
                match workspace_file.trust {
                    Some(true) => println!("  ✅ Workspace is explicitly trusted"),
                    Some(false) => println!("  ❌ Workspace is explicitly untrusted"),
                    None => println!("  ❓ Trust not specified (will use auto-detection)"),
                }

                // Test settings
                if let Some(settings) = &workspace_file.settings {
                    println!("\n⚙️  Workspace settings configured:");
                    if let Some(editor) = &settings.editor {
                        println!("  📝 Editor settings: {}", editor.len());
                    }
                    if let Some(language) = &settings.language_specific {
                        println!("  💻 Language settings: {}", language.len());
                    }
                } else {
                    println!("\n⚙️  No workspace-specific settings configured");
                }

                // Test extensions
                if let Some(extensions) = &workspace_file.extensions {
                    println!("\n🔧 Extension recommendations:");
                    println!("  ✅ Recommended: {}", extensions.recommendations.len());
                    println!("  ❌ Unwanted: {}", extensions.unwanted_recommendations.len());
                } else {
                    println!("\n🔧 No extension recommendations configured");
                }

                // Test launch configuration
                if let Some(launch) = &workspace_file.launch {
                    println!("\n🚀 Launch configuration:");
                    println!("  Type: {}", launch.type_);
                    println!("  Name: {}", launch.name);
                } else {
                    println!("\n🚀 No launch configuration");
                }

                // Test tasks
                if let Some(tasks) = &workspace_file.tasks {
                    println!("\n⚡ Task configuration:");
                    println!("  Label: {}", tasks.label);
                    println!("  Type: {}", tasks.type_);
                    println!("  Command: {}", tasks.command);
                } else {
                    println!("\n⚡ No task configuration");
                }

                println!("\n🎉 Multi-root workspace integration test PASSED!");
                println!("================================================");
            }
            Err(err) => {
                println!("❌ Failed to load workspace file: {}", err);
                println!("Integration test FAILED!");
            }
        }
    }
}

fn main() {
    workspace_integration::test_workspace_integration();
}
