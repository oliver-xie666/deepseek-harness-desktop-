use gpui::{div, prelude::*, rgb, Context, FontWeight, IntoElement, Window};

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
        let status_color = if self.is_connected {
            rgb(0x22c55e)
        } else {
            rgb(0xef4444)
        };

        div()
            .h_9()
            .w_full()
            .bg(rgb(0x18181b))
            .border_b_1()
            .border_color(rgb(0x27272a))
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .child(
                // Left: Logo & App Name
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x38bdf8))
                            .child("⚡ DeepSeek Harness"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x71717a))
                            .child(format!("• {}", self.workspace_name)),
                    ),
            )
            .child(
                // Center / Right Controls
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    // Active Model Badge
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .px_2p5()
                            .py_0p5()
                            .rounded_md()
                            .bg(rgb(0x27272a))
                            .border_1()
                            .border_color(rgb(0x3f3f46))
                            .text_xs()
                            .text_color(rgb(0xe4e4e7))
                            .child("🤖")
                            .child(self.active_model.clone()),
                    )
                    // Daemon Status Indicator
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .text_xs()
                            .text_color(rgb(0xa1a1aa))
                            .child(div().size_2().rounded_full().bg(status_color))
                            .child(if self.is_connected { "Daemon: Online" } else { "Daemon: Disconnected" }),
                    )
                    // Settings Button
                    .child(
                        div()
                            .px_2()
                            .py_0p5()
                            .rounded_md()
                            .bg(rgb(0x27272a))
                            .hover(|s| s.bg(rgb(0x3f3f46)))
                            .text_xs()
                            .text_color(rgb(0xe4e4e7))
                            .cursor_pointer()
                            .child("⚙️ Settings"),
                    ),
            )
    }
}
