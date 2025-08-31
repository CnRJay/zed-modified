//! Remote Terminal Integration
//!
//! Provides SSH terminal integration with Zed's terminal system.

use gpui::{div, px, FocusHandle, Focusable, IntoElement, Render};
use ui::{prelude::*, Label};

/// Remote Terminal View
pub struct RemoteTerminal {
    /// Focus handle
    focus_handle: FocusHandle,
}

impl RemoteTerminal {
    /// Create a new remote terminal
    pub fn new(cx: &mut gpui::Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        Self {
            focus_handle,
        }
    }
}

impl Focusable for RemoteTerminal {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for RemoteTerminal {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .flex_col()
            .w_full()
            .h(px(300.0))
            .bg(theme.colors().background)
            .border_1()
            .border_color(theme.colors().border)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .px(px(12.0))
                    .py(px(8.0))
                    .child(
                        Label::new("SSH Remote Terminal - Coming Soon!")
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
                        Label::new("This will provide SSH terminal integration")
                            .size(LabelSize::Small)
                            .color(Color::Muted)
                    )
            )
    }
}