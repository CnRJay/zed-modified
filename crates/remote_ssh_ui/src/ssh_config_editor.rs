//! SSH Configuration Editor UI Component
//!
//! Provides a UI for editing SSH configuration files with syntax highlighting and validation.

use gpui::{div, px, FocusHandle, Focusable, IntoElement, Render};
use ui::{prelude::*, Label};

/// SSH Configuration Editor View
pub struct SshConfigEditor {
    /// Focus handle
    focus_handle: FocusHandle,
}

impl SshConfigEditor {
    /// Create a new SSH config editor
    pub fn new(cx: &mut gpui::Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        Self {
            focus_handle,
        }
    }
}

impl Focusable for SshConfigEditor {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SshConfigEditor {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .flex_col()
            .w(px(500.0))
            .h(px(400.0))
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
                        Label::new("SSH Configuration Editor - Coming Soon!")
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
                        Label::new("This will allow you to edit SSH configuration files")
                            .size(LabelSize::Small)
                            .color(Color::Muted)
                    )
            )
    }
}