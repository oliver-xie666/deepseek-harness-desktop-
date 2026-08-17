use gpui::{
    div, prelude::*, rgb, Context, FontWeight, IntoElement, Window, WindowControlArea,
};

pub struct TitleBar {
    pub workspace_name: String,
    pub active_model: String,
    pub is_connected: bool,
}

impl TitleBar {
    pub fn new(workspace: &str, model: &str, is_connected: bool) -> Self {
        Self {
            workspace_name: workspace.to_string(),
            active_model: model.to_string(),
            is_connected,
        }
    }
}

impl Render for TitleBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h_10()
            .w_full()
            .bg(rgb(0x0f1115))
            .border_b_1()
            .border_color(rgb(0x1f2228))
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .window_control_area(WindowControlArea::Drag)
            // Left: DeepSeek Brand & Workspace
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x4176e6))
                            .child("⚡ DeepSeek Harness"),
                    )
                    .child(div().text_xs().text_color(rgb(0x61666b)).child("•"))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x979da6))
                            .child(self.workspace_name.clone()),
                    ),
            )
            // Right: Model Pill, Status Pill, Settings Pill & Controls
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2p5()
                    // Model Tag
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .px_2p5()
                            .py_1()
                            .rounded_md()
                            .bg(rgb(0x15171b))
                            .border_1()
                            .border_color(rgb(0x282c34))
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0xe4e4e7))
                            .child("🤖")
                            .child(self.active_model.clone()),
                    )
                    // Daemon Status
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .px_2p5()
                            .py_1()
                            .rounded_md()
                            .bg(rgb(0x15171b))
                            .border_1()
                            .border_color(rgb(0x282c34))
                            .text_xs()
                            .text_color(rgb(0xe4e4e7))
                            .child(div().size_2().rounded_full().bg(rgb(0x22c55e)))
                            .child("Daemon: Online"),
                    )
                    // Settings Pill
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .px_2p5()
                            .py_1()
                            .rounded_md()
                            .bg(rgb(0x15171b))
                            .border_1()
                            .border_color(rgb(0x282c34))
                            .hover(|s| s.bg(rgb(0x1f2228)))
                            .cursor_pointer()
                            .text_xs()
                            .text_color(rgb(0xe4e4e7))
                            .child("⚙️ Settings"),
                    )
                    // Divider
                    .child(div().w_px().h_4().bg(rgb(0x282c34)))
                    // Window Controls: Minimize & Close
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .size_6()
                                    .rounded_md()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_xs()
                                    .text_color(rgb(0x979da6))
                                    .hover(|s| s.bg(rgb(0x1f2228)).text_color(rgb(0xffffff)))
                                    .cursor_pointer()
                                    .child("─"),
                            )
                            .child(
                                div()
                                    .size_6()
                                    .rounded_md()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_xs()
                                    .text_color(rgb(0x979da6))
                                    .hover(|s| s.bg(rgb(0xef4444)).text_color(rgb(0xffffff)))
                                    .cursor_pointer()
                                    .on_mouse_down(gpui::MouseButton::Left, |_, window, _| {
                                        window.remove_window();
                                    })
                                    .child("✕"),
                            ),
                    ),
            )
    }
}
