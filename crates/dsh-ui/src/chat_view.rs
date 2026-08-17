use dsh_core::AppState;
use dsh_markdown::{CodeHighlighter, InlineSpan, MarkdownBlock, StreamingMarkdownParser, TokenType};
use gpui::{div, prelude::*, rgb, Context, Entity, FontWeight, IntoElement, Window};
use std::sync::Arc;

pub struct ChatView {
    pub state: Entity<Arc<AppState>>,
    pub prompt_draft: String,
    pub active_preset_sent: bool,
}

impl ChatView {
    pub fn new(state: Entity<Arc<AppState>>, _cx: &mut Context<Self>) -> Self {
        Self {
            state,
            prompt_draft: String::new(),
            active_preset_sent: false,
        }
    }

    pub fn send_text(&mut self, text: String, cx: &mut Context<Self>) {
        if text.trim().is_empty() {
            return;
        }

        let state_arc = self.state.read(cx).clone();
        self.prompt_draft.clear();
        self.active_preset_sent = true;
        cx.notify();

        tokio::spawn(async move {
            let session_id = {
                let active_opt = state_arc.active_session_id.read().await.clone();
                match active_opt {
                    Some(id) => id,
                    None => state_arc.create_session("New Chat", ".").await,
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
                    .text_color(rgb(0x4176e6))
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
                            .bg(rgb(0x282c34))
                            .text_color(rgb(0xf43f5e))
                            .child(t),
                        InlineSpan::Text(t) => div().text_color(rgb(0xe4e4e7)).child(t),
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
                    .rounded_lg()
                    .bg(rgb(0x13151b))
                    .border_1()
                    .border_color(rgb(0x282c34))
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_3()
                            .py_1p5()
                            .bg(rgb(0x1c1f26))
                            .text_xs()
                            .text_color(rgb(0x979da6))
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
                                    TokenType::Function => rgb(0x4176e6),
                                    TokenType::Type => rgb(0xfbbf24),
                                    TokenType::String => rgb(0x4ade80),
                                    TokenType::Comment => rgb(0x61666b),
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
                    .rounded_lg()
                    .bg(rgb(0x151b28))
                    .border_l_4()
                    .border_color(rgb(0x4176e6))
                    .text_xs()
                    .text_color(rgb(0xe2e8f0))
                    .child(text)
            }
            _ => div(),
        }
    }

    fn render_empty_state(&self) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_6()
            .p_8()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .size_16()
                            .rounded_2xl()
                            .bg(rgb(0x1f2228))
                            .border_1()
                            .border_color(rgb(0x282c34))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_3xl()
                            .child("🐳"),
                    )
                    .child(
                        div()
                            .font_weight(FontWeight::BOLD)
                            .text_xl()
                            .text_color(rgb(0xffffff))
                            .child("What can I help you build today?"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x979da6))
                            .child("DeepSeek-V3 & DeepSeek-R1 powered Native Coding Agent"),
                    ),
            )
            // Quick Action Suggestion Cards
            .child(
                div()
                    .flex()
                    .gap_3()
                    .max_w(gpui::px(768.0))
                    .w_full()
                    .child(
                        div()
                            .flex_1()
                            .p_3p5()
                            .rounded_xl()
                            .bg(rgb(0x15171b))
                            .border_1()
                            .border_color(rgb(0x282c34))
                            .hover(|s| s.bg(rgb(0x1f2228)).border_color(rgb(0x4176e6)))
                            .cursor_pointer()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(div().text_sm().child("💡 探索代码库架构"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x979da6))
                                    .child("分析当前工程模块与核心逻辑"),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .p_3p5()
                            .rounded_xl()
                            .bg(rgb(0x15171b))
                            .border_1()
                            .border_color(rgb(0x282c34))
                            .hover(|s| s.bg(rgb(0x1f2228)).border_color(rgb(0x4176e6)))
                            .cursor_pointer()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(div().text_sm().child("⚡ 重构核心组件"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x979da6))
                                    .child("120 FPS 异步流式优化"),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .p_3p5()
                            .rounded_xl()
                            .bg(rgb(0x15171b))
                            .border_1()
                            .border_color(rgb(0x282c34))
                            .hover(|s| s.bg(rgb(0x1f2228)).border_color(rgb(0x4176e6)))
                            .cursor_pointer()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(div().text_sm().child("🛠️ 审查当前 Diff 变更"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x979da6))
                                    .child("自动生成自动化回归测试用例"),
                            ),
                    ),
            )
    }

    fn render_active_messages(&self) -> impl IntoElement {
        div()
            .flex_1()
            .p_4()
            .overflow_hidden()
            .flex()
            .flex_col()
            .gap_4()
            // User Bubble
            .child(
                div()
                    .flex()
                    .justify_end()
                    .child(
                        div()
                            .max_w_3_4()
                            .p_3p5()
                            .rounded_2xl()
                            .bg(rgb(0x4176e6))
                            .text_sm()
                            .text_color(rgb(0xffffff))
                            .child("请帮我使用 Rust + GPUI 重构 DeepSeek Harness 原生桌面端！"),
                    ),
            )
            // Assistant Card
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .max_w_full()
                    .p_4()
                    .rounded_2xl()
                    .bg(rgb(0x15171b))
                    .border_1()
                    .border_color(rgb(0x282c34))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_base().child("🐳"))
                            .child(
                                div()
                                    .font_weight(FontWeight::BOLD)
                                    .text_xs()
                                    .text_color(rgb(0x4176e6))
                                    .child("DeepSeek-V3"),
                            ),
                    )
                    // Tool Call Badge
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_2p5()
                            .py_1()
                            .rounded_full()
                            .bg(rgb(0x1f2228))
                            .border_1()
                            .border_color(rgb(0x282c34))
                            .text_xs()
                            .text_color(rgb(0x979da6))
                            .child("🔧 grep_search (100ms)")
                            .child("✓"),
                    )
                    .children(
                        StreamingMarkdownParser::parse_markdown(
                            "已成功建立与 **DeepSeek Harness** 守护进程的 120 FPS 极速双向数据管道！\n\n```rust\npub async fn run_agent() {\n    println!(\"DeepSeek Harness Native Desktop Ready!\");\n}\n```\n\n> [!TIP]\n> 完整的原生窗口拖拽、流式打字与代码 Diff 审查现已全面就绪。",
                        )
                        .into_iter()
                        .map(|b| self.render_markdown_block(b)),
                    ),
            )
    }
}

