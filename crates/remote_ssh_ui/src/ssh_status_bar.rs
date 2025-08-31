//! SSH Status Bar Component
//!
//! Displays SSH connection status in the status bar and provides quick access to SSH features.

use gpui::{div, px, FocusHandle, Focusable, IntoElement, Render};
use ui::{prelude::*, Label, Icon, IconName, IconSize};

/// SSH Status Bar View
pub struct SshStatusBar {
    /// Focus handle
    focus_handle: FocusHandle,
}

impl SshStatusBar {
    /// Create a new SSH status bar
    pub fn new(cx: &mut gpui::Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        Self {
            focus_handle,
        }
    }

    /// Get status display text
    fn status_text(&self) -> String {
        // In a full implementation, this would check active SSH connections
        // For now, show a ready state indicating SSH is available
        "SSH: Available".to_string()
    }
}

impl Focusable for SshStatusBar {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SshStatusBar {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let status_text = self.status_text();

        div()
            .flex()
            .items_center()
            .px(px(8.0))
            .py(px(4.0))
            .gap(px(6.0))
            .bg(theme.colors().background)
            .child(
                Icon::new(IconName::BoltOutlined) // SSH/Network connectivity icon
                    .size(IconSize::Small)
                    .color(Color::Default)
            )
            .child(
                Label::new(status_text)
                    .size(LabelSize::Small)
                    .color(Color::Default)
            )
    }
}