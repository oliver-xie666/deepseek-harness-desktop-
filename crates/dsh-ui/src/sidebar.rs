use crate::icons;
use crate::settings_modal::SettingsModal;
use dsh_core::{FileNode, WorkspaceScanner};
use gpui::{div, prelude::*, px, rgb, Context, Entity, FontWeight, IntoElement, Window};
use std::path::Path;

pub struct SessionItemView {
    pub id: String,
    pub title: String,
    pub is_active: bool,
}

/// Left column matching the official `SidebarRoot`: brand wordmark + panel
/// toggle, a "新会话" action, the workspace/session browsing region, and a
/// footer settings trigger.
pub struct Sidebar {
    pub file_tree: Option<FileNode>,
    pub sessions: Vec<SessionItemView>,
    pub active_workspace: String,
    pub collapsed: bool,
    settings_modal: Entity<SettingsModal>,
}

impl Sidebar {
    pub fn new(settings_modal: Entity<SettingsModal>) -> Self {
        let tree = WorkspaceScanner::scan_dir(Path::new("."), 2).ok();

        Self {
            file_tree: tree,
            sessions: Vec::new(),
            active_workspace: "deepseek-harness-desktop".into(),
            collapsed: false,
            settings_modal,
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

    pub fn toggle_collapse(&mut self, cx: &mut Context<Self>) {
        self.collapsed = !self.collapsed;
        cx.notify();
    }

    fn open_settings(&mut self, cx: &mut Context<Self>) {
        self.settings_modal.update(cx, |modal, cx| modal.toggle(cx));
    }
}

impl Render for Sidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let handle_new_chat = cx.listener(|this, _, _, cx| {
            this.add_new_session(cx);
        });
        let handle_toggle = cx.listener(|this, _, _, cx| {
            this.toggle_collapse(cx);
        });
        let handle_settings = cx.listener(|this, _, _, cx| {
            this.open_settings(cx);
        });

        let has_sessions = !self.sessions.is_empty();
        let collapsed = self.collapsed;

        div()
            .when(collapsed, |this| this.w(px(56.0)))
            .when(!collapsed, |this| this.w_64())
            .h_full()
            .bg(rgb(0x0d0f12))
            .border_r_1()
            .border_color(rgb(0x1a1c22))
            .flex()
            .flex_col()
            .justify_between()
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_3()
                    // Logo row: brand wordmark + panel toggle
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .cursor_pointer()
                                    .child(icons::fish(18.0, rgb(0xffffff)))
                                    .when(!collapsed, |this| {
                                        this.children(vec![
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(rgb(0xffffff))
                                                .child("deepseek")
                                                .into_any_element(),
                                            div()
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded_sm()
                                                .bg(rgb(0xe4e4e7))
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(rgb(0x0d0f12))
                                                .child("HARNESS")
                                                .into_any_element(),
                                        ])
                                    }),
                            )
                            .child(
                                div()
                                    .size_6()
                                    .rounded_md()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_color(rgb(0x979da6))
                                    .hover(|s| s.bg(rgb(0x1a1c22)).text_color(rgb(0xffffff)))
                                    .cursor_pointer()
                                    .on_mouse_down(gpui::MouseButton::Left, handle_toggle)
                                    .child(icons::panel_left(16.0, rgb(0x979da6))),
                            ),
                    )
                    // New session action
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_2()
                            .py_2()
                            .rounded_lg()
                            .bg(rgb(0x212328))
                            .hover(|s| s.bg(rgb(0x2a2d35)))
                            .cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, handle_new_chat)
                            .when(collapsed, |this| this.justify_center().px_0())
                            .child(icons::new_chat(16.0, rgb(0xe4e4e7)))
                            .when(!collapsed, |this| {
                                this.child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgb(0xe4e4e7))
                                        .child("新会话"),
                                )
                            }),
                    )
                    // Workspace section header
                    .when(!collapsed, |this| {
                        this.child(
                            div().flex().items_center().justify_between().px_1().child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(0x979da6))
                                    .child("工作区"),
                            ),
                        )
                    })
                    // Sessions list or empty hint
                    .when(!collapsed, |this| {
                        this.child(if has_sessions {
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .children(self.sessions.iter().map(|sess| {
                                    let is_act = sess.is_active;
                                    let bg = if is_act { rgb(0x212328) } else { rgb(0x0d0f12) };
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
                        })
                    }),
            )
            // Footer: settings trigger
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_2()
                    .m_3()
                    .rounded_lg()
                    .hover(|s| s.bg(rgb(0x1a1c22)))
                    .cursor_pointer()
                    .on_mouse_down(gpui::MouseButton::Left, handle_settings)
                    .when(collapsed, |this| this.justify_center().px_0())
                    .child(icons::settings(16.0, rgb(0x979da6)))
                    .when(!collapsed, |this| {
                        this.child(div().text_xs().text_color(rgb(0x979da6)).child("设置"))
                    }),
            )
    }
}
