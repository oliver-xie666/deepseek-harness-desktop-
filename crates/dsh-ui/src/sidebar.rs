use dsh_core::{FileNode, WorkspaceScanner};
use gpui::{div, prelude::*, rgb, Context, FontWeight, IntoElement, Window};
use std::path::Path;

pub struct SessionItemView {
    pub id: String,
    pub title: String,
    pub is_active: bool,
}

pub struct Sidebar {
    pub file_tree: Option<FileNode>,
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
        let tree = WorkspaceScanner::scan_dir(Path::new("."), 2).ok();

        Self {
            file_tree: tree,
            sessions: vec![
                SessionItemView {
                    id: "1".into(),
                    title: "🚀 DeepSeek Harness Desktop".into(),
                    is_active: true,
                },
                SessionItemView {
                    id: "2".into(),
                    title: "⚡ Refactor WebSocket Reconnect".into(),
                    is_active: false,
                },
                SessionItemView {
                    id: "3".into(),
                    title: "🎨 120 FPS GPUI Markdown Theme".into(),
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

    fn render_file_node(&self, node: &FileNode, depth: usize) -> impl IntoElement {
        let indent = depth * 12;
        let icon = node.icon.clone();
        let name = node.name.clone();
        let is_dir = node.is_dir;

        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .py_1()
                    .px_2()
                    .rounded_md()
                    .hover(|s| s.bg(rgb(0x1f2228)))
                    .cursor_pointer()
                    .text_xs()
                    .text_color(if is_dir { rgb(0xf4f4f5) } else { rgb(0x979da6) })
                    .child(div().w(gpui::px(indent as f32)))
                    .child(div().child(icon))
                    .child(div().child(name)),
            )
            .children(
                node.children
                    .iter()
                    .map(move |child| self.render_file_node(child, depth + 1)),
            )
    }
}

impl Render for Sidebar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let file_nodes: Vec<_> = self
            .file_tree
            .as_ref()
            .map(|tree| {
                tree.children
                    .iter()
                    .map(|child| self.render_file_node(child, 0))
                    .collect()
            })
            .unwrap_or_default();

        div()
            .w_64()
            .h_full()
            .bg(rgb(0x15171b))
            .border_r_1()
            .border_color(rgb(0x23262d))
            .flex()
            .flex_col()
            .p_3()
            .gap_3()
            .overflow_hidden()
            // New Session Action Button (Official DeepSeek Blue)
            .child(
                div()
                    .w_full()
                    .py_2()
                    .px_3()
                    .rounded_lg()
                    .bg(rgb(0x4176e6))
                    .hover(|s| s.bg(rgb(0x4d93f8)))
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xffffff))
                    .cursor_pointer()
                    .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .child("💬 + New Session"),
            )
            // Section 1: Workspaces & File Explorer
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_1p5()
                    .overflow_hidden()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0x61666b))
                                    .child("WORKSPACE FILES"),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .flex()
                            .flex_col()
                            .children(file_nodes),
                    ),
            )
            // Divider
            .child(div().h_px().bg(rgb(0x23262d)))
            // Section 2: Recent Sessions
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x61666b))
                            .child("RECENT SESSIONS"),
                    )
                    .children(self.sessions.iter().map(|sess| {
                        let is_act = sess.is_active;
                        let bg = if is_act { rgb(0x1f2228) } else { rgb(0x15171b) };
                        let fg = if is_act { rgb(0xffffff) } else { rgb(0x979da6) };

                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_2p5()
                            .py_1p5()
                            .rounded_md()
                            .bg(bg)
                            .hover(|s| s.bg(rgb(0x1f2228)))
                            .cursor_pointer()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .w_1()
                                            .h_3()
                                            .rounded_full()
                                            .bg(if is_act { rgb(0x4176e6) } else { rgb(0x00000000) }),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(if is_act { FontWeight::MEDIUM } else { FontWeight::NORMAL })
                                            .text_color(fg)
                                            .child(sess.title.clone()),
                                    ),
                            )
                    })),
            )
            // Divider
            .child(div().h_px().bg(rgb(0x23262d)))
            // Section 3: Active MCP Tools Foot
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x61666b))
                            .child("ACTIVE MCP PLUGINS"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap_1p5()
                            .children(self.mcp_tools.iter().map(|tool| {
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_md()
                                    .bg(rgb(0x1f2228))
                                    .text_xs()
                                    .text_color(rgb(0x979da6))
                                    .child(div().size_1p5().rounded_full().bg(rgb(0x4176e6)))
                                    .child(tool.clone())
                            })),
                    ),
            )
    }
}
