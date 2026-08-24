use crate::icons;
use crate::settings_modal::SettingsModal;
use crate::text_input::TextInput;
use dsh_core::{AppState, FileNode, WorkspaceScanner};
use gpui::{
    deferred, div, prelude::*, px, rgb, Context, Entity, FontWeight, IntoElement, MouseButton,
    Subscription, Window,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

pub struct SessionItemView {
    pub id: String,
    pub title: String,
    pub is_active: bool,
}

/// Sidebar for workspace navigation and session actions.
pub struct Sidebar {
    pub file_tree: Option<FileNode>,
    pub sessions: Vec<SessionItemView>,
    pub active_workspace: String,
    pub collapsed: bool,
    search_open: bool,
    view_options_open: bool,
    workspace_menu_open: bool,
    sort_by_name: bool,
    session_menu: Option<String>,
    renaming_session: Option<String>,
    search_input: Entity<TextInput>,
    rename_input: Entity<TextInput>,
    state: Entity<Arc<AppState>>,
    settings_modal: Entity<SettingsModal>,
    _search_subscription: Subscription,
}

impl Sidebar {
    pub fn new(
        state: Entity<Arc<AppState>>,
        settings_modal: Entity<SettingsModal>,
        cx: &mut Context<Self>,
    ) -> Self {
        let initial_workspace = state.read(cx).workspace_path.blocking_read().clone();
        let tree = WorkspaceScanner::scan_dir(&initial_workspace, 2).ok();
        let initial_workspace_label = initial_workspace
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_else(|| initial_workspace.to_str().unwrap_or("工作区"))
            .to_string();
        let search_input = cx.new(|cx| TextInput::new("搜索会话…", cx));
        let rename_input = cx.new(|cx| TextInput::new("输入会话名称", cx));

        let view = Self {
            file_tree: tree,
            sessions: Vec::new(),
            active_workspace: initial_workspace_label,
            collapsed: false,
            search_open: false,
            view_options_open: false,
            workspace_menu_open: false,
            sort_by_name: false,
            session_menu: None,
            renaming_session: None,
            search_input: search_input.clone(),
            rename_input,
            state,
            settings_modal,
            _search_subscription: cx.observe(&search_input, |_, _, cx| cx.notify()),
        };

        let app_state = view.state.read(cx).clone();
        cx.spawn(async move |this, cx| {
            let mut last_snapshot = String::new();
            loop {
                tokio::time::sleep(Duration::from_millis(100)).await;
                let active_id = app_state.active_session_id.read().await.clone();
                let workspace = app_state.workspace_path.read().await.clone();
                let sessions = app_state.session_snapshot().await;
                let snapshot = format!(
                    "{}:{:?}:{}",
                    workspace.display(),
                    active_id,
                    sessions
                        .iter()
                        .map(|session| format!(
                            "{}:{}:{}",
                            session.id, session.title, session.updated_at
                        ))
                        .collect::<Vec<_>>()
                        .join("|")
                );
                if snapshot == last_snapshot {
                    continue;
                }
                last_snapshot = snapshot;
                let tree = WorkspaceScanner::scan_dir(&workspace, 2).ok();
                let workspace_label = workspace
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_else(|| workspace.to_str().unwrap_or("工作区"))
                    .to_string();
                if this
                    .update(cx, |sidebar, cx| {
                        sidebar.file_tree = tree.clone();
                        sidebar.active_workspace = workspace_label.clone();
                        sidebar.sessions = sessions
                            .iter()
                            .map(|session| SessionItemView {
                                id: session.id.clone(),
                                title: session.title.clone(),
                                is_active: active_id.as_deref() == Some(session.id.as_str()),
                            })
                            .collect();
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();

        view
    }

    pub fn select_session(&mut self, id: &str, cx: &mut Context<Self>) {
        let state = self.state.read(cx).clone();
        let id = id.to_string();
        tokio::spawn(async move {
            state.select_session(&id).await;
        });
        self.session_menu = None;
        cx.notify();
    }

    pub fn add_new_session(&mut self, cx: &mut Context<Self>) {
        let state = self.state.read(cx).clone();
        tokio::spawn(async move {
            let workspace = state.workspace_path.read().await.display().to_string();
            state.create_session("新会话", &workspace).await;
        });
        self.session_menu = None;
        cx.notify();
    }

    fn toggle_collapse(&mut self, cx: &mut Context<Self>) {
        self.collapsed = !self.collapsed;
        self.search_open = false;
        self.view_options_open = false;
        self.workspace_menu_open = false;
        self.session_menu = None;
        cx.notify();
    }

    fn toggle_search(&mut self, cx: &mut Context<Self>) {
        self.search_open = !self.search_open;
        self.view_options_open = false;
        self.workspace_menu_open = false;
        self.session_menu = None;
        if !self.search_open {
            self.search_input.update(cx, |input, cx| input.clear(cx));
        }
        cx.notify();
    }

    fn toggle_view_options(&mut self, cx: &mut Context<Self>) {
        self.view_options_open = !self.view_options_open;
        self.search_open = false;
        self.workspace_menu_open = false;
        self.session_menu = None;
        cx.notify();
    }

    fn toggle_workspace_menu(&mut self, cx: &mut Context<Self>) {
        self.workspace_menu_open = !self.workspace_menu_open;
        self.search_open = false;
        self.view_options_open = false;
        self.session_menu = None;
        cx.notify();
    }

    fn set_workspace(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let state = self.state.read(cx).clone();
        let label = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_else(|| path.to_str().unwrap_or("工作区"))
            .to_string();
        self.file_tree = WorkspaceScanner::scan_dir(&path, 2).ok();
        self.active_workspace = label;
        self.workspace_menu_open = false;
        tokio::spawn(async move {
            state.set_workspace_path(path).await;
        });
        cx.notify();
    }

    fn toggle_sort(&mut self, cx: &mut Context<Self>) {
        self.sort_by_name = !self.sort_by_name;
        self.view_options_open = false;
        cx.notify();
    }

    fn toggle_session_menu(&mut self, id: &str, cx: &mut Context<Self>) {
        if self.session_menu.as_deref() == Some(id) {
            self.session_menu = None;
        } else {
            self.session_menu = Some(id.to_string());
        }
        self.view_options_open = false;
        self.workspace_menu_open = false;
        cx.notify();
    }

    fn begin_rename(&mut self, id: &str, cx: &mut Context<Self>) {
        let Some(title) = self
            .sessions
            .iter()
            .find(|session| session.id == id)
            .map(|session| session.title.clone())
        else {
            return;
        };
        self.rename_input
            .update(cx, |input, cx| input.set_text(&title, cx));
        self.renaming_session = Some(id.to_string());
        self.session_menu = None;
        cx.notify();
    }

    fn commit_rename(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.renaming_session.take() else {
            return;
        };
        let title = self.rename_input.read(cx).text().trim().to_string();
        let state = self.state.read(cx).clone();
        tokio::spawn(async move {
            let _ = state.rename_session(&id, &title).await;
        });
        self.rename_input.update(cx, |input, cx| input.clear(cx));
        cx.notify();
    }

    fn duplicate_session(&mut self, id: &str, cx: &mut Context<Self>) {
        let state = self.state.read(cx).clone();
        self.session_menu = None;
        let id = id.to_string();
        tokio::spawn(async move {
            let _ = state.duplicate_session(&id).await;
        });
        cx.notify();
    }

    fn delete_session(&mut self, id: &str, cx: &mut Context<Self>) {
        let state = self.state.read(cx).clone();
        self.session_menu = None;
        let id = id.to_string();
        tokio::spawn(async move {
            let _ = state.delete_session(&id).await;
        });
        cx.notify();
    }

    fn open_settings(&mut self, cx: &mut Context<Self>) {
        self.settings_modal.update(cx, |modal, cx| modal.toggle(cx));
    }
}

impl Render for Sidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let handle_new_chat = cx.listener(|this, _, _, cx| this.add_new_session(cx));
        let handle_toggle = cx.listener(|this, _, _, cx| this.toggle_collapse(cx));
        let handle_settings = cx.listener(|this, _, _, cx| this.open_settings(cx));
        let handle_search = cx.listener(|this, _, _, cx| this.toggle_search(cx));
        let handle_view = cx.listener(|this, _, _, cx| this.toggle_view_options(cx));
        let handle_workspace = cx.listener(|this, _, _, cx| this.toggle_workspace_menu(cx));
        let handle_sort = cx.listener(|this, _, _, cx| this.toggle_sort(cx));
        let has_sessions = !self.sessions.is_empty();
        let collapsed = self.collapsed;
        let query = self.search_input.read(cx).text().trim().to_lowercase();
        let mut visible_sessions = self
            .sessions
            .iter()
            .filter(|session| query.is_empty() || session.title.to_lowercase().contains(&query))
            .collect::<Vec<_>>();
        if self.sort_by_name {
            visible_sessions.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        }

        div()
            .when(collapsed, |this| this.w(px(56.0)))
            .when(!collapsed, |this| this.w(px(280.0)))
            .h_full()
            .bg(rgb(0xf9fafb))
            .border_r_1()
            .border_color(rgb(0xe5e7eb))
            .flex()
            .flex_col()
            .justify_between()
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .px_3()
                    .py_1p5()
                    .child(
                        div()
                            .h(px(60.0))
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .cursor_pointer()
                                    .child(icons::fish(18.0, rgb(0x0f1115)))
                                    .when(!collapsed, |this| {
                                        this.children(vec![
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(rgb(0x0f1115))
                                                .child("deepseek")
                                                .into_any_element(),
                                            div()
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded_sm()
                                                .bg(rgb(0x0f1115))
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(rgb(0xffffff))
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
                                    .hover(|s| s.bg(rgb(0xf1f3f5)))
                                    .cursor_pointer()
                                    .on_mouse_down(MouseButton::Left, handle_toggle)
                                    .child(icons::panel_left(16.0, rgb(0x61666b))),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_2()
                            .h(px(38.0))
                            .rounded_xl()
                            .bg(rgb(0xffffff))
                            .border_1()
                            .border_color(rgb(0xe1e5eb))
                            .hover(|s| s.bg(rgb(0xf1f3f5)))
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, handle_new_chat)
                            .when(collapsed, |this| this.justify_center().px_0())
                            .child(icons::new_chat(16.0, rgb(0x0f1115)))
                            .when(!collapsed, |this| {
                                this.child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgb(0x0f1115))
                                        .child("新会话"),
                                )
                            }),
                    )
                    .when(!collapsed, |this| {
                        this.child(
                            div()
                                .relative()
                                .flex()
                                .flex_col()
                                .gap_2()
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
                                                .text_color(rgb(0x61666b))
                                                .child("工作区"),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_1()
                                                .child(sidebar_icon("⌕", handle_search))
                                                .child(sidebar_icon("☷", handle_view))
                                                .child(sidebar_icon("+", handle_workspace)),
                                        ),
                                )
                                .when(self.search_open, |this| {
                                    this.child(
                                        div()
                                            .h(px(30.0))
                                            .px_2()
                                            .rounded(px(7.0))
                                            .bg(rgb(0xffffff))
                                            .border_1()
                                            .border_color(rgb(0xe1e5eb))
                                            .child(self.search_input.clone()),
                                    )
                                })
                                .when(self.view_options_open, |this| {
                                    this.child(deferred(
                                        div()
                                            .absolute()
                                            .top(px(25.0))
                                            .right(px(20.0))
                                            .w(px(190.0))
                                            .p_1()
                                            .rounded_lg()
                                            .bg(rgb(0xffffff))
                                            .border_1()
                                            .border_color(rgb(0xe1e5eb))
                                            .shadow_lg()
                                            .child(menu_item(
                                                if self.sort_by_name {
                                                    "按最近使用排序"
                                                } else {
                                                    "按名称排序"
                                                },
                                                handle_sort,
                                            )),
                                    ))
                                })
                                .when(self.workspace_menu_open, |this| {
                                    this.child(deferred(workspace_menu(cx)))
                                })
                                .child(if has_sessions {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .children(visible_sessions.into_iter().map(|sess| {
                                            let is_active = sess.is_active;
                                            let session_id = sess.id.clone();
                                            let menu_open = self.session_menu.as_deref()
                                                == Some(session_id.as_str());
                                            let renaming = self.renaming_session.as_deref()
                                                == Some(session_id.as_str());
                                            let handle_click = cx.listener({
                                                let session_id = session_id.clone();
                                                move |this, _, _, cx| {
                                                    this.select_session(&session_id, cx);
                                                }
                                            });
                                            let handle_more = cx.listener({
                                                let session_id = session_id.clone();
                                                move |this, _, _, cx| {
                                                    this.toggle_session_menu(&session_id, cx);
                                                }
                                            });
                                            let handle_rename = cx.listener({
                                                let session_id = session_id.clone();
                                                move |this, _, _, cx| {
                                                    this.begin_rename(&session_id, cx);
                                                }
                                            });
                                            let handle_duplicate = cx.listener({
                                                let session_id = session_id.clone();
                                                move |this, _, _, cx| {
                                                    this.duplicate_session(&session_id, cx);
                                                }
                                            });
                                            let handle_delete = cx.listener({
                                                let session_id = session_id.clone();
                                                move |this, _, _, cx| {
                                                    this.delete_session(&session_id, cx);
                                                }
                                            });
                                            let handle_commit = cx.listener(|this, _, _, cx| {
                                                this.commit_rename(cx);
                                            });
                                            let bg = if is_active {
                                                rgb(0xe9edf2)
                                            } else {
                                                rgb(0xf9fafb)
                                            };
                                            let fg = if is_active {
                                                rgb(0x0f1115)
                                            } else {
                                                rgb(0x3f454d)
                                            };
                                            let action_button = if renaming {
                                                div()
                                                    .size(px(24.0))
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .rounded(px(6.0))
                                                    .text_size(px(17.0))
                                                    .text_color(rgb(0x81858c))
                                                    .hover(|s| {
                                                        s.bg(rgb(0xe1e5eb))
                                                            .text_color(rgb(0x0f1115))
                                                    })
                                                    .cursor_pointer()
                                                    .on_mouse_down(MouseButton::Left, handle_commit)
                                                    .child("✓")
                                                    .into_any_element()
                                            } else {
                                                div()
                                                    .size(px(24.0))
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .rounded(px(6.0))
                                                    .text_size(px(17.0))
                                                    .text_color(rgb(0x81858c))
                                                    .hover(|s| {
                                                        s.bg(rgb(0xe1e5eb))
                                                            .text_color(rgb(0x0f1115))
                                                    })
                                                    .cursor_pointer()
                                                    .on_mouse_down(MouseButton::Left, handle_more)
                                                    .child("⋯")
                                                    .into_any_element()
                                            };

                                            div()
                                                .relative()
                                                .flex()
                                                .items_center()
                                                .gap_1()
                                                .px_2p5()
                                                .py_1p5()
                                                .rounded_xl()
                                                .bg(bg)
                                                .hover(|s| s.bg(rgb(0xf1f3f5)))
                                                .child(if renaming {
                                                    div()
                                                        .flex_1()
                                                        .min_w(px(0.0))
                                                        .h(px(28.0))
                                                        .px_1()
                                                        .bg(rgb(0xffffff))
                                                        .border_1()
                                                        .border_color(rgb(0x3964fe))
                                                        .child(self.rename_input.clone())
                                                        .into_any_element()
                                                } else {
                                                    div()
                                                        .flex_1()
                                                        .min_w(px(0.0))
                                                        .overflow_hidden()
                                                        .text_xs()
                                                        .text_color(fg)
                                                        .text_ellipsis()
                                                        .cursor_pointer()
                                                        .on_mouse_down(
                                                            MouseButton::Left,
                                                            handle_click,
                                                        )
                                                        .child(sess.title.clone())
                                                        .into_any_element()
                                                })
                                                .child(action_button)
                                                .when(menu_open, |this| {
                                                    this.child(deferred(
                                                        div()
                                                            .absolute()
                                                            .top(px(34.0))
                                                            .right(px(4.0))
                                                            .w(px(150.0))
                                                            .p_1()
                                                            .rounded_lg()
                                                            .bg(rgb(0xffffff))
                                                            .border_1()
                                                            .border_color(rgb(0xe1e5eb))
                                                            .shadow_lg()
                                                            .flex()
                                                            .flex_col()
                                                            .child(menu_item(
                                                                "重命名",
                                                                handle_rename,
                                                            ))
                                                            .child(menu_item(
                                                                "复制会话",
                                                                handle_duplicate,
                                                            ))
                                                            .child(menu_item(
                                                                "删除会话",
                                                                handle_delete,
                                                            )),
                                                    ))
                                                })
                                                .into_any_element()
                                        }))
                                        .into_any_element()
                                } else {
                                    div()
                                        .px_1()
                                        .h(px(38.0))
                                        .text_xs()
                                        .text_color(rgb(0x81858c))
                                        .child("暂无会话")
                                        .into_any_element()
                                })
                                .child(file_tree_panel(self.file_tree.as_ref(), cx)),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .h(px(38.0))
                    .m_3()
                    .rounded_xl()
                    .hover(|s| s.bg(rgb(0xf1f3f5)))
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, handle_settings)
                    .when(collapsed, |this| this.justify_center().px_0())
                    .child(icons::settings(16.0, rgb(0x61666b)))
                    .when(!collapsed, |this| {
                        this.child(div().text_xs().text_color(rgb(0x61666b)).child("设置"))
                    }),
            )
    }
}

fn sidebar_icon(
    glyph: &'static str,
    handler: impl Fn(&gpui::MouseDownEvent, &mut Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    div()
        .size(px(24.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(6.0))
        .text_size(px(16.0))
        .text_color(rgb(0x81858c))
        .hover(|s| s.bg(rgb(0xe1e5eb)).text_color(rgb(0x0f1115)))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, handler)
        .child(glyph)
}

fn menu_item(
    label: &'static str,
    handler: impl Fn(&gpui::MouseDownEvent, &mut Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    div()
        .h(px(30.0))
        .px_2()
        .flex()
        .items_center()
        .rounded(px(6.0))
        .hover(|s| s.bg(rgb(0xf1f3f5)))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, handler)
        .text_xs()
        .text_color(rgb(0x3f454d))
        .child(label)
}

fn file_tree_panel(tree: Option<&FileNode>, cx: &mut Context<Sidebar>) -> impl IntoElement {
    div()
        .mt_2()
        .pt_2()
        .border_t_1()
        .border_color(rgb(0xe5e7eb))
        .flex()
        .flex_col()
        .gap_0p5()
        .child(
            div()
                .px_1()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0x61666b))
                .child("Explorer"),
        )
        .child(if let Some(tree) = tree {
            file_tree_node(tree, 0, cx).into_any_element()
        } else {
            div()
                .px_1()
                .py_1()
                .text_xs()
                .text_color(rgb(0x81858c))
                .child("无法读取工作区")
                .into_any_element()
        })
}

fn file_tree_node(node: &FileNode, depth: usize, cx: &mut Context<Sidebar>) -> impl IntoElement {
    let path = node.path.clone();
    let handle_open = cx.listener(move |_this, _, _, cx| {
        cx.open_with_system(&path);
    });
    let child_rows = node
        .children
        .iter()
        .map(|child| file_tree_node(child, depth + 1, cx).into_any_element())
        .collect::<Vec<_>>();
    let indent = 8.0 + (depth as f32 * 12.0);
    div()
        .flex()
        .flex_col()
        .child(
            div()
                .h(px(24.0))
                .pl(px(indent))
                .flex()
                .items_center()
                .gap_1()
                .hover(|style| style.bg(rgb(0xf1f3f5)))
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, handle_open)
                .text_xs()
                .text_color(if node.is_dir {
                    rgb(0x3f454d)
                } else {
                    rgb(0x61666b)
                })
                .child(node.icon.clone())
                .child(node.name.clone()),
        )
        .children(child_rows)
}

fn workspace_menu(cx: &mut Context<Sidebar>) -> impl IntoElement {
    div()
        .absolute()
        .top(px(25.0))
        .right(px(0.0))
        .w(px(210.0))
        .p_1()
        .rounded_lg()
        .bg(rgb(0xffffff))
        .border_1()
        .border_color(rgb(0xe1e5eb))
        .shadow_lg()
        .flex()
        .flex_col()
        .child(menu_item(
            ".",
            cx.listener(|this, _, _, cx| this.set_workspace(PathBuf::from("."), cx)),
        ))
        .child(menu_item(
            "..",
            cx.listener(|this, _, _, cx| this.set_workspace(PathBuf::from(".."), cx)),
        ))
        .child(div().h(px(1.0)).my_1().bg(rgb(0xe5e7eb)))
        .child(menu_item(
            "当前工作区",
            cx.listener(|this, _, _, cx| this.set_workspace(PathBuf::from("."), cx)),
        ))
}
