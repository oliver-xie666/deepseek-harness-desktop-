use gpui::{div, prelude::*, rgb, Context, FontWeight, IntoElement, Window};

pub struct DiffPanel {
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
                "[dsh-ui] Ready for user prompt.".into(),
            ],
        }
    }
}

impl Render for DiffPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_96()
            .h_full()
            .bg(rgb(0x121215))
            .border_l_1()
            .border_color(rgb(0x27272a))
            .flex()
            .flex_col()
            .overflow_hidden()
            // Header
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(rgb(0x27272a))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x71717a))
                            .child("DIFF REVIEW"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_sm()
                                    .bg(rgb(0x15803d))
                                    .hover(|s| s.bg(rgb(0x16a34a)))
                                    .text_xs()
                                    .text_color(rgb(0xffffff))
                                    .cursor_pointer()
                                    .child("✓ Accept All"),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_sm()
                                    .bg(rgb(0x991b1b))
                                    .hover(|s| s.bg(rgb(0xb91c1c)))
                                    .text_xs()
                                    .text_color(rgb(0xffffff))
                                    .cursor_pointer()
                                    .child("✕ Reject"),
                            ),
                    ),
            )
            // File target bar
            .child(
                div()
                    .px_3()
                    .py_1p5()
                    .bg(rgb(0x18181b))
                    .text_xs()
                    .text_color(rgb(0x38bdf8))
                    .child(format!("📄 {}", self.current_file)),
            )
            // Diff Content Lines
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .children(self.lines.iter().map(|line| {
                        let (bg, fg) = match line.line_type {
                            DiffLineType::Added => (rgb(0x064e3b), rgb(0x4ade80)),
                            DiffLineType::Removed => (rgb(0x7f1d1d), rgb(0xf87171)),
                            DiffLineType::Unchanged => (rgb(0x121215), rgb(0xa1a1aa)),
                        };
                        div()
                            .px_3()
                            .py_0p5()
                            .bg(bg)
                            .text_xs()
                            .text_color(fg)
                            .child(line.content.clone())
                    })),
            )
            // Terminal Logs Drawer (Bottom)
            .child(
                div()
                    .h_40()
                    .border_t_1()
                    .border_color(rgb(0x27272a))
                    .bg(rgb(0x09090b))
                    .flex()
                    .flex_col()
                    .p_2()
                    .gap_1()
                    .overflow_hidden()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x71717a))
                            .child("TERMINAL LOGS"),
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
