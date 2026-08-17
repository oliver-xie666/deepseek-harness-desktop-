//! Official DeepSeek Harness vector glyphs, rendered through GPUI's `svg()`
//! element. Each glyph is an alpha mask tinted by `color` (the `currentColor`
//! semantics of the upstream `dsh-client-ui-primitives` icon set), so a single
//! color argument reproduces the exact same look as the web UI.
//!
//! Assets resolve at runtime: prefer a sibling `assets/` directory next to the
//! executable (packaged layout), falling back to the source tree during
//! `cargo run`. Call [`init_assets`] once at startup to pick the packaged dir.

use gpui::{prelude::*, px, svg, Hsla, IntoElement};
use std::path::PathBuf;
use std::sync::OnceLock;

static ASSET_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Set the packaged assets directory. Call once at startup when an
/// executable-relative `assets/` directory exists; otherwise the source-tree
/// fallback is used automatically.
pub fn init_assets(dir: PathBuf) {
    let _ = ASSET_DIR.set(dir);
}

/// Resolve an asset filename to a loadable path.
fn asset(name: &str) -> String {
    if let Some(dir) = ASSET_DIR.get() {
        dir.join(name).to_string_lossy().into_owned()
    } else {
        format!("{}/assets/{}", env!("CARGO_MANIFEST_DIR"), name)
    }
}

/// The DeepSeek whale/fish mark (native 23.16×17.04, rendered width×height).
pub fn fish(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(asset("fish.svg"))
        .text_color(color)
        .w(px(size))
        .h(px(size * 17.04 / 23.16))
}

/// Closed-folder glyph (workspace placeholder / picker rows).
pub fn folder_close(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(asset("folder_close.svg"))
        .text_color(color)
        .w(px(size))
        .h(px(size))
}

/// Open-folder glyph (selected workspace chip), duotone inner fill included.
pub fn folder_open(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(asset("folder_open.svg"))
        .text_color(color)
        .w(px(size))
        .h(px(size))
}

/// Downward chevron for select chips.
pub fn chevron_down(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(asset("chevron_down.svg"))
        .text_color(color)
        .w(px(size))
        .h(px(size))
}

/// Plus glyph ("add workspace" footer entry).
pub fn plus(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(asset("plus.svg"))
        .text_color(color)
        .w(px(size))
        .h(px(size))
}

/// Agent-preset glyph (the new-session mode selector).
pub fn agent_preset(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(asset("agent_preset.svg"))
        .text_color(color)
        .w(px(size))
        .h(px(size))
}

/// New-session chat glyph.
pub fn new_chat(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(asset("new_chat.svg"))
        .text_color(color)
        .w(px(size))
        .h(px(size))
}

/// Left-panel glyph (sidebar collapse/expand toggle).
pub fn panel_left(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(asset("panel_left.svg"))
        .text_color(color)
        .w(px(size))
        .h(px(size))
}

/// Settings gear glyph.
pub fn settings(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(asset("settings.svg"))
        .text_color(color)
        .w(px(size))
        .h(px(size))
}

/// Check glyph (trailing selection marker in picker rows).
pub fn check(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(asset("check.svg"))
        .text_color(color)
        .w(px(size))
        .h(px(size))
}

/// The soft blue hero backdrop ellipse (native 1051×468).
pub fn glow(width: f32, height: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(asset("glow.svg"))
        .text_color(color)
        .w(px(width))
        .h(px(height))
}

/// Database glyph (models settings nav row).
pub fn data(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(asset("data.svg"))
        .text_color(color)
        .w(px(size))
        .h(px(size))
}

/// Close glyph (settings panel close button).
pub fn close(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(asset("close.svg"))
        .text_color(color)
        .w(px(size))
        .h(px(size))
}
