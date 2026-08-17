mod chat_view;
mod diff_panel;
mod settings_modal;
mod sidebar;
mod title_bar;
mod workspace;

use dsh_common::init_logging;
use dsh_core::AppState;
use dsh_daemon::{DaemonConfig, DaemonManager};
use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions, px, size};
use gpui_platform::application;
use tracing::info;
use workspace::WorkspaceView;

fn main() {
    init_logging();
    info!("Starting DeepSeek Harness Desktop (Rust + GPUI)...");

    let port = DaemonManager::find_available_port(3000, 3100).unwrap_or(3000);
    let daemon_config = DaemonConfig {
        port,
        use_embedded_mock: true,
        ..Default::default()
    };

    let (app_state, outbox_rx) = AppState::new(daemon_config);
    let app_state_clone = app_state.clone();

    // Start background runtime & WebSocket worker
    tokio::spawn(async move {
        if let Err(e) = app_state_clone.daemon_manager.start().await {
            tracing::warn!("Failed to start daemon manager: {}", e);
        }
    });

    let _ws_client = AppState::start_background_client(app_state.clone(), outbox_rx);

    application().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1280.0), px(800.0)), cx);
        let state_for_window = app_state.clone();

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
            move |_, cx| {
                let state_model = cx.new(|_| state_for_window.clone());
                cx.new(|cx| WorkspaceView::new(state_model, cx))
            },
        )
        .expect("Failed to open main window");

        cx.activate(true);
    });
}