impl Render for ChatView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let has_messages = self.active_preset_sent;

        div()
            .flex_1()
            .h_full()
            .bg(rgb(0x0f1115))
            .flex()
            .flex_col()
            .justify_between()
            .relative()
            // Messages / Empty Area
            .child(
                if has_messages {
                    self.render_active_messages().into_any_element()
                } else {
                    self.render_empty_state().into_any_element()
                }
            )
            // Floating Input Composer (Official DeepSeek UI Style)
            .child(
                div()
                    .p_4()
                    .bg(rgb(0x0f1115))
                    .flex()
                    .flex_col()
                    .items_center()
                    .child(
                        div()
                            .max_w(gpui::px(768.0))
                            .w_full()
                            .rounded_2xl()
                            .bg(rgb(0x15171b))
                            .border_1()
                            .border_color(rgb(0x282c34))
                            .p_3()
                            .flex()
                            .flex_col()
                            .gap_2()
                            // Input line
                            .child(
                                div()
                                    .flex_1()
                                    .px_2()
                                    .py_1p5()
                                    .text_sm()
                                    .text_color(rgb(0xe4e4e7))
                                    .child("输入需求，按 Enter 发送，或键入 @ 引用文件与 MCP 工具..."),
                            )
                            // Composer Footer Controls
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .px_2()
                                                    .py_1()
                                                    .rounded_md()
                                                    .bg(rgb(0x1f2228))
                                                    .text_xs()
                                                    .text_color(rgb(0x979da6))
                                                    .child("🤖 DeepSeek-V3"),
                                            )
                                            .child(
                                                div()
                                                    .px_2()
                                                    .py_1()
                                                    .rounded_md()
                                                    .bg(rgb(0x1f2228))
                                                    .text_xs()
                                                    .text_color(rgb(0x4176e6))
                                                    .child("⚡ Full Access"),
                                            ),
                                    )
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
                                            .child("⏎ Send"),
                                    ),
                            ),
                    ),
            )
    }
}
