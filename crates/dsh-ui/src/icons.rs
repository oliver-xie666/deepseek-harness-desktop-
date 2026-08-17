//! Official DeepSeek Harness vector glyphs, rendered through GPUI's `svg()`
//! element. Each glyph is an alpha mask tinted by `color` (the `currentColor`
//! semantics of the upstream `dsh-client-ui-primitives` icon set), so a single
//! color argument reproduces the exact same look as the web UI.

use gpui::{prelude::*, px, svg, Hsla, IntoElement};

const FISH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fish.svg");
const FOLDER_CLOSE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/folder_close.svg");
const FOLDER_OPEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/folder_open.svg");
const CHEVRON_DOWN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/chevron_down.svg");
const PLUS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/plus.svg");
const AGENT_PRESET: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent_preset.svg");
const NEW_CHAT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/new_chat.svg");
const PANEL_LEFT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/panel_left.svg");
const SETTINGS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/settings.svg");
const CHECK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/check.svg");
const GLOW: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/glow.svg");
const DATA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/data.svg");
const CLOSE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/close.svg");

/// The DeepSeek whale/fish mark (native 23.16×17.04, rendered width×height).
pub fn fish(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(FISH)
        .text_color(color)
        .w(px(size))
        .h(px(size * 17.04 / 23.16))
}

/// Closed-folder glyph (workspace placeholder / picker rows).
pub fn folder_close(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(FOLDER_CLOSE)
        .text_color(color)
        .w(px(size))
        .h(px(size))
}

/// Open-folder glyph (selected workspace chip), duotone inner fill included.
pub fn folder_open(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(FOLDER_OPEN)
        .text_color(color)
        .w(px(size))
        .h(px(size))
}

/// Downward chevron for select chips.
pub fn chevron_down(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(CHEVRON_DOWN)
        .text_color(color)
        .w(px(size))
        .h(px(size))
}

/// Plus glyph ("add workspace" footer entry).
pub fn plus(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(PLUS)
        .text_color(color)
        .w(px(size))
        .h(px(size))
}

/// Agent-preset glyph (the new-session mode selector).
pub fn agent_preset(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(AGENT_PRESET)
        .text_color(color)
        .w(px(size))
        .h(px(size))
}

/// New-session chat glyph.
pub fn new_chat(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(NEW_CHAT)
        .text_color(color)
        .w(px(size))
        .h(px(size))
}

/// Left-panel glyph (sidebar collapse/expand toggle).
pub fn panel_left(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(PANEL_LEFT)
        .text_color(color)
        .w(px(size))
        .h(px(size))
}

/// Settings gear glyph.
pub fn settings(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(SETTINGS)
        .text_color(color)
        .w(px(size))
        .h(px(size))
}

/// Check glyph (trailing selection marker in picker rows).
pub fn check(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(CHECK)
        .text_color(color)
        .w(px(size))
        .h(px(size))
}

/// The soft blue hero backdrop ellipse (native 1051×468).
pub fn glow(width: f32, height: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(GLOW)
        .text_color(color)
        .w(px(width))
        .h(px(height))
}

/// Database glyph (models settings nav row).
pub fn data(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(DATA)
        .text_color(color)
        .w(px(size))
        .h(px(size))
}

/// Close glyph (settings panel close button).
pub fn close(size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .external_path(CLOSE)
        .text_color(color)
        .w(px(size))
        .h(px(size))
}
