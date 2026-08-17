use gpui::{div, prelude::*, rgb, Context, FontWeight, IntoElement, SharedString, Window};

pub struct TitleBar {
    pub current_workspace: SharedString,
    pub current_model: SharedString,
    pub is_daemon_connected: bool,
}

impl TitleBar {
    pub fn new(workspace: &str, model: &str, is_connected: bool) -> Self {
        Self {
            current_workspace: workspace.to_string().into(),
            current_model: model.to_string().into(),
            is_daemon_connected: is_connected,
        }
    }
}

impl Render for TitleBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let status_color = if self.is_daemon_connected {
            rgb(0x48bb78) // green
        } else {
            rgb(0xf56565) // red
        };

        div()
            .h_10()
            .w_full()
            .bg(rgb(0x18181b)) // zinc-900
            .border_b_1()
            .border_color(rgb(0x27272a)) // zinc-800
            .flex()
            .items_center()
            .justify_between()
            .px_4()
            .text_sm()
            .text_color(rgb(0xd4d4d8)) // zinc-300
            // Left: Logo & Workspace
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x38bdf8)) // sky-400
                            .child("⚡ DeepSeek Harness"),
                    )
                    .child(
                        div()
                            .px_2()
                            .py_0p5()
                            .rounded_md()
                            .bg(rgb(0x27272a))
                            .text_xs()
                            .child(format!("📁 {}", self.current_workspace)),
                    ),
            )
            // Center: Model Switcher
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_1()
                    .rounded_md()
                    .bg(rgb(0x27272a))
                    .border_1()
                    .border_color(rgb(0x3f3f46))
                    .child(format!("🤖 {}", self.current_model)),
            )
            // Right: Status indicator
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .size_2p5()
                            .rounded_full()
                            .bg(status_color),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xa1a1aa))
                            .child(if self.is_daemon_connected {
                                "Daemon Ready"
                            } else {
                                "Daemon Disconnected"
                            }),
                    ),
            )
    }
}
