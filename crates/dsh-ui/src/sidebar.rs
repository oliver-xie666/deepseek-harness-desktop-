use gpui::{div, prelude::*, rgb, Context, FontWeight, IntoElement, Window};

pub struct SessionItemView {
    pub id: String,
    pub title: String,
    pub is_active: bool,
}

pub struct Sidebar {
    pub sessions: Vec<SessionItemView>,
    pub mcp_tools: Vec<String>,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            sessions: vec![
                SessionItemView {
                    id: "1".into(),
                    title: "🚀 Project Scaffolding".into(),
                    is_active: true,
                },
                SessionItemView {
                    id: "2".into(),
                    title: "🔧 Fix WebSocket Reconnect".into(),
                    is_active: false,
                },
                SessionItemView {
                    id: "3".into(),
                    title: "🎨 Markdown Syntax Theme".into(),
                    is_active: false,
                },
            ],
            mcp_tools: vec![
                "filesystem".into(),
                "terminal".into(),
                "git".into(),
                "browser".into(),
            ],
        }
    }
}

impl Render for Sidebar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_64()
            .h_full()
            .bg(rgb(0x121215))
            .border_r_1()
            .border_color(rgb(0x27272a))
            .flex()
            .flex_col()
            .p_3()
            .gap_4()
            // New Chat Button
            .child(
                div()
                    .w_full()
                    .py_1p5()
                    .px_3()
                    .rounded_md()
                    .bg(rgb(0x2563eb)) // blue-600
                    .hover(|s| s.bg(rgb(0x1d4ed8)))
                    .text_center()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xffffff))
                    .cursor_pointer()
                    .child("+ New Session"),
            )
            // Sessions section
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x71717a))
                            .child("RECENT SESSIONS"),
                    )
                    .children(self.sessions.iter().map(|sess| {
                        let bg = if sess.is_active {
                            rgb(0x27272a)
                        } else {
                            rgb(0x121215)
                        };
                        div()
                            .px_2p5()
                            .py_1p5()
                            .rounded_md()
                            .bg(bg)
                            .hover(|s| s.bg(rgb(0x27272a)))
                            .text_sm()
                            .text_color(rgb(0xe4e4e7))
                            .cursor_pointer()
                            .child(sess.title.clone())
                    })),
            )
            // Divider
            .child(div().h_px().bg(rgb(0x27272a)))
            // MCP Plugins section
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x71717a))
                            .child("ACTIVE MCP TOOLS"),
                    )
                    .children(self.mcp_tools.iter().map(|tool| {
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_2()
                            .py_1()
                            .text_xs()
                            .text_color(rgb(0xa1a1aa))
                            .child(div().size_1p5().rounded_full().bg(rgb(0x38bdf8)))
                            .child(tool.clone())
                    })),
            )
    }
}
