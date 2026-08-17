use gpui::{div, prelude::*, rgb, Context, FontWeight, IntoElement, Window};

#[derive(Clone, Copy, PartialEq)]
pub enum DetailsTab {
    Diff,
    Terminal,
}

pub struct DiffPanel {
    pub active_tab: DetailsTab,
    pub current_file: String,
    pub lines: Vec<DiffLineView>,
    pub terminal_logs: Vec<String>,
}

pub struct DiffLineView {
    pub line_type: DiffLineType,
    pub content: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum DiffLineType {
    Unchanged,
    Added,
    Removed,
}

impl Default for DiffPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl DiffPanel {
    pub fn new() -> Self {
        Self {
            active_tab: DetailsTab::Diff,
            current_file: "crates/dsh-daemon/src/lib.rs".to_string(),
            lines: vec![
                DiffLineView {
                    line_type: DiffLineType::Unchanged,
                    content: " pub struct DaemonManager {".into(),
                },
                DiffLineView {
                    line_type: DiffLineType::Removed,
                    content: "-    pub port: u16,".into(),
                },
                DiffLineView {
                    line_type: DiffLineType::Added,
                    content: "+    pub config: DaemonConfig,".into(),
                },
                DiffLineView {
                    line_type: DiffLineType::Unchanged,
                    content: "     pub child: Arc<Mutex<Option<Child>>>, ".into(),
                },
            ],
            terminal_logs: vec![
                "[dsh-daemon] Spawning node daemon on ws://127.0.0.1:3000...".into(),
                "[dsh-core] Handshake with deepseek-harness successful.".into(),
                "[dsh-ui] 120 FPS DirectX Engine online. Ready.".into(),
            ],
        }
    }

    pub fn set_tab(&mut self, tab: DetailsTab) {
        self.active_tab = tab;
    }
}

impl Render for DiffPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let is_diff = self.active_tab == DetailsTab::Diff;

        div()
            .w_96()
            .h_full()
            .bg(rgb(0x15171b))
            .border_l_1()
            .border_color(rgb(0x23262d))
            .flex()
            .flex_col()
            .overflow_hidden()
            // Top Tab Header
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(rgb(0x23262d))
                    .bg(rgb(0x13151b))
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                div()
                                    .px_2p5()
                                    .py_1()
                                    .rounded_md()
                                    .bg(if is_diff { rgb(0x1f2228) } else { rgb(0x00000000) })
                                    .text_xs()
                                    .font_weight(if is_diff { FontWeight::BOLD } else { FontWeight::NORMAL })
                                    .text_color(if is_diff { rgb(0x4176e6) } else { rgb(0x979da6) })
                                    .cursor_pointer()
                                    .child("📑 Diff Review"),
                            )
                            .child(
                                div()
                                    .px_2p5()
                                    .py_1()
                                    .rounded_md()
                                    .bg(if !is_diff { rgb(0x1f2228) } else { rgb(0x00000000) })
                                    .text_xs()
                                    .font_weight(if !is_diff { FontWeight::BOLD } else { FontWeight::NORMAL })
                                    .text_color(if !is_diff { rgb(0x4176e6) } else { rgb(0x979da6) })
                                    .cursor_pointer()
                                    .child("💻 Terminal"),
                            ),
                    )
                    // Diff Actions (when Diff is active)
                    .child(
                        div()
                            .flex()
                            .gap_1p5()
                            .child(
                                div()
                                    .px_2p5()
                                    .py_0p5()
                                    .rounded_md()
                                    .bg(rgb(0x15803d))
                                    .hover(|s| s.bg(rgb(0x16a34a)))
                                    .text_xs()
                                    .text_color(rgb(0xffffff))
                                    .cursor_pointer()
                                    .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                                        cx.stop_propagation();
                                    })
                                    .child("✓ Accept"),
                            )
                            .child(
                                div()
                                    .px_2p5()
                                    .py_0p5()
                                    .rounded_md()
                                    .bg(rgb(0x991b1b))
                                    .hover(|s| s.bg(rgb(0xb91c1c)))
                                    .text_xs()
                                    .text_color(rgb(0xffffff))
                                    .cursor_pointer()
                                    .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                                        cx.stop_propagation();
                                    })
                                    .child("✕ Reject"),
                            ),
                    ),
            )
            // Body Content (Diff or Terminal)
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .child(
                        // File target indicator
                        div()
                            .px_3()
                            .py_1p5()
                            .bg(rgb(0x191c22))
                            .text_xs()
                            .text_color(rgb(0x4176e6))
                            .child(format!("📄 {}", self.current_file)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .flex()
                            .flex_col()
                            .children(self.lines.iter().map(|line| {
                                let (bg, fg) = match line.line_type {
                                    DiffLineType::Added => (rgb(0x0a3324), rgb(0x4ade80)),
                                    DiffLineType::Removed => (rgb(0x4a151b), rgb(0xf87171)),
                                    DiffLineType::Unchanged => (rgb(0x15171b), rgb(0x979da6)),
                                };
                                div()
                                    .px_3()
                                    .py_1()
                                    .bg(bg)
                                    .text_xs()
                                    .text_color(fg)
                                    .child(line.content.clone())
                            })),
                    ),
            )
            // Terminal Log Box at Bottom
            .child(
                div()
                    .h(gpui::px(144.0))
                    .border_t_1()
                    .border_color(rgb(0x23262d))
                    .bg(rgb(0x0f1115))
                    .flex()
                    .flex_col()
                    .p_2p5()
                    .gap_1()
                    .overflow_hidden()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x61666b))
                            .child("TERMINAL OUTPUT"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .flex()
                            .flex_col()
                            .children(self.terminal_logs.iter().map(|log| {
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x22c55e))
                                    .child(log.clone())
                            })),
                    ),
            )
    }
}
