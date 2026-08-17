mod chat_view;
mod details_drawer;
mod settings_modal;
mod sidebar;
mod text_input;
mod title_bar;
pub mod workspace;

use crate::workspace::WorkspaceView;
use clap::Parser;
use dsh_common::init_logging;
use dsh_core::AppState;
use dsh_daemon::{DaemonConfig, DaemonManager};
use gpui::{px, size, AppContext, Bounds, TitlebarOptions, WindowBounds, WindowOptions};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

#[derive(Parser, Debug)]
#[command(
    name = "dsh-desktop",
    version = "0.1.0",
    about = "DeepSeek Harness Native Desktop Workspace"
)]
struct CliArgs {
    /// Workspace root directory path
    #[arg(default_value = ".")]
    workspace: PathBuf,

    /// Explicit port for DeepSeek Harness daemon
    #[arg(short, long)]
    port: Option<u16>,

    /// Run with embedded mock daemon
    #[arg(long, default_value_t = true)]
    mock: bool,

    /// Default model name
    #[arg(short, long)]
    model: Option<String>,
}

fn main() {
    let args = CliArgs::parse();
    init_logging();

    info!(
        "Starting DeepSeek Harness 100% Pure Rust + GPUI Desktop (workspace: {}, mock: {})...",
        args.workspace.display(),
        args.mock
    );

    // Dedicated background Tokio Multi-Thread Runtime for Daemon & WebSocket
    let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to initialize background Tokio runtime");

    let _tokio_guard = tokio_runtime.enter();

    let port = args
        .port
        .or_else(|| DaemonManager::find_available_port(3080, 3180).ok())
        .unwrap_or(3080);

    let daemon_config = DaemonConfig {
        port,
        use_embedded_mock: args.mock,
        ..Default::default()
    };

    let daemon_manager = Arc::new(DaemonManager::new(daemon_config.clone()));
    let daemon_clone = daemon_manager.clone();

    // Start background daemon process
    tokio_runtime.spawn(async move {
        if let Err(e) = daemon_clone.start().await {
            tracing::warn!("Failed to start deepseek-harness daemon: {}", e);
        }
    });

    let (app_state, _rx) = AppState::new(daemon_config);

    // Launch GPUI 120 FPS DirectX Engine
    gpui_platform::application().run(move |cx| {
        let state_entity = cx.new(|_| app_state.clone());

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Default::default(),
                size: size(px(1280.0), px(800.0)),
            })),
            titlebar: Some(TitlebarOptions {
                title: None,
                appears_transparent: true,
                traffic_light_position: None,
            }),
            focus: true,
            show: true,
            kind: gpui::WindowKind::Normal,
            is_movable: true,
            app_owns_titlebar_drag: false, // Critical for Windows native dragging
            display_id: None,
            window_background: gpui::WindowBackgroundAppearance::Opaque,
            ..Default::default()
        };

        cx.open_window(window_options, |_window, cx| {
            cx.new(|cx| WorkspaceView::new(state_entity, cx))
        })
        .expect("Failed to create GPUI window");
    });
}
