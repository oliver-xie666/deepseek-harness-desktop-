use gpui::{div, prelude::*, px, rgb, Context, IntoElement, Window, WindowControlArea};

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
            .h(px(38.0))
            .w_full()
            .bg(rgb(0xf9fafb))
            .border_b_1()
            .border_color(rgb(0xe5e7eb))
            .flex()
            .items_center()
            // Keep the blank area draggable while the traffic lights own their clicks.
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .window_control_area(WindowControlArea::Drag),
            )
            // Right: macOS traffic-light controls with native window actions.
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .pr_3()
                    // Minimize
                    .child(
                        div()
                            .size(px(15.0))
                            .rounded_full()
                            .bg(rgb(0xfebc2e))
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|s| s.bg(rgb(0xffca55)))
                            .cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, |_, window, _| {
                                window.minimize_window();
                            })
                            .child(div().w(px(7.0)).h(px(1.0)).bg(rgb(0x7a5a0b))),
                    )
                    // Maximize / restore
                    .child(
                        div()
                            .size(px(15.0))
                            .rounded_full()
                            .bg(rgb(0x28c840))
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|s| s.bg(rgb(0x4edc61)))
                            .cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, |_, window, _| {
                                window.zoom_window();
                            })
                            .child(div().size(px(7.0)).border_1().border_color(rgb(0x0d6b22))),
                    )
                    // Close
                    .child(
                        div()
                            .size(px(15.0))
                            .rounded_full()
                            .bg(rgb(0xff5f57))
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|s| s.bg(rgb(0xff7b74)))
                            .cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, |_, window, _| {
                                window.remove_window();
                            })
                            .child(crate::icons::close(8.0, rgb(0xffffff))),
                    ),
            )
    }
}
