use dsh_markdown::{CodeHighlighter, InlineSpan, MarkdownBlock, StreamingMarkdownParser, TokenType};
use gpui::{div, prelude::*, rgb, Context, FontWeight, IntoElement, Window};

pub struct ChatMessageView {
    pub is_user: bool,
    pub content: String,
    pub tool_calls: Vec<ToolCallView>,
}

pub struct ToolCallView {
    pub name: String,
    pub status: String,
    pub duration_ms: u64,
}

pub struct ChatView {
    pub messages: Vec<ChatMessageView>,
    pub input_text: String,
}

impl Default for ChatView {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatView {
    pub fn new() -> Self {
        Self {
            messages: vec![
                ChatMessageView {
                    is_user: true,
                    content: "请帮我用 Rust + GPUI 重构 DeepSeek Harness 的桌面端，并支持 Markdown 语法高亮。".into(),
                    tool_calls: vec![],
                },
                ChatMessageView {
                    is_user: false,
                    content: "已经为您规划并创建了完整的 Cargo Workspace 架构！以下是核心协议定义：\n\n```rust\npub enum HarnessServerEvent {\n    TokenChunk { text: String },\n    ToolCallStart { name: String },\n}\n```\n\n> [!TIP]\n> GPUI 渲染引擎可达到 120 FPS 极速刷新，内存开销极低。".into(),
                    tool_calls: vec![
                        ToolCallView {
                            name: "cargo_check".into(),
                            status: "success".into(),
                            duration_ms: 320,
                        },
                    ],
                },
            ],
            input_text: String::new(),
        }
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
                    // Code header
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
                    // Code content
                    .child(
                        div().p_3().text_xs().flex().flex_col().children(
                            spans.into_iter().map(|span| {
                                let color = match span.token_type {
                                    TokenType::Keyword => rgb(0xf43f5e),  // rose
                                    TokenType::Function => rgb(0x38bdf8), // sky
                                    TokenType::Type => rgb(0xfbbf24),     // amber
                                    TokenType::String => rgb(0x4ade80),   // green
                                    TokenType::Comment => rgb(0x71717a),  // zinc
                                    TokenType::Number => rgb(0xc084fc),   // purple
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
                    .children(self.messages.iter().map(|msg| {
                        if msg.is_user {
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
                                        .child(msg.content.clone()),
                                )
                        } else {
                            let blocks = StreamingMarkdownParser::parse_markdown(&msg.content);
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
                                // Tool call cards
                                .children(msg.tool_calls.iter().map(|tc| {
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
                                        .child(tc.name.clone())
                                        .child(format!("({}ms)", tc.duration_ms))
                                        .child("✓")
                                }))
                                // Markdown blocks
                                .children(blocks.into_iter().map(|b| self.render_markdown_block(b)))
                        }
                    })),
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
                            .text_color(rgb(0xa1a1aa))
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
                            .child("发送 ⏎"),
                    ),
            )
    }
}
