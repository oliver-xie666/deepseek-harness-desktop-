use clap::Parser;
use dsh_common::init_logging;
use dsh_daemon::{DaemonConfig, DaemonManager};
use std::path::PathBuf;
use std::sync::Arc;
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use tracing::info;
use wry::WebViewBuilder;

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
    #[arg(long, default_value_t = false)]
    mock: bool,

    /// Default model name
    #[arg(short, long)]
    model: Option<String>,
}

fn main() {
    let args = CliArgs::parse();
    init_logging();

    info!(
        "Starting DeepSeek Harness Native Desktop (workspace: {}, mock: {})...",
        args.workspace.display(),
        args.mock
    );

    // Initialize dedicated background Tokio Multi-Thread Runtime
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

    let daemon_manager = Arc::new(DaemonManager::new(daemon_config));
    let daemon_clone = daemon_manager.clone();

    // Start background daemon process
    tokio_runtime.spawn(async move {
        if let Err(e) = daemon_clone.start().await {
            tracing::warn!("Failed to start deepseek-harness daemon: {}", e);
        }
    });

    // Wait for daemon readiness
    let target_url = daemon_manager.http_url();
    info!("Target DeepSeek Harness URL: {}", target_url);

    // Create native Windows Desktop window
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("DeepSeek Harness")
        .with_inner_size(LogicalSize::new(1280.0, 800.0))
        .with_min_inner_size(LogicalSize::new(900.0, 600.0))
        .with_resizable(true)
        .build(&event_loop)
        .expect("Failed to create native desktop window");

    let _webview = WebViewBuilder::new()
        .with_url(&target_url)
        .build(&window)
        .expect("Failed to initialize DirectX WebView hardware engine");

    let daemon_for_shutdown = daemon_manager.clone();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            let daemon_shutdown = daemon_for_shutdown.clone();
            tokio::spawn(async move {
                let _ = daemon_shutdown.stop().await;
            });
            *control_flow = ControlFlow::Exit;
        }
    });
}
