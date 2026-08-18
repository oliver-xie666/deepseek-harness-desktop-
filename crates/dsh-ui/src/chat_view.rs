use crate::details_drawer::DetailsDrawer;
use crate::dropdown::{AgentPresetSelector, WorkspaceSelector};
use crate::icons;
use crate::text_input::TextInput;
use dsh_core::AppState;
use dsh_markdown::{
    CodeHighlighter, InlineSpan, MarkdownBlock, StreamingMarkdownParser, TokenType,
};
use gpui::{div, prelude::*, px, rgb, rgba, Context, Entity, FontWeight, IntoElement, Window};
use std::sync::Arc;
use std::time::Duration;

pub struct ChatView {
    pub state: Entity<Arc<AppState>>,
    pub details_drawer: Entity<DetailsDrawer>,
    pub text_input: Entity<TextInput>,
    pub workspace_selector: Entity<WorkspaceSelector>,
    pub preset_selector: Entity<AgentPresetSelector>,
    pub has_messages: bool,
    pub has_active_session: bool,
    pub active_prompt: String,
    pub active_session_title: String,
    pub active_turns: usize,
    pub active_steps: usize,
    pub active_tool_ms: u64,
    pub streaming_text: String,
}

impl ChatView {
    pub fn new(
        state: Entity<Arc<AppState>>,
        details_drawer: Entity<DetailsDrawer>,
        cx: &mut Context<Self>,
    ) -> Self {
        let text_input = cx.new(|cx| TextInput::new("输入消息…", cx));
        let workspace_selector = cx.new(|_| WorkspaceSelector::new());
        let preset_selector = cx.new(|_| AgentPresetSelector::new());

        let view = Self {
            state,
            details_drawer,
            text_input,
            workspace_selector,
            preset_selector,
            has_messages: false,
            has_active_session: false,
            active_prompt: String::new(),
            active_session_title: String::new(),
            active_turns: 0,
            active_steps: 0,
            active_tool_ms: 0,
            streaming_text: String::new(),
        };

        // AppState is shared with the Tokio WebSocket task, so bridge its
        // updates into this GPUI entity and redraw only when content changes.
        let state = view.state.read(cx).clone();
        cx.spawn(async move |this, cx| {
            let mut last_snapshot = String::new();
            loop {
                tokio::time::sleep(Duration::from_millis(50)).await;

                let snapshot = {
                    let active_id = state.active_session_id.read().await.clone();
                    let sessions = state.sessions.read().await;
                    active_id
                        .and_then(|id| sessions.get(&id).cloned())
                        .map(|session| {
                            let text = session
                                .messages
                                .iter()
                                .map(|message| message.content.as_str())
                                .collect::<Vec<_>>()
                                .join("\n\n");
                            (session, text)
                        })
                };

                let Some((session, text)) = snapshot else {
                    continue;
                };

                let snapshot_key = format!("{}:{}:{}", session.id, session.title, text);
                if snapshot_key == last_snapshot {
                    continue;
                }
                last_snapshot = snapshot_key;

                let turns = session
                    .messages
                    .iter()
                    .filter(|message| matches!(message.sender, dsh_core::MessageSender::User))
                    .count();
                let assistant_steps = session
                    .messages
                    .iter()
                    .filter(|message| matches!(message.sender, dsh_core::MessageSender::Assistant))
                    .count();
                let tool_steps = session
                    .messages
                    .iter()
                    .map(|message| message.tool_calls.len())
                    .sum::<usize>();
                let tool_ms = session
                    .messages
                    .iter()
                    .flat_map(|message| message.tool_calls.iter())
                    .map(|tool| tool.duration_ms)
                    .sum::<u64>();

                this.update(cx, |view, cx| {
                    view.has_messages = !session.messages.is_empty();
                    view.has_active_session = view.has_messages || session.title != "新会话";
                    view.active_session_title = session.title.clone();
                    view.active_turns = turns;
                    view.active_steps = assistant_steps + tool_steps;
                    view.active_tool_ms = tool_ms;
                    view.active_prompt = session
                        .messages
                        .iter()
                        .find(|message| matches!(message.sender, dsh_core::MessageSender::User))
                        .map(|message| message.content.clone())
                        .unwrap_or_default();
                    view.streaming_text = text;
                    cx.notify();
                })?;
            }
            #[allow(unreachable_code)]
            Ok::<(), anyhow::Error>(())
        })
        .detach();

        view
    }

