use crate::chat_view::ChatView;
use crate::details_drawer::DetailsDrawer;
use crate::settings_modal::SettingsModal;
use crate::sidebar::Sidebar;
use crate::title_bar::TitleBar;
use dsh_core::AppState;
use gpui::{div, prelude::*, rgb, Context, Entity, IntoElement, Window};
use std::sync::Arc;

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
        let title_bar = cx.new(|_| TitleBar::new("deepseek-harness-desktop"));
        let settings_modal = cx.new(|_| SettingsModal::new());
        let sidebar = cx.new(|_| Sidebar::new(state.clone(), settings_modal.clone()));
        let details_drawer = cx.new(|_| DetailsDrawer::new());
        let chat_view = cx.new(|cx| ChatView::new(state.clone(), details_drawer.clone(), cx));

        Self {
            state,
            title_bar,
            sidebar,
            chat_view,
            details_drawer,
            settings_modal,
        }
    }
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
