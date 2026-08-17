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
                    .py_0p5()
                    .px_2()
                    .rounded_sm()
                    .hover(|s| s.bg(rgb(0x27272a)))
                    .cursor_pointer()
                    .text_xs()
                    .text_color(if is_dir { rgb(0xf4f4f5) } else { rgb(0xa1a1aa) })
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
            .bg(rgb(0x121215))
            .border_r_1()
            .border_color(rgb(0x27272a))
            .flex()
            .flex_col()
            .p_3()
            .gap_3()
            .overflow_hidden()
            // New Chat Button
            .child(
                div()
                    .w_full()
                    .py_1p5()
                    .px_3()
                    .rounded_md()
                    .bg(rgb(0x2563eb))
                    .hover(|s| s.bg(rgb(0x1d4ed8)))
                    .text_center()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xffffff))
                    .cursor_pointer()
                    .child("+ New Session"),
            )
            // Section 1: File Explorer
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .overflow_hidden()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x71717a))
                            .child("WORKSPACE FILES"),
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
            .child(div().h_px().bg(rgb(0x27272a)))
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
                            .py_1()
                            .rounded_md()
                            .bg(bg)
                            .hover(|s| s.bg(rgb(0x27272a)))
                            .text_xs()
                            .text_color(rgb(0xe4e4e7))
                            .cursor_pointer()
                            .child(sess.title.clone())
                    })),
            )
            // Divider
            .child(div().h_px().bg(rgb(0x27272a)))
            // Section 3: MCP Plugins
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
                            .py_0p5()
                            .text_xs()
                            .text_color(rgb(0xa1a1aa))
                            .child(div().size_1p5().rounded_full().bg(rgb(0x38bdf8)))
                            .child(tool.clone())
                    })),
            )
    }
}