    pub fn reset(&mut self, cx: &mut Context<Self>) {
        self.has_messages = false;
        self.has_active_session = false;
        self.active_prompt.clear();
        self.active_session_title.clear();
        self.active_turns = 0;
        self.active_steps = 0;
        self.active_tool_ms = 0;
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
                    .text_color(rgb(0x0f1115))
                    .py_1()
                    .child(format!("{} {}", "#".repeat(level as usize), text))
            }
            MarkdownBlock::Paragraph { inlines } => div()
                .py_1()
                .flex()
                .flex_wrap()
                .gap_1()
                .children(inlines.into_iter().map(|span| {
                    match span {
                        InlineSpan::Bold(t) => div()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x0f1115))
                            .child(t),
                        InlineSpan::Italic(t) => div().text_color(rgb(0x61666b)).child(t),
                        InlineSpan::Code(t) => div()
                            .px_1p5()
                            .py_0p5()
                            .rounded_md()
                            .bg(rgb(0xf1f3f5))
                            .text_color(rgb(0x0f1115))
                            .child(t),
                        InlineSpan::Text(t) => div().text_color(rgb(0x3f454d)).child(t),
                        InlineSpan::Link { text, .. } => {
                            div().text_color(rgb(0x60a5fa)).cursor_pointer().child(text)
                        }
                        InlineSpan::FilePath { path, .. } => div()
                            .px_1p5()
                            .py_0p5()
                            .rounded_md()
                            .bg(rgb(0x2c2c2e))
                            .text_color(rgb(0x679efe))
                            .cursor_pointer()
                            .child(path),
                    }
                })),
            MarkdownBlock::CodeBlock { language, code } => {
                let spans = CodeHighlighter::highlight(&code, &language);

                div()
                    .my_2()
                    .rounded(px(12.0))
                    .bg(rgb(0xf5f6f8))
                    .border_1()
                    .border_color(rgb(0xe1e5eb))
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_3()
                            .py_1p5()
                            .bg(rgb(0xf5f6f8))
                            .text_xs()
                            .text_color(rgb(0x81858c))
                            .child(language)
                            .child(
                                div()
                                    .cursor_pointer()
                                    .hover(|s| s.text_color(rgb(0x0f1115)))
                                    .child("复制"),
                            ),
                    )
                    .child(
                        div()
                            .p_3()
                            .text_xs()
                            .flex()
                            .flex_col()
                            .children(spans.into_iter().map(|span| {
                                let color = match span.token_type {
                                    TokenType::Keyword => rgb(0xf43f5e),
                                    TokenType::Function => rgb(0x4176e6),
                                    TokenType::Type => rgb(0xfbbf24),
                                    TokenType::String => rgb(0x4ade80),
                                    TokenType::Comment => rgb(0x61666b),
                                    TokenType::Number => rgb(0xc084fc),
                                    _ => rgb(0x3f454d),
                                };
                                div().text_color(color).child(span.text)
                            })),
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
                    .rounded(px(12.0))
                    .bg(rgb(0xe8f0ff))
                    .border_l_4()
                    .border_color(rgb(0x3964fe))
                    .text_xs()
                    .text_color(rgb(0x3f454d))
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
            .relative()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_3()
            .p_8()
            // Soft blue glow backdrop (official HeroGlow)
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(icons::glow(620.0, 276.0, rgba(0x6187D814))),
            )
            // Fish + headline + preview badge
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2p5()
                    .child(icons::fish(34.0, rgb(0x0f1115)))
                    .child(
                        div()
                            .font_weight(FontWeight::MEDIUM)
                            .text_size(px(26.0))
                            .line_height(px(32.0))
                            .text_color(rgb(0x0f1115))
                            .child("探索未至之境"),
                    )
                    .child(
                        div()
                            .px_2()
                            .py_0p5()
                            .rounded_full()
                            .bg(rgb(0xe8f0ff))
                            .border_1()
                            .border_color(rgb(0xd7e4ff))
                            .text_xs()
                            .text_color(rgb(0x3964fe))
                            .child("预览版"),
                    ),
            )
            // Workspace + preset chips row, then the composer card
            .child(
                div()
                    .w(px(776.0))
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_1()
                            .child(self.workspace_selector.clone())
                            .child(self.preset_selector.clone()),
                    )
                    .child(
                        div()
                            .w_full()
                            .min_h(px(120.0))
                            .rounded(px(22.0))
                            .bg(rgb(0xffffff))
                            .border_1()
                            .border_color(rgb(0xe1e5eb))
                            .shadow_lg()
                            .pt_2p5()
                            .px_3p5()
                            .pb_3()
                            .flex()
                            .flex_col()
                            .justify_between()
                            .child(self.text_input.clone())
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .pt_2()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .size(px(28.0))
                                                    .rounded_full()
                                                    .bg(rgb(0xf1f3f5))
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .text_sm()
                                                    .text_color(rgb(0x61666b))
                                                    .hover(|s| {
                                                        s.bg(rgb(0xe9edf2))
                                                            .text_color(rgb(0x0f1115))
                                                    })
                                                    .cursor_pointer()
                                                    .child(icons::plus(14.0, rgb(0x61666b))),
                                            )
                                            .child(self.render_access_selector()),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_3()
                                            .child(self.render_model_selector())
                                            .child(
                                                div()
                                                    .size(px(34.0))
                                                    .rounded_full()
                                                    .bg(rgb(0xadc6ff))
                                                    .hover(|s| s.bg(rgb(0x679efe)))
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .text_sm()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(rgb(0xffffff))
                                                    .cursor_pointer()
                                                    .on_mouse_down(
                                                        gpui::MouseButton::Left,
                                                        handle_submit,
                                                    )
                                                    .child(icons::send(16.0, rgb(0xffffff))),
                                            ),
                                    ),
                            ),
                    ),
            )
    }

    fn render_session_header(&self) -> impl IntoElement {
        let title = if self.active_session_title.is_empty() {
            "新会话"
        } else {
            &self.active_session_title
        };

        div()
            .w_full()
            .flex()
            .flex_col()
            .border_b_1()
            .border_color(rgb(0xe5e7eb))
            .child(
                div()
                    .h(px(42.0))
                    .px_4()
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
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(0x0f1115))
                                    .child(title.to_string()),
                            )
                            .child(icons::agent_preset(14.0, rgb(0x61666b)))
                            .child(div().text_xs().text_color(rgb(0x61666b)).child("PTC 模式"))
                            .child(icons::chevron_down(12.0, rgb(0x81858c))),
                    )
                    .child(
                        div()
                            .px_2p5()
                            .py_1()
                            .rounded(px(8.0))
                            .border_1()
                            .border_color(rgb(0xe1e5eb))
                            .text_xs()
                            .text_color(rgb(0x61666b))
                            .child("Session log ↓"),
                    ),
            )
            .child(
                div()
                    .h(px(30.0))
                    .px_4()
                    .flex()
                    .items_end()
                    .gap_5()
                    .child(
                        div()
                            .h_full()
                            .flex()
                            .items_center()
                            .border_b_2()
                            .border_color(rgb(0x3964fe))
                            .text_xs()
                            .text_color(rgb(0x3964fe))
                            .child("对话"),
                    )
                    .child(
                        div()
                            .h_full()
                            .flex()
                            .items_center()
                            .text_xs()
                            .text_color(rgb(0x81858c))
                            .child("轨迹"),
                    ),
            )
    }

    fn render_access_selector(&self) -> impl IntoElement {
        div()
            .h(px(28.0))
            .flex()
            .items_center()
            .gap_1()
            .px_1p5()
            .rounded_md()
            .hover(|s| s.bg(rgb(0xf1f3f5)))
            .cursor_pointer()
            .child(icons::check(14.0, rgb(0x61666b)))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x3f454d))
                    .child("Full access"),
            )
            .child(icons::chevron_down(12.0, rgb(0x81858c)))
    }

    fn render_model_selector(&self) -> impl IntoElement {
        div()
            .h(px(28.0))
            .flex()
            .items_center()
            .gap_1()
            .px_1p5()
            .rounded_md()
            .hover(|s| s.bg(rgb(0xf1f3f5)))
            .cursor_pointer()
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x3f454d))
                    .child("gpt-5.6-luna"),
            )
            .child(icons::chevron_down(12.0, rgb(0x81858c)))
    }

    fn render_stats_line(&self) -> impl IntoElement {
        let turns = self.active_turns.max(1);
        let steps = self.active_steps.max(turns);
        let summary = format!("{} 轮 · {} 步", turns, steps);
        let tool_summary = if self.active_tool_ms > 0 {
            format!("工具调用 {}ms", self.active_tool_ms)
        } else {
            "工具调用 --".to_string()
        };

        div()
            .max_w(px(776.0))
            .w_full()
            .pt_1()
            .px_2()
            .text_xs()
            .text_color(rgb(0x81858c))
            .child(format!(
                "{} | LLM -- · {} | 首 token -- · -- tok/s | 缓存命中 -- | 输入 -- tok · 输出 -- tok",
                summary, tool_summary
            ))
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
                drawer.open_tool(
                    "grep_search",
                    100,
                    "{\n  \"query\": \"start_harness\",\n  \"path\": \"crates/\"\n}",
                    "crates/dsh-core/src/lib.rs:84\n1 match found.",
                    cx,
                );
            });
        });

        div()
            .flex_1()
            .overflow_hidden()
            .flex()
            .flex_col()
            .child(self.render_session_header())
            .child(
                div()
                    .flex_1()
                    .p_6()
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div().flex().justify_end().child(
                            div()
                                .max_w_3_4()
                                .px_4()
                                .py_2p5()
                                .rounded_2xl()
                                .bg(rgb(0xe8f0ff))
                                .text_sm()
                                .text_color(rgb(0x0f1115))
                                .child(user_prompt.to_string()),
                        ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .max_w_full()
                            .p_4()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .px_3()
                                    .py_1p5()
                                    .rounded_md()
                                    .bg(rgb(0xf5f6f8))
                                    .border_1()
                                    .border_color(rgb(0xe1e5eb))
                                    .hover(|s| s.bg(rgb(0xf1f3f5)).border_color(rgb(0x3964fe)))
                                    .cursor_pointer()
                                    .on_mouse_down(gpui::MouseButton::Left, handle_tool_click)
                                    .child(icons::agent_preset(14.0, rgb(0x61666b)))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x61666b))
                                            .child("grep_search (100ms)"),
                                    )
                                    .child(div().text_xs().text_color(rgb(0x16a34a)).child("✓")),
                            )
                            .children(
                                StreamingMarkdownParser::parse_markdown(&self.streaming_text)
                                    .into_iter()
                                    .map(|b| self.render_markdown_block(b)),
                            ),
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
            .bg(rgb(0xffffff))
            .flex()
            .flex_col()
            .justify_between()
            .relative()
            .child(if has_messages || self.has_active_session {
                self.render_active_messages(cx).into_any_element()
            } else {
                self.render_empty_state(window, cx).into_any_element()
            })
            .when(has_messages || self.has_active_session, |this| {
                this.child(
                    div()
                        .p_4()
                        .bg(rgb(0xffffff))
                        .flex()
                        .flex_col()
                        .items_center()
                        .child(
                            div()
                                .max_w(px(776.0))
                                .w_full()
                                .rounded_2xl()
                                .bg(rgb(0xffffff))
                                .border_1()
                                .border_color(rgb(0xe1e5eb))
                                .p_3()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .child(self.text_input.clone())
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
                                                        .size(px(28.0))
                                                        .rounded_full()
                                                        .bg(rgb(0xf1f3f5))
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .hover(|s| s.bg(rgb(0xe9edf2)))
                                                        .cursor_pointer()
                                                        .child(icons::plus(14.0, rgb(0x61666b))),
                                                )
                                                .child(self.render_access_selector()),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_3()
                                                .child(self.render_model_selector())
                                                .child(
                                                    div()
                                                        .size(px(34.0))
                                                        .rounded_full()
                                                        .bg(rgb(0x679efe))
                                                        .hover(|s| s.bg(rgb(0x4176e6)))
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .text_sm()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(rgb(0xffffff))
                                                        .cursor_pointer()
                                                        .on_mouse_down(
                                                            gpui::MouseButton::Left,
                                                            handle_submit,
                                                        )
                                                        .child(icons::send(16.0, rgb(0xffffff))),
                                                ),
                                        ),
                                ),
                        )
                        .child(self.render_stats_line()),
                )
            })
    }
}
