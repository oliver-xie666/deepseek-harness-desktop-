use crate::details_drawer::DetailsDrawer;
use dsh_core::AppState;
use dsh_markdown::{CodeHighlighter, InlineSpan, MarkdownBlock, StreamingMarkdownParser, TokenType};
use gpui::{div, prelude::*, rgb, Context, Entity, FontWeight, IntoElement, Window};
use std::sync::Arc;

pub struct ChatView {
    pub state: Entity<Arc<AppState>>,
    pub details_drawer: Entity<DetailsDrawer>,
    pub has_messages: bool,
    pub active_prompt: String,
    pub streaming_text: String,
    pub is_streaming: bool,
}

impl ChatView {
    pub fn new(state: Entity<Arc<AppState>>, details_drawer: Entity<DetailsDrawer>, _cx: &mut Context<Self>) -> Self {
        Self {
            state,
            details_drawer,
            has_messages: false,
            active_prompt: String::new(),
            streaming_text: String::new(),
            is_streaming: false,
        }
    }

    pub fn reset(&mut self, cx: &mut Context<Self>) {
        self.has_messages = false;
        self.active_prompt.clear();
        self.streaming_text.clear();
        self.is_streaming = false;
        cx.notify();
    }

    pub fn send_prompt(&mut self, prompt: &str, cx: &mut Context<Self>) {
        self.has_messages = true;
        self.active_prompt = prompt.to_string();
        self.streaming_text = "正在实时连接 DeepSeek Harness 守护进程...\n\n```rust\npub async fn run_agent() {\n    println!(\"120 FPS Realtime Agent Stream!\");\n}\n```\n\n[ !TIP ] WebSocket IPC 已成功建立双向数据通路!".to_string();
        self.is_streaming = false;
        cx.notify();

        let state_arc = self.state.read(cx).clone();
        let prompt_owned = prompt.to_string();

        tokio::spawn(async move {
            let session_id = {
                let active_opt = state_arc.active_session_id.read().await.clone();
                match active_opt {
                    Some(id) => id,
                    None => state_arc.create_session("New Chat", ".").await,
                }
            };
            let _ = state_arc.add_user_message(&session_id, &prompt_owned).await;
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
                    .text_color(rgb(0xffffff))
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
                            .bg(rgb(0x191c22))
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

    fn render_empty_state(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let handle_card1 = cx.listener(|this, _, _, cx| {
            this.send_prompt("请帮我用 Rust + GPUI 重构 DeepSeek Harness 桌面端", cx);
        });
        let handle_card2 = cx.listener(|this, _, _, cx| {
            this.send_prompt("请帮我分析当前工程中的插件调用机制", cx);
        });
        let handle_card3 = cx.listener(|this, _, _, cx| {
            this.send_prompt("请帮我重构 WebSocket 自动重连与指数退避逻辑", cx);
        });

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
                            .bg(rgb(0x15171b))
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
            // 3 Quick Action Suggestion Cards
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
                            .on_mouse_down(gpui::MouseButton::Left, handle_card1)
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
                            .on_mouse_down(gpui::MouseButton::Left, handle_card2)
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(div().text_sm().child("⚡ 插件系统调用"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x979da6))
                                    .child("IPC 双向协议与 MCP 插件挂载"),
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
                            .on_mouse_down(gpui::MouseButton::Left, handle_card3)
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(div().text_sm().child("🛠️ 异步流式优化"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x979da6))
                                    .child("120 FPS 原生 GPU 流式打字"),
                            ),
                    ),
            )
    }

    fn render_active_messages(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let user_prompt = if self.active_prompt.is_empty() {
            "请帮我用 Rust + GPUI 重构 DeepSeek Harness 桌面端"
        } else {
            &self.active_prompt
        };

        let drawer_entity = self.details_drawer.clone();
        let handle_tool_click = cx.listener(move |_this, _, _, cx| {
            drawer_entity.update(cx, |drawer, cx| {
                drawer.open_tool("grep_search", 100, "{\n  \"query\": \"run_agent\",\n  \"path\": \"crates/\"\n}", "crates/dsh-ui/src/main.rs:42\n1 match found.", cx);
            });
        });

        div()
            .flex_1()
            .p_6()
            .overflow_hidden()
            .flex()
            .flex_col()
            .gap_4()
            // User Message Bubble (DeepSeek Blue, Right Aligned)
            .child(
                div()
                    .flex()
                    .justify_end()
                    .child(
                        div()
                            .max_w_3_4()
                            .px_4()
                            .py_2p5()
                            .rounded_2xl()
                            .bg(rgb(0x4176e6))
                            .text_sm()
                            .text_color(rgb(0xffffff))
                            .child(user_prompt.to_string()),
                    ),
            )
            // Assistant Card
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .max_w_full()
                    .p_4()
                    .rounded_2xl()
                    .bg(rgb(0x15171b))
                    .border_1()
                    .border_color(rgb(0x282c34))
                    // Clickable Tool Call Badge
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(rgb(0x191c22))
                            .border_1()
                            .border_color(rgb(0x282c34))
                            .hover(|s| s.bg(rgb(0x1f2228)).border_color(rgb(0x4176e6)))
                            .cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, handle_tool_click)
                            .child(div().text_xs().child("🔧"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x979da6))
                                    .child("grep_search (100ms)"),
                            )
                            .child(div().text_xs().text_color(rgb(0x22c55e)).child("✓")),
                    )
                    .children(
                        StreamingMarkdownParser::parse_markdown(&self.streaming_text)
                            .into_iter()
                            .map(|b| self.render_markdown_block(b)),
                    ),
            )
    }
}

impl Render for ChatView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let has_messages = self.has_messages;
        let handle_send = cx.listener(|this, _, _, cx| {
            this.send_prompt("请帮我用 Rust + GPUI 重构 DeepSeek Harness 桌面端", cx);
        });

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
                    self.render_active_messages(cx).into_any_element()
                } else {
                    self.render_empty_state(cx).into_any_element()
                }
            )
            // Floating Input Composer (Matching official design)
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
                            .rounded_xl()
                            .bg(rgb(0x15171b))
                            .border_1()
                            .border_color(rgb(0x282c34))
                            .p_2p5()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap_3()
                            // Input line
                            .child(
                                div()
                                    .flex_1()
                                    .px_2()
                                    .py_1()
                                    .text_sm()
                                    .text_color(rgb(0x979da6))
                                    .child("输入指令，按 Enter 发送，或键入 @ 引用代码..."),
                            )
                            // Send Action Button (DeepSeek Blue)
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
                                    .on_mouse_down(gpui::MouseButton::Left, handle_send)
                                    .child("发送 ⏎"),
                            ),
                    ),
            )
    }
}
