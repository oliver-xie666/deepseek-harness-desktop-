use crate::chat_view::ChatView;
use crate::diff_panel::DiffPanel;
use crate::sidebar::Sidebar;
use crate::title_bar::TitleBar;
use gpui::{div, prelude::*, rgb, AppContext, Context, Entity, IntoElement, Window};

pub struct WorkspaceView {
    pub title_bar: Entity<TitleBar>,
    pub sidebar: Entity<Sidebar>,
    pub chat_view: Entity<ChatView>,
    pub diff_panel: Entity<DiffPanel>,
}

impl WorkspaceView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let title_bar = cx.new(|_| TitleBar::new("deepseek-harness-desktop", "DeepSeek-V3", true));
        let sidebar = cx.new(|_| Sidebar::new());
        let chat_view = cx.new(|_| ChatView::new());
        let diff_panel = cx.new(|_| DiffPanel::new());

        Self {
            title_bar,
            sidebar,
            chat_view,
            diff_panel,
        }
    }
}

impl Render for WorkspaceView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0x09090b))
            .flex()
            .flex_col()
            // Top Title Bar
            .child(self.title_bar.clone())
            // Main Three-Pane Body
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
    }
}
