mod chat_view;
mod diff_panel;
mod sidebar;
mod title_bar;
mod workspace;

use dsh_common::init_logging;
use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions, px, size};
use gpui_platform::application;
use tracing::info;
use workspace::WorkspaceView;

fn main() {
    init_logging();
    info!("Starting DeepSeek Harness Desktop (Rust + GPUI)...");

    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1280.0), px(800.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("DeepSeek Harness".into()),
                    appears_transparent: true,
                    traffic_light_position: None,
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| WorkspaceView::new(cx)),
        )
        .expect("Failed to open main window");

        cx.activate(true);
    });
}
