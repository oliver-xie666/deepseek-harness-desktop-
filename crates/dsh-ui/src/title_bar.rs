use gpui::{div, prelude::*, rgb, Context, IntoElement, Window, WindowControlArea};

/// Slim native title strip: a full-width drag region carrying only the window
/// controls. The brand mark lives in the sidebar (official `SidebarRoot`
/// layout), so this bar stays chrome-only.
pub struct TitleBar {
    pub workspace_name: String,
}

impl TitleBar {
    pub fn new(workspace: &str) -> Self {
        Self {
            workspace_name: workspace.to_string(),
        }
    }
}

impl Render for TitleBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h_0()
            .w_full()
            .bg(rgb(0xf9fafb))
            .border_b_1()
            .border_color(rgb(0xe5e7eb))
            .flex()
            .items_center()
            .justify_end()
            .window_control_area(WindowControlArea::Drag)
            // Right: window controls
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .pr_1()
                    // Minimize
                    .child(
                        div()
                            .size_7()
                            .rounded_md()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xs()
                            .text_color(rgb(0x81858c))
                            .hover(|s| s.bg(rgb(0xf1f3f5)).text_color(rgb(0x0f1115)))
                            .cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, |_, window, _| {
                                window.minimize_window();
                            })
                            .child("─"),
                    )
                    // Maximize / restore
                    .child(
                        div()
                            .size_7()
                            .rounded_md()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xs()
                            .text_color(rgb(0x81858c))
                            .hover(|s| s.bg(rgb(0xf1f3f5)).text_color(rgb(0x0f1115)))
                            .cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, |_, window, _| {
                                window.zoom_window();
                            })
                            .child("▢"),
                    )
                    // Close
                    .child(
                        div()
                            .size_7()
                            .rounded_md()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xs()
                            .text_color(rgb(0x81858c))
                            .hover(|s| s.bg(rgb(0xef4444)).text_color(rgb(0xffffff)))
                            .cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, |_, window, _| {
                                window.remove_window();
                            })
                            .child("✕"),
                    ),
            )
    }
}
