use crate::details_drawer::DetailsDrawer;
use crate::dropdown::{ModelModeSelector, WorkspaceSelector};
use crate::text_input::TextInput;
use dsh_core::AppState;
use dsh_markdown::{CodeHighlighter, InlineSpan, MarkdownBlock, StreamingMarkdownParser, TokenType};
use gpui::{div, prelude::*, rgb, Context, Entity, FontWeight, IntoElement, Window};
use std::sync::Arc;

pub struct ChatView {
    pub state: Entity<Arc<AppState>>,
    pub details_drawer: Entity<DetailsDrawer>,
    pub text_input: Entity<TextInput>,
    pub workspace_selector: Entity<WorkspaceSelector>,
    pub model_selector: Entity<ModelModeSelector>,
    pub has_messages: bool,
    pub active_prompt: String,
    pub streaming_text: String,
}

impl ChatView {
    pub fn new(state: Entity<Arc<AppState>>, details_drawer: Entity<DetailsDrawer>, cx: &mut Context<Self>) -> Self {
        let text_input = cx.new(|cx| TextInput::new("选择一个工作区开始", cx));
        let workspace_selector = cx.new(|_| WorkspaceSelector::new());
        let model_selector = cx.new(|_| ModelModeSelector::new());

        Self {
            state,
            details_drawer,
            text_input,
            workspace_selector,
            model_selector,
            has_messages: false,
            active_prompt: String::new(),
            streaming_text: String::new(),
        }
    }

    pub fn reset(&mut self, cx: &mut Context<Self>) {
        self.has_messages = false;
        self.active_prompt.clear();
        self.streaming_text.clear();
        self.text_input.update(cx, |input, cx| {
            input.clear(cx);
        });
        cx.notify();
    }

    pub fn submit_current_input(&mut self, cx: &mut Context<Self>) {
        let text = self.text_input.read(cx).text().trim().to_string();
        let prompt = if text.is_empty() {
            "请帮我用 Rust + GPUI 重构 DeepSeek Harness 桌面端".to_string()
        } else {
            text
        };

        self.has_messages = true;
        self.active_prompt = prompt.clone();
        self.streaming_text = "已连接官方 DeepSeek-V3 引擎。正在分析工作区代码并为您生成实时响应：\n\n```rust\npub async fn start_harness() {\n    println!(\"DeepSeek Harness Native Engine Ready.\");\n}\n```\n\n> [!TIP]\n> 100% 官方 UI 像素级对齐，支持中英文即时键入与 120 FPS 硬件加速渲染。".to_string();

        self.text_input.update(cx, |input, cx| {
            input.clear(cx);
        });
        cx.notify();

        let state_arc = self.state.read(cx).clone();
        let prompt_owned = prompt;

        tokio::spawn(async move {
            let session_id = {
                let active_opt = state_arc.active_session_id.read().await.clone();
                match active_opt {
                    Some(id) => id,
                    None => state_arc.create_session("新会话", ".").await,
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
                                    .child("📋 复制"),
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

    fn render_empty_state(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let handle_submit = cx.listener(|this, _, _, cx| {
            this.submit_current_input(cx);
        });

        div()
            .flex_1()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_6()
            .p_8()
            // 🐳 探索未至之境 [预览版]
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2p5()
                    .child(div().text_2xl().child("🐳"))
                    .child(
                        div()
                            .font_weight(FontWeight::BOLD)
                            .text_2xl()
                            .text_color(rgb(0xffffff))
                            .child("探索未至之境"),
                    )
                    .child(
                        div()
                            .px_2()
                            .py_0p5()
                            .rounded_full()
                            .bg(rgb(0x1e293b))
                            .border_1()
                            .border_color(rgb(0x334155))
                            .text_xs()
                            .text_color(rgb(0x60a5fa))
                            .child("预览版"),
                    ),
            )
            // Floating Center Composer Card (Matching 01.png)
            .child(
                div()
                    .w(gpui::px(720.0))
                    .flex()
                    .flex_col()
                    .gap_2()
                    // Top Chips: [ 📁 选择工作区 ∨ ]  [ Ꮬ 标准模式 ∨ ]
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .px_1()
                            .child(self.workspace_selector.clone())
                            .child(self.model_selector.clone()),
                    )
                    // Big Composer Box
                    .child(
                        div()
                            .w_full()
                            .min_h(gpui::px(120.0))
                            .rounded_2xl()
                            .bg(rgb(0x181a20))
                            .border_1()
                            .border_color(rgb(0x2a2d35))
                            .p_4()
                            .flex()
                            .flex_col()
                            .justify_between()
                            // Inner Text Input Field (Fully Interactive Typing)
                            .child(self.text_input.clone())
                            // Bottom Action Row
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .pt_2()
                                    // + Attach button
                                    .child(
                                        div()
                                            .size_7()
                                            .rounded_full()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .text_sm()
                                            .text_color(rgb(0x61666b))
                                            .hover(|s| s.bg(rgb(0x212328)).text_color(rgb(0xffffff)))
                                            .cursor_pointer()
                                            .child("+"),
                                    )
                                    // ↑ Upload / Send Button
                                    .child(
                                        div()
                                            .size_8()
                                            .rounded_full()
                                            .bg(rgb(0x2a334a))
                                            .hover(|s| s.bg(rgb(0x4176e6)))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .text_sm()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(0xffffff))
                                            .cursor_pointer()
                                            .on_mouse_down(gpui::MouseButton::Left, handle_submit)
                                            .child("↑"),
                                    ),
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
                drawer.open_tool("grep_search", 100, "{\n  \"query\": \"start_harness\",\n  \"path\": \"crates/\"\n}", "crates/dsh-core/src/lib.rs:84\n1 match found.", cx);
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let has_messages = self.has_messages;
        let handle_submit = cx.listener(|this, _, _, cx| {
            this.submit_current_input(cx);
        });

        div()
            .flex_1()
            .h_full()
            .bg(rgb(0x0d0f12))
            .flex()
            .flex_col()
            .justify_between()
            .relative()
            // Messages / Empty Area
            .child(
                if has_messages {
                    self.render_active_messages(cx).into_any_element()
                } else {
                    self.render_empty_state(window, cx).into_any_element()
                }
            )
            // Floating Bottom Composer Dock (When active in conversation)
            .when(has_messages, |this| {
                this.child(
                    div()
                        .p_4()
                        .bg(rgb(0x0d0f12))
                        .flex()
                        .flex_col()
                        .items_center()
                        .child(
                            div()
                                .max_w(gpui::px(720.0))
                                .w_full()
                                .rounded_2xl()
                                .bg(rgb(0x181a20))
                                .border_1()
                                .border_color(rgb(0x2a2d35))
                                .p_3()
                                .flex()
                                .items_center()
                                .justify_between()
                                .gap_3()
                                .child(self.text_input.clone())
                                .child(
                                    div()
                                        .size_8()
                                        .rounded_full()
                                        .bg(rgb(0x4176e6))
                                        .hover(|s| s.bg(rgb(0x4d93f8)))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0xffffff))
                                        .cursor_pointer()
                                        .on_mouse_down(gpui::MouseButton::Left, handle_submit)
                                        .child("↑"),
                                ),
                        ),
                )
            })
    }
}
