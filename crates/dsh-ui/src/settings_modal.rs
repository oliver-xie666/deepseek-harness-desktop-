use dsh_core::{AppConfig, McpRegistry, McpServerConfig};
use gpui::{div, prelude::*, rgb, Context, FontWeight, IntoElement, Window};

#[derive(Clone, Copy, PartialEq)]
pub enum SettingsTab {
    Model,
    Mcp,
    General,
}

pub struct SettingsModal {
    pub is_open: bool,
    pub active_tab: SettingsTab,
    pub api_key: String,
    pub base_url: String,
    pub model_name: String,
    pub mcp_servers: Vec<McpServerConfig>,
}

impl Default for SettingsModal {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsModal {
    pub fn new() -> Self {
        let default_config = AppConfig::default();
        let default_mcp = McpRegistry::get_default_presets();

        Self {
            is_open: false,
            active_tab: SettingsTab::Model,
            api_key: default_config.model.api_key,
            base_url: default_config.model.base_url,
            model_name: default_config.model.model_name,
            mcp_servers: default_mcp,
        }
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn set_tab(&mut self, tab: SettingsTab) {
        self.active_tab = tab;
    }

    fn render_model_tab(&self) -> impl IntoElement {
        let base_url = self.base_url.clone();

        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_xs().text_color(rgb(0xa1a1aa)).child("DeepSeek API Key"))
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(0x18181b))
                            .border_1()
                            .border_color(rgb(0x27272a))
                            .text_sm()
                            .text_color(rgb(0xf43f5e))
                            .child("sk-••••••••••••••••••••••••••••"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_xs().text_color(rgb(0xa1a1aa)).child("API Base URL"))
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(0x18181b))
                            .border_1()
                            .border_color(rgb(0x27272a))
                            .text_sm()
                            .text_color(rgb(0xe4e4e7))
                            .child(base_url),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_xs().text_color(rgb(0xa1a1aa)).child("Default Model"))
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                div()
                                    .px_3()
                                    .py_1p5()
                                    .rounded_md()
                                    .bg(rgb(0x2563eb))
                                    .text_xs()
                                    .text_color(rgb(0xffffff))
                                    .cursor_pointer()
                                    .child("deepseek-chat (V3)"),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1p5()
                                    .rounded_md()
                                    .bg(rgb(0x27272a))
                                    .text_xs()
                                    .text_color(rgb(0xa1a1aa))
                                    .hover(|s| s.bg(rgb(0x3f3f46)))
                                    .cursor_pointer()
                                    .child("deepseek-reasoner (R1)"),
                            ),
                    ),
            )
    }

    fn render_mcp_tab(&self) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .children(self.mcp_servers.iter().map(|server| {
                let name = server.name.clone();
                let desc = server.description.clone();

                div()
                    .p_3()
                    .rounded_md()
                    .bg(rgb(0x18181b))
                    .border_1()
                    .border_color(rgb(0x27272a))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .font_weight(FontWeight::BOLD)
                                    .text_sm()
                                    .text_color(rgb(0xf4f4f5))
                                    .child(name),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x71717a))
                                    .child(desc),
                            ),
                    )
                    .child(
                        div()
                            .px_2p5()
                            .py_1()
                            .rounded_md()
                            .bg(if server.enabled { rgb(0x15803d) } else { rgb(0x27272a) })
                            .text_xs()
                            .text_color(rgb(0xffffff))
                            .cursor_pointer()
                            .child(if server.enabled { "Enabled" } else { "Disabled" }),
                    )
            }))
    }

    fn render_general_tab(&self) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(div().text_sm().text_color(rgb(0xf4f4f5)).child("Color Theme"))
                    .child(div().text_xs().text_color(rgb(0xa1a1aa)).child("Dark (OLED Black)")),
            )
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(div().text_sm().text_color(rgb(0xf4f4f5)).child("Target Framerate"))
                    .child(div().text_xs().text_color(rgb(0x22c55e)).child("120 FPS (DirectX GPU)")),
            )
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(div().text_sm().text_color(rgb(0xf4f4f5)).child("Portable Node.js Runtime"))
                    .child(div().text_xs().text_color(rgb(0x38bdf8)).child("Self-Contained")),
            )
    }
}

impl Render for SettingsModal {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        if !self.is_open {
            return div();
        }

        let is_model = self.active_tab == SettingsTab::Model;
        let is_mcp = self.active_tab == SettingsTab::Mcp;
        let is_gen = self.active_tab == SettingsTab::General;

        div()
            .absolute()
            .inset_0()
            .bg(gpui::rgba(0x000000bb))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(gpui::px(600.0))
                    .h(gpui::px(440.0))
                    .rounded_xl()
                    .bg(rgb(0x121215))
                    .border_1()
                    .border_color(rgb(0x3f3f46))
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    // Modal Header
                    .child(
                        div()
                            .px_4()
                            .py_3()
                            .border_b_1()
                            .border_color(rgb(0x27272a))
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .font_weight(FontWeight::BOLD)
                                    .text_base()
                                    .text_color(rgb(0xffffff))
                                    .child("⚙️ Settings & Preferences"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0xa1a1aa))
                                    .hover(|s| s.text_color(rgb(0xffffff)))
                                    .cursor_pointer()
                                    .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                                        cx.stop_propagation();
                                    })
                                    .child("✕ Close"),
                            ),
                    )
                    // Tabs Header
                    .child(
                        div()
                            .flex()
                            .border_b_1()
                            .border_color(rgb(0x27272a))
                            .bg(rgb(0x0e0e11))
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(if is_model { rgb(0x38bdf8) } else { rgb(0x71717a) })
                                    .border_b_2()
                                    .border_color(if is_model { rgb(0x38bdf8) } else { rgb(0x00000000) })
                                    .cursor_pointer()
                                    .child("Model & API"),
                            )
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(if is_mcp { rgb(0x38bdf8) } else { rgb(0x71717a) })
                                    .border_b_2()
                                    .border_color(if is_mcp { rgb(0x38bdf8) } else { rgb(0x00000000) })
                                    .cursor_pointer()
                                    .child("MCP Tools"),
                            )
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(if is_gen { rgb(0x38bdf8) } else { rgb(0x71717a) })
                                    .border_b_2()
                                    .border_color(if is_gen { rgb(0x38bdf8) } else { rgb(0x00000000) })
                                    .cursor_pointer()
                                    .child("General"),
                            ),
                    )
                    // Tab Body
                    .child(
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .flex()
                            .flex_col()
                            .child(match self.active_tab {
                                SettingsTab::Model => self.render_model_tab().into_any_element(),
                                SettingsTab::Mcp => self.render_mcp_tab().into_any_element(),
                                SettingsTab::General => self.render_general_tab().into_any_element(),
                            }),
                    )
                    // Footer
                    .child(
                        div()
                            .px_4()
                            .py_2p5()
                            .border_t_1()
                            .border_color(rgb(0x27272a))
                            .bg(rgb(0x141417))
                            .flex()
                            .justify_end()
                            .gap_2()
                            .child(
                                div()
                                    .px_4()
                                    .py_1p5()
                                    .rounded_md()
                                    .bg(rgb(0x2563eb))
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xffffff))
                                    .cursor_pointer()
                                    .child("Save & Apply"),
                            ),
                    ),
            )
    }
}
