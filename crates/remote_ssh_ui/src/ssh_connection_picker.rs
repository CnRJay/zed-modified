//! SSH Connection Picker UI Component
//!
//! Provides a UI for selecting, managing, and connecting to SSH remote hosts.

use gpui::{div, px, FocusHandle, Focusable, IntoElement, Render};
use ui::{prelude::*, Label};

/// SSH Connection Picker View
pub struct SshConnectionPicker {
    /// Focus handle
    focus_handle: FocusHandle,
}

impl SshConnectionPicker {
    /// Create a new SSH connection picker
    pub fn new(cx: &mut gpui::Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        Self {
            focus_handle,
        }
    }
}

impl Focusable for SshConnectionPicker {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SshConnectionPicker {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .flex_col()
            .w(px(400.0))
            .h(px(300.0))
            .bg(theme.colors().background)
            .border_1()
            .border_color(theme.colors().border)
            .rounded(px(6.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .px(px(12.0))
                    .py(px(8.0))
                    .child(
                        Label::new("SSH Connection Picker - Coming Soon!")
                            .size(LabelSize::Default)
                            .color(Color::Default)
                    )
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .px(px(12.0))
                    .py(px(8.0))
                    .child(
                        Label::new("This will allow you to select and connect to SSH hosts")
                            .size(LabelSize::Small)
                            .color(Color::Muted)
                    )
            )
    }
}
