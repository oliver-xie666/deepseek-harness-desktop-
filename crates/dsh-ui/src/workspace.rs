use crate::chat_view::ChatView;
use crate::details_drawer::DetailsDrawer;
use crate::settings_modal::SettingsModal;
use crate::sidebar::Sidebar;
use crate::title_bar::TitleBar;
use dsh_core::AppState;
use gpui::{div, prelude::*, rgb, Context, Entity, IntoElement, Window};
use std::sync::Arc;
use std::time::Duration;

pub struct WorkspaceView {
    pub state: Entity<Arc<AppState>>,
    pub title_bar: Entity<TitleBar>,
    pub sidebar: Entity<Sidebar>,
    pub chat_view: Entity<ChatView>,
    pub details_drawer: Entity<DetailsDrawer>,
    pub settings_modal: Entity<SettingsModal>,
}

impl WorkspaceView {
    pub fn new(state: Entity<Arc<AppState>>, cx: &mut Context<Self>) -> Self {
        let app_state = state.read(cx).clone();
        let initial_workspace = app_state.workspace_path.blocking_read().clone();
        let initial_name = workspace_name(&initial_workspace);
        let title_bar = cx.new(|_| TitleBar::new(&initial_name));
        let settings_modal = cx.new(|cx| SettingsModal::new(state.clone(), cx));
        let sidebar = cx.new(|cx| Sidebar::new(state.clone(), settings_modal.clone(), cx));
        let details_drawer = cx.new(|_| DetailsDrawer::new());
        let chat_view = cx.new(|cx| ChatView::new(state.clone(), details_drawer.clone(), cx));

        let view = Self {
            state,
            title_bar,
            sidebar,
            chat_view,
            details_drawer,
            settings_modal,
        };

        let title_state = app_state.clone();
        let title_bar_entity = view.title_bar.clone();
        cx.spawn(async move |_this, cx| {
            let mut last_workspace = String::new();
            loop {
                tokio::time::sleep(Duration::from_millis(100)).await;
                let workspace = title_state.workspace_path.read().await.clone();
                let name = workspace_name(&workspace);
                if name == last_workspace {
                    continue;
                }
                last_workspace = name.clone();
                title_bar_entity
                    .update(cx, |title_bar, cx| title_bar.set_workspace_name(&name, cx));
            }
        })
        .detach();

        view
    }
}

fn workspace_name(path: &std::path::Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_else(|| path.to_str().unwrap_or("工作区"))
        .to_string()
}

impl Render for WorkspaceView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0xf9fafb))
            .flex()
            .flex_col()
            .relative()
            // Top Native Title Bar (Draggable)
            .child(self.title_bar.clone())
            // Main Body: Sidebar + Conversation + Details Drawer
            .child(
                div()
                    .flex_1()
                    .flex()
                    .w_full()
                    .overflow_hidden()
                    .child(self.sidebar.clone())
                    .child(self.chat_view.clone())
                    .child(self.details_drawer.clone()),
            )
            // Floating Settings Modal
            .child(self.settings_modal.clone())
    }
}
