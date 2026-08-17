use gpui::{div, prelude::*, rgb, Context, FontWeight, IntoElement, Window};

#[derive(Clone, Copy, PartialEq)]
pub enum SettingsTab {
    Models,
    Mcp,
    General,
}

pub struct SettingsModal {
    pub is_open: bool,
    pub active_tab: SettingsTab,
    pub api_key: String,
    pub base_url: String,
    pub model_name: String,
}

impl Default for SettingsModal {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsModal {
    pub fn new() -> Self {
        Self {
            is_open: false,
            active_tab: SettingsTab::Models,
            api_key: "sk-••••••••••••••••".into(),
            base_url: "https://api.deepseek.com/v1".into(),
            model_name: "deepseek-chat".into(),
        }
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn set_tab(&mut self, tab: SettingsTab) {
        self.active_tab = tab;
    }
}

impl Render for SettingsModal {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        if !self.is_open {
            return div();
        }

        let is_models = self.active_tab == SettingsTab::Models;
        let is_mcp = self.active_tab == SettingsTab::Mcp;
        let is_general = self.active_tab == SettingsTab::General;

        div()
            .absolute()
            .inset_0()
            .bg(rgb(0x000000))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(gpui::px(640.0))
                    .h(gpui::px(480.0))
                    .rounded_2xl()
                    .bg(rgb(0x15171b))
                    .border_1()
                    .border_color(rgb(0x282c34))
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    // Modal Header
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_6()
                            .py_4()
                            .border_b_1()
                            .border_color(rgb(0x23262d))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(div().text_lg().child("⚙️"))
                                    .child(
                                        div()
                                            .font_weight(FontWeight::BOLD)
                                            .text_base()
                                            .text_color(rgb(0xffffff))
                                            .child("DeepSeek Harness Settings"),
                                    ),
                            )
                            .child(
                                div()
                                    .size_7()
                                    .rounded_md()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_sm()
                                    .text_color(rgb(0x979da6))
                                    .hover(|s| s.bg(rgb(0x23262d)).text_color(rgb(0xffffff)))
                                    .cursor_pointer()
                                    .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                                        cx.stop_propagation();
                                    })
                                    .child("✕"),
                            ),
                    )
                    // Tabs
                    .child(
                        div()
                            .flex()
                            .px_6()
                            .gap_4()
                            .border_b_1()
                            .border_color(rgb(0x23262d))
                            .bg(rgb(0x13151b))
                            .child(
                                div()
                                    .py_2p5()
                                    .border_b_2()
                                    .border_color(if is_models { rgb(0x4176e6) } else { rgb(0x00000000) })
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(if is_models { rgb(0x4176e6) } else { rgb(0x979da6) })
                                    .cursor_pointer()
                                    .child("🤖 Models & API"),
                            )
                            .child(
                                div()
                                    .py_2p5()
                                    .border_b_2()
                                    .border_color(if is_mcp { rgb(0x4176e6) } else { rgb(0x00000000) })
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(if is_mcp { rgb(0x4176e6) } else { rgb(0x979da6) })
                                    .cursor_pointer()
                                    .child("🔌 MCP Servers"),
                            )
                            .child(
                                div()
                                    .py_2p5()
                                    .border_b_2()
                                    .border_color(if is_general { rgb(0x4176e6) } else { rgb(0x00000000) })
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(if is_general { rgb(0x4176e6) } else { rgb(0x979da6) })
                                    .cursor_pointer()
                                    .child("🎨 General"),
                            ),
                    )
                    // Tab Body
                    .child(
                        div()
                            .flex_1()
                            .p_6()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .overflow_hidden()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1p5()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(0x979da6))
                                            .child("DEEPSEEK API KEY"),
                                    )
                                    .child(
                                        div()
                                            .p_2p5()
                                            .rounded_lg()
                                            .bg(rgb(0x191c22))
                                            .border_1()
                                            .border_color(rgb(0x282c34))
                                            .text_xs()
                                            .text_color(rgb(0xe4e4e7))
                                            .child(self.api_key.clone()),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1p5()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(0x979da6))
                                            .child("BASE URL (ENDPOINT)"),
                                    )
                                    .child(
                                        div()
                                            .p_2p5()
                                            .rounded_lg()
                                            .bg(rgb(0x191c22))
                                            .border_1()
                                            .border_color(rgb(0x282c34))
                                            .text_xs()
                                            .text_color(rgb(0xe4e4e7))
                                            .child(self.base_url.clone()),
                                    ),
                            ),
                    )
                    // Modal Footer
                    .child(
                        div()
                            .px_6()
                            .py_3p5()
                            .border_t_1()
                            .border_color(rgb(0x23262d))
                            .bg(rgb(0x13151b))
                            .flex()
                            .items_center()
                            .justify_end()
                            .gap_3()
                            .child(
                                div()
                                    .px_4()
                                    .py_1p5()
                                    .rounded_lg()
                                    .bg(rgb(0x4176e6))
                                    .hover(|s| s.bg(rgb(0x4d93f8)))
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xffffff))
                                    .cursor_pointer()
                                    .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                                        cx.stop_propagation();
                                    })
                                    .child("Save & Apply"),
                            ),
                    ),
            )
    }
}
