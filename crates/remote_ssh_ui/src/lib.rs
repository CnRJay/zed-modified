//! Remote SSH UI Components for Zed
//!
//! This crate provides UI components for SSH remote development,
//! including connection management, configuration editing, and status displays.

mod ssh_connection_picker;
mod ssh_config_editor;
mod ssh_status_bar;
mod ssh_key_manager;
mod remote_terminal;

pub use ssh_connection_picker::*;
pub use ssh_config_editor::*;
pub use ssh_status_bar::*;
pub use ssh_key_manager::*;
pub use remote_terminal::*;

/// Initialize the SSH UI system
pub fn init(_cx: &mut gpui::App) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("SSH UI system initialized - Ready for SSH remote development!");
    Ok(())
}
