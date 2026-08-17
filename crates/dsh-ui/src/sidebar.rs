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
    pub active_workspace: String,
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
            sessions: Vec::new(),
            active_workspace: "deepseek-harness-desktop".into(),
        }
    }

    pub fn select_session(&mut self, id: &str, cx: &mut Context<Self>) {
        for sess in &mut self.sessions {
            sess.is_active = sess.id == id;
        }
        cx.notify();
    }

    pub fn add_new_session(&mut self, cx: &mut Context<Self>) {
        for sess in &mut self.sessions {
            sess.is_active = false;
        }
        let new_id = (self.sessions.len() + 1).to_string();
        self.sessions.insert(
            0,
            SessionItemView {
                id: new_id,
                title: "新会话".into(),
                is_active: true,
            },
        );
        cx.notify();
    }
}

impl Render for Sidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let handle_new_chat = cx.listener(|this, _, _, cx| {
            this.add_new_session(cx);
        });

        let has_sessions = !self.sessions.is_empty();

        div()
            .w_64()
            .h_full()
            .bg(rgb(0x111215))
            .border_r_1()
            .border_color(rgb(0x1a1c22))
            .flex()
            .flex_col()
            .justify_between()
            .p_3()
            .overflow_hidden()
            // Top Section
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    // ⊕ 新会话 (Dark Rounded Pill #212328)
                    .child(
                        div()
                            .w_full()
                            .py_2()
                            .px_3()
                            .rounded_xl()
                            .bg(rgb(0x212328))
                            .hover(|s| s.bg(rgb(0x2a2d35)))
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0xe4e4e7))
                            .cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, handle_new_chat)
                            .child("⊕ 新会话"),
                    )
                    // 工作区 Header + 🔍 ⇋ ⎘+
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(0x979da6))
                                    .child("工作区"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x61666b))
                                            .hover(|s| s.text_color(rgb(0xffffff)))
                                            .cursor_pointer()
                                            .child("🔍"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x61666b))
                                            .hover(|s| s.text_color(rgb(0xffffff)))
                                            .cursor_pointer()
                                            .child("⇋"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x61666b))
                                            .hover(|s| s.text_color(rgb(0xffffff)))
                                            .cursor_pointer()
                                            .child("⎘+"),
                                    ),
                            ),
                    )
                    // Sessions List or 暂无会话
                    .child(
                        if has_sessions {
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .children(self.sessions.iter().map(|sess| {
                                    let is_act = sess.is_active;
                                    let bg = if is_act { rgb(0x212328) } else { rgb(0x111215) };
                                    let fg = if is_act { rgb(0xffffff) } else { rgb(0x979da6) };
                                    let sess_id = sess.id.clone();
                                    let handle_click = cx.listener(move |this, _, _, cx| {
                                        this.select_session(&sess_id, cx);
                                    });

                                    div()
                                        .flex()
                                        .items_center()
                                        .px_2p5()
                                        .py_1p5()
                                        .rounded_lg()
                                        .bg(bg)
                                        .hover(|s| s.bg(rgb(0x212328)))
                                        .cursor_pointer()
                                        .on_mouse_down(gpui::MouseButton::Left, handle_click)
                                        .text_xs()
                                        .text_color(fg)
                                        .child(sess.title.clone())
                                }))
                                .into_any_element()
                        } else {
                            div()
                                .px_1()
                                .py_2()
                                .text_xs()
                                .text_color(rgb(0x61666b))
                                .child("暂无会话")
                                .into_any_element()
                        }
                    ),
            )
            // Bottom Section: ⚙ 设置
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_2()
                    .rounded_lg()
                    .hover(|s| s.bg(rgb(0x1a1c22)))
                    .cursor_pointer()
                    .text_xs()
                    .text_color(rgb(0x979da6))
                    .child("⚙")
                    .child("设置"),
            )
    }
}
