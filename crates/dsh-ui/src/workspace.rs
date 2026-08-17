use crate::chat_view::ChatView;
use crate::diff_panel::DiffPanel;
use crate::settings_modal::SettingsModal;
use crate::sidebar::Sidebar;
use crate::title_bar::TitleBar;
use dsh_core::AppState;
use gpui::{div, prelude::*, rgb, AppContext, Context, Entity, IntoElement, Window};
use std::sync::Arc;

pub struct WorkspaceView {
    pub state: Entity<Arc<AppState>>,
    pub title_bar: Entity<TitleBar>,
    pub sidebar: Entity<Sidebar>,
    pub chat_view: Entity<ChatView>,
    pub diff_panel: Entity<DiffPanel>,
    pub settings_modal: Entity<SettingsModal>,
}

impl WorkspaceView {
    pub fn new(state: Entity<Arc<AppState>>, cx: &mut Context<Self>) -> Self {
        let title_bar = cx.new(|_| TitleBar::new("deepseek-harness-desktop", "DeepSeek-V3", true));
        let sidebar = cx.new(|_| Sidebar::new());
        let chat_view = cx.new(|cx| ChatView::new(state.clone(), cx));
        let diff_panel = cx.new(|_| DiffPanel::new());
        let settings_modal = cx.new(|_| SettingsModal::new());

        Self {
            state,
            title_bar,
            sidebar,
            chat_view,
            diff_panel,
            settings_modal,
        }
    }
}

impl Render for WorkspaceView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0x0f1115))
            .flex()
            .flex_col()
            .relative()
            // Top Title Bar
            .child(self.title_bar.clone())
            // Main Three-Pane Body (AppFrame)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .w_full()
                    .overflow_hidden()
                    .child(self.sidebar.clone())
                    .child(self.chat_view.clone())
                    .child(self.diff_panel.clone()),
            )
            // Floating Settings Modal
            .child(self.settings_modal.clone())
    }
}
