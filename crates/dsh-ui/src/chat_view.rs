use dsh_core::AppState;
use dsh_markdown::{CodeHighlighter, InlineSpan, MarkdownBlock, StreamingMarkdownParser, TokenType};
use gpui::{div, prelude::*, rgb, Context, Entity, FontWeight, IntoElement, Window};
use std::sync::Arc;

pub struct ChatView {
    pub state: Entity<Arc<AppState>>,
    pub prompt_draft: String,
}

impl ChatView {
    pub fn new(state: Entity<Arc<AppState>>, _cx: &mut Context<Self>) -> Self {
        Self {
            state,
            prompt_draft: "请帮我用 Rust + GPUI 重构 DeepSeek Harness 桌面端".to_string(),
        }
    }

    pub fn send_prompt(&mut self, _cx: &mut Context<Self>) {
        let text = self.prompt_draft.clone();
        if text.trim().is_empty() {
            return;
        }

        let state_arc = self.state.read(_cx).clone();
        tokio::spawn(async move {
            let session_id = {
                let active_opt = state_arc.active_session_id.read().await.clone();
                match active_opt {
                    Some(id) => id,
                    None => state_arc.create_session("New Session", ".").await,
                }
            };
            let _ = state_arc.add_user_message(&session_id, &text).await;
        });
    }

    fn render_markdown_block(&self, block: MarkdownBlock) -> impl IntoElement {
        match block {
            MarkdownBlock::Heading { level, inlines } => {
                let text: String = inlines
                    .into_iter()
                    .map(|i| match i {
                        InlineSpan::Text(t)
                        | InlineSpan::Bold(t)
                        | InlineSpan::Italic(t)
                        | InlineSpan::Code(t) => t,
                        InlineSpan::Link { text, .. } => text,
                        InlineSpan::FilePath { path, .. } => path,
                    })
                    .collect();

                div()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0x38bdf8))
                    .py_1()
                    .child(format!("{} {}", "#".repeat(level as usize), text))
            }
            MarkdownBlock::Paragraph { inlines } => {
                div().py_1().flex().flex_wrap().gap_1().children(
                    inlines.into_iter().map(|span| match span {
                        InlineSpan::Bold(t) => div().font_weight(FontWeight::BOLD).text_color(rgb(0xffffff)).child(t),
                        InlineSpan::Italic(t) => div().text_color(rgb(0xd4d4d8)).child(t),
                        InlineSpan::Code(t) => div()
                            .px_1p5()
                            .py_0p5()
                            .rounded_md()
                            .bg(rgb(0x27272a))
                            .text_color(rgb(0xf43f5e))
                            .child(t),
                        InlineSpan::Text(t) => div().text_color(rgb(0xd4d4d8)).child(t),
                        InlineSpan::Link { text, .. } => div()
                            .text_color(rgb(0x60a5fa))
                            .underline()
                            .cursor_pointer()
                            .child(text),
                        InlineSpan::FilePath { path, .. } => div()
                            .px_1p5()
                            .py_0p5()
                            .rounded_md()
                            .bg(rgb(0x1e293b))
                            .text_color(rgb(0x38bdf8))
                            .cursor_pointer()
                            .child(path),
                    }),
                )
            }
            MarkdownBlock::CodeBlock { language, code } => {
                let spans = CodeHighlighter::highlight(&code, &language);

                div()
                    .my_2()
                    .rounded_md()
                    .bg(rgb(0x18181b))
                    .border_1()
                    .border_color(rgb(0x27272a))
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_3()
                            .py_1()
                            .bg(rgb(0x202024))
                            .text_xs()
                            .text_color(rgb(0xa1a1aa))
                            .child(language)
                            .child(
                                div()
                                    .cursor_pointer()
                                    .hover(|s| s.text_color(rgb(0xffffff)))
                                    .child("📋 Copy"),
                            ),
                    )
                    .child(
                        div().p_3().text_xs().flex().flex_col().children(
                            spans.into_iter().map(|span| {
                                let color = match span.token_type {
                                    TokenType::Keyword => rgb(0xf43f5e),
                                    TokenType::Function => rgb(0x38bdf8),
                                    TokenType::Type => rgb(0xfbbf24),
                                    TokenType::String => rgb(0x4ade80),
                                    TokenType::Comment => rgb(0x71717a),
                                    TokenType::Number => rgb(0xc084fc),
                                    _ => rgb(0xe4e4e7),
                                };
                                div().text_color(color).child(span.text)
                            }),
                        ),
                    )
            }
            MarkdownBlock::Alert { inlines, .. } => {
                let text: String = inlines
                    .into_iter()
                    .map(|i| match i {
                        InlineSpan::Text(t) => t,
                        _ => String::new(),
                    })
                    .collect();

                div()
                    .my_2()
                    .p_3()
                    .rounded_md()
                    .bg(rgb(0x0f172a))
                    .border_l_4()
                    .border_color(rgb(0x38bdf8))
                    .text_xs()
                    .text_color(rgb(0xe2e8f0))
                    .child(text)
            }
            _ => div(),
        }
    }
}

impl Render for ChatView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let prompt_preview = self.prompt_draft.clone();

        div()
            .flex_1()
            .h_full()
            .bg(rgb(0x09090b))
            .flex()
            .flex_col()
            .justify_between()
            // Messages Scroll Area
            .child(
                div()
                    .flex_1()
                    .p_4()
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .child(
                                div()
                                    .max_w_3_4()
                                    .p_3()
                                    .rounded_lg()
                                    .bg(rgb(0x2563eb))
                                    .text_sm()
                                    .text_color(rgb(0xffffff))
                                    .child(prompt_preview),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .max_w_full()
                            .p_4()
                            .rounded_lg()
                            .bg(rgb(0x141417))
                            .border_1()
                            .border_color(rgb(0x27272a))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .px_2p5()
                                    .py_1()
                                    .rounded_md()
                                    .bg(rgb(0x1e1e24))
                                    .border_1()
                                    .border_color(rgb(0x3f3f46))
                                    .text_xs()
                                    .text_color(rgb(0xa1a1aa))
                                    .child("🔧")
                                    .child("grep_search (100ms)")
                                    .child("✓"),
                            )
                            .children(
                                StreamingMarkdownParser::parse_markdown(
                                    "正在实时连接 DeepSeek Harness 守护进程...\n\n```rust\npub async fn run_agent() {\n    println!(\"120 FPS Realtime Agent Stream!\");\n}\n```\n\n> [!TIP]\n> WebSocket IPC 已成功建立双向数据通路！",
                                )
                                .into_iter()
                                .map(|b| self.render_markdown_block(b)),
                            ),
                    ),
            )
            // Bottom Prompt Input Bar
            .child(
                div()
                    .p_3()
                    .border_t_1()
                    .border_color(rgb(0x27272a))
                    .bg(rgb(0x121215))
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(0x18181b))
                            .border_1()
                            .border_color(rgb(0x27272a))
                            .text_sm()
                            .text_color(rgb(0xe4e4e7))
                            .child("输入指令，按 Enter 发送，或键入 @ 引用代码..."),
                    )
                    .child(
                        div()
                            .px_4()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(0x2563eb))
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xffffff))
                            .cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                                cx.stop_propagation();
                            })
                            .child("发送 ⏎"),
                    ),
            )
    }
}
