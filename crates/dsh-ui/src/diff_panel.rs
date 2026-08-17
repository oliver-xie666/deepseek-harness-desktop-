use gpui::{div, prelude::*, rgb, Context, FontWeight, IntoElement, Window};

pub struct DiffFileView {
    pub file_path: String,
    pub lines: Vec<DiffLine>,
}

pub enum DiffLine {
    Context(String),
    Add(String),
    Delete(String),
}

pub struct DiffPanel {
    pub diffs: Vec<DiffFileView>,
    pub terminal_logs: Vec<String>,
}

impl Default for DiffPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl DiffPanel {
    pub fn new() -> Self {
        Self {
            diffs: vec![DiffFileView {
                file_path: "crates/dsh-protocol/src/lib.rs".into(),
                lines: vec![
                    DiffLine::Context(" pub enum HarnessServerEvent {".into()),
                    DiffLine::Delete("-    Token(String),".into()),
                    DiffLine::Add("+    TokenChunk { text: String },".into()),
                    DiffLine::Context(" }".into()),
                ],
            }],
            terminal_logs: vec![
                "$ cargo test -p dsh-protocol".into(),
                "   Compiling dsh-protocol v0.1.0".into(),
                "   Running tests/unit_tests.rs".into(),
                "test tests::test_client_message_roundtrip ... ok".into(),
                "test tests::test_server_event_roundtrip ... ok".into(),
                "test result: ok. 2 passed; 0 failed".into(),
            ],
        }
    }
}

impl Render for DiffPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_80()
            .h_full()
            .bg(rgb(0x121215))
            .border_l_1()
            .border_color(rgb(0x27272a))
            .flex()
            .flex_col()
            .p_3()
            .gap_3()
            // Header
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x71717a))
                            .child("CHANGES & DIFFS"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_1()
                            .child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_md()
                                    .bg(rgb(0x15803d)) // green-700
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xffffff))
                                    .cursor_pointer()
                                    .child("Accept All"),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_md()
                                    .bg(rgb(0x991b1b)) // red-800
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xffffff))
                                    .cursor_pointer()
                                    .child("Reject"),
                            ),
                    ),
            )
            // Diff Content Cards
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .children(self.diffs.iter().map(|diff| {
                        div()
                            .p_2p5()
                            .rounded_md()
                            .bg(rgb(0x18181b))
                            .border_1()
                            .border_color(rgb(0x27272a))
                            .flex()
                            .flex_col()
                            .gap_1p5()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0x38bdf8))
                                    .child(diff.file_path.clone()),
                            )
                            .child(
                                div()
                                    .p_2()
                                    .rounded_md()
                                    .bg(rgb(0x09090b))
                                    .text_xs()
                                    .flex()
                                    .flex_col()
                                    .children(diff.lines.iter().map(|l| match l {
                                        DiffLine::Context(t) => {
                                            div().text_color(rgb(0x71717a)).child(t.clone())
                                        }
                                        DiffLine::Add(t) => {
                                            div().text_color(rgb(0x4ade80)).child(t.clone())
                                        }
                                        DiffLine::Delete(t) => {
                                            div().text_color(rgb(0xf43f5e)).child(t.clone())
                                        }
                                    })),
                            )
                    })),
            )
            // Live Terminal Log
            .child(
                div()
                    .h_48()
                    .p_2p5()
                    .rounded_md()
                    .bg(rgb(0x09090b))
                    .border_1()
                    .border_color(rgb(0x27272a))
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x71717a))
                            .child("TERMINAL OUTPUT"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .text_xs()
                            .text_color(rgb(0xa1a1aa))
                            .children(self.terminal_logs.iter().map(|log| div().child(log.clone()))),
                    ),
            )
    }
}
