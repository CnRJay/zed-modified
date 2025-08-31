use gpui::{App, Context, FocusHandle, Focusable, Render, Styled, ParentElement};

/// Marketplace view for browsing and installing extensions
pub struct MarketplaceView {
    focus_handle: FocusHandle,
}

impl MarketplaceView {
    /// Create a new marketplace view
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        Self {
            focus_handle,
        }
    }
}

impl Render for MarketplaceView {
    fn render(&mut self, _cx: &mut gpui::Window, _context: &mut Context<Self>) -> impl gpui::IntoElement {
        gpui::div()
            .size_full()
            .child("Enhanced Extension Marketplace - Coming Soon!")
    }
}

impl Focusable for MarketplaceView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}