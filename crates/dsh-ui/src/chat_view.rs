use crate::details_drawer::DetailsDrawer;
use crate::dropdown::{AgentPresetSelector, WorkspaceSelector};
use crate::icons;
use crate::model_catalog::provider_groups;
use crate::text_input::TextInput;
use dsh_common::AppPaths;
use dsh_core::AppState;
use dsh_markdown::{
    CodeHighlighter, InlineSpan, MarkdownBlock, StreamingMarkdownParser, TokenType,
};
use dsh_protocol::ToolStatus;
use gpui::{
    deferred, div, prelude::*, px, rgb, rgba, Context, Entity, FontWeight, IntoElement,
    MouseButton, ScrollHandle, Subscription, Window,
};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq)]
enum SessionView {
    Chat,
    Trace,
}

#[derive(Clone)]
struct TraceEntry {
    turn: usize,
    kind: &'static str,
    title: String,
    detail: String,
    args: String,
    output: String,
    duration_ms: u64,
    tool_status: Option<ToolStatus>,
}

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
    pub active_llm_ms: u64,
    pub active_ttft_ms: u64,
    pub active_input_tokens: usize,
    pub active_output_tokens: usize,
    pub active_cache_hit: Option<u8>,
    pub goal: Option<dsh_protocol::GoalItem>,
    pub goal_editing: bool,
    pub goal_draft: String,
    pub jobs: Vec<dsh_protocol::JobItem>,
    pub jobs_menu_open: bool,
    pub attachments: Vec<String>,
    pub plan_active: bool,
    pub plan_markdown: String,
    pub pending_question: Option<dsh_protocol::QuestionItem>,
    pub selected_question_options: Vec<String>,
    pub streaming_text: String,
    pub permission_open: bool,
    pub model_open: bool,
    pub permission_mode: String,
    pub model_name: String,
    plus_menu_open: bool,
    conversation_messages: Vec<dsh_core::ChatMessage>,
    active_session_id: Option<String>,
    pending_diffs: Vec<dsh_core::FileDiffItem>,
    diff_notice: Option<String>,
    active_view: SessionView,
    pub session_log_open: bool,
    pub trace_actual_duration: bool,
    pub trace_all_collapsed: bool,
    pub trace_collapsed_turns: HashSet<usize>,
    trace_entries: Vec<TraceEntry>,
    session_log_lines: Vec<String>,
    session_log_scroll_handle: ScrollHandle,
    trace_search_input: Entity<TextInput>,
    _trace_search_subscription: Subscription,
}

#[cfg(test)]
mod diff_notice_tests {
    use super::record_diff_result;

    #[test]
    fn successful_diff_action_clears_previous_notice() {
        let mut notice = Some("旧错误".to_string());
        record_diff_result(&mut notice, None, "无法应用");
        assert_eq!(notice, None);
    }

    #[test]
    fn failed_diff_action_replaces_notice_with_action_context() {
        let mut notice = None;
        record_diff_result(
            &mut notice,
            Some("上下文不匹配".to_string()),
            "无法拒绝变更",
        );
        assert_eq!(notice.as_deref(), Some("无法拒绝变更：上下文不匹配"));
    }
}

#[cfg(test)]
mod session_log_tests {
    use super::session_log_display_lines;

    #[test]
    fn session_log_display_keeps_all_lines_in_newest_first_order() {
        let lines = vec!["one".to_string(), "two".to_string(), "three".to_string()];
        assert_eq!(
            session_log_display_lines(&lines),
            vec!["three", "two", "one"]
        );
    }
}

#[cfg(test)]
mod tool_status_tests {
    use super::tool_status_label;
    use dsh_protocol::ToolStatus;

    #[test]
    fn tool_status_labels_cover_running_success_and_failure() {
        assert_eq!(tool_status_label(ToolStatus::Running), "运行中");
        assert_eq!(tool_status_label(ToolStatus::Success), "成功");
        assert_eq!(tool_status_label(ToolStatus::Failed), "失败");
    }
}

fn record_diff_result(notice: &mut Option<String>, error: Option<String>, action: &str) {
    *notice = error.map(|error| format!("{}：{}", action, error));
}

fn session_log_display_lines(lines: &[String]) -> Vec<String> {
    lines.iter().rev().cloned().collect()
}

fn tool_status_label(status: ToolStatus) -> &'static str {
    match status {
        ToolStatus::Running => "运行中",
        ToolStatus::Success => "成功",
        ToolStatus::Failed => "失败",
    }
}

pub fn goal_phase_label(phase: dsh_protocol::GoalPhase) -> &'static str {
    match phase {
        dsh_protocol::GoalPhase::Active => "进行中",
        dsh_protocol::GoalPhase::Paused => "已暂停",
        dsh_protocol::GoalPhase::Blocked => "阻塞",
        dsh_protocol::GoalPhase::Complete => "已完成",
    }
}

pub fn goal_phase_color(phase: dsh_protocol::GoalPhase) -> (gpui::Rgba, gpui::Rgba, gpui::Rgba) {
    match phase {
        dsh_protocol::GoalPhase::Active => (rgba(0xeff6ffFF), rgba(0xbfdbfeFF), rgba(0x1d4ed8FF)),
        dsh_protocol::GoalPhase::Paused => (rgba(0xf3f4f6FF), rgba(0xe5e7ebFF), rgba(0x4b5563FF)),
        dsh_protocol::GoalPhase::Blocked => (rgba(0xfef2f2FF), rgba(0xfecacaFF), rgba(0xb91c1cFF)),
        dsh_protocol::GoalPhase::Complete => (rgba(0xf0fdf4FF), rgba(0xbbf7d0FF), rgba(0x15803dFF)),
    }
}

pub fn job_status_label(status: dsh_protocol::JobStatus) -> &'static str {
    match status {
        dsh_protocol::JobStatus::Running => "运行中",
        dsh_protocol::JobStatus::Stopping => "正在停止",
        dsh_protocol::JobStatus::Completed => "已完成",
        dsh_protocol::JobStatus::Killed => "已终止",
        dsh_protocol::JobStatus::Failed => "失败",
    }
}

pub fn job_status_dot_color(status: dsh_protocol::JobStatus) -> gpui::Rgba {
    match status {
        dsh_protocol::JobStatus::Running => rgba(0x16803cFF),
        dsh_protocol::JobStatus::Stopping | dsh_protocol::JobStatus::Killed => rgba(0xb7791fFF),
        dsh_protocol::JobStatus::Failed => rgba(0xb42318FF),
        dsh_protocol::JobStatus::Completed => rgba(0x81858cFF),
    }
}

fn tool_status_color(status: ToolStatus) -> gpui::Rgba {
    match status {
        ToolStatus::Running => rgba(0xb7791fFF),
        ToolStatus::Success => rgba(0x16803cFF),
        ToolStatus::Failed => rgba(0xb42318FF),
    }
}

impl ChatView {
    pub fn new(
        state: Entity<Arc<AppState>>,
        details_drawer: Entity<DetailsDrawer>,
        cx: &mut Context<Self>,
    ) -> Self {
        let text_input = cx.new(|cx| TextInput::new("输入消息…", cx));
        let trace_search_input = cx.new(|cx| TextInput::new("搜索轨迹…", cx));
        let preset_state = state.read(cx).clone();
        let workspace_state = state.read(cx).clone();
        let workspace_selector = cx.new(|_| WorkspaceSelector::with_state(workspace_state));
        let preset_selector = cx.new(|_| AgentPresetSelector::with_state(preset_state));

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
            active_llm_ms: 0,
            active_ttft_ms: 0,
            active_input_tokens: 0,
            active_output_tokens: 0,
            active_cache_hit: None,
            goal: None,
            goal_editing: false,
            goal_draft: String::new(),
            jobs: Vec::new(),
            jobs_menu_open: false,
            attachments: Vec::new(),
            plan_active: false,
            plan_markdown: String::new(),
            pending_question: None,
            selected_question_options: Vec::new(),
            streaming_text: String::new(),
            permission_open: false,
            model_open: false,
            permission_mode: "Full access".into(),
            model_name: "gpt-5.6-luna".into(),
            plus_menu_open: false,
            conversation_messages: Vec::new(),
            active_session_id: None,
            pending_diffs: Vec::new(),
            diff_notice: None,
            active_view: SessionView::Chat,
            session_log_open: false,
            trace_actual_duration: false,
            trace_all_collapsed: false,
            trace_collapsed_turns: HashSet::new(),
            trace_entries: Vec::new(),
            session_log_lines: Vec::new(),
            session_log_scroll_handle: ScrollHandle::new(),
            trace_search_input: trace_search_input.clone(),
            _trace_search_subscription: cx.observe(&trace_search_input, |_, _, cx| cx.notify()),
        };

        // AppState is shared with the Tokio WebSocket task, so bridge its
        // updates into this GPUI entity and redraw only when content changes.
        let state = view.state.read(cx).clone();
        cx.spawn(async move |this, cx| {
            let mut last_snapshot = String::new();
            loop {
                tokio::time::sleep(Duration::from_millis(50)).await;

                let config = state.config.read().await.clone();
                let workspace_label = state
                    .workspace_path
                    .read()
                    .await
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("工作区")
                    .to_string();
                let permission_mode = match config.ui.permission_mode.as_str() {
                    "read-only" => "Read Only",
                    "workspace-write" => "Workspace Write",
                    _ => "Full access",
                }
                .to_string();
                let model_name = config.model.model_name.clone();
                let preset_name = match config.ui.agent_preset.as_str() {
                    "code" => "PTC 模式",
                    "minimal" => "极简模式",
                    "cordis" => "创造模式",
                    _ => "标准模式",
                }
                .to_string();

                let should_submit = this.update(cx, |view, cx| {
                    view.text_input
                        .update(cx, |input, cx| input.take_submit_requested(cx))
                })?;
                if should_submit {
                    this.update(cx, |view, cx| view.submit_current_input(cx))?;
                }
                this.update(cx, |view, cx| {
                    view.workspace_selector.update(cx, |selector, cx| {
                        selector.sync_workspace(&workspace_label, cx)
                    });
                })?;

                let snapshot = {
                    let active_id = state.active_session_id.read().await.clone();
                    let sessions = state.sessions.read().await;
                    active_id
                        .and_then(|id| sessions.get(&id).cloned())
                        .map(|session| {
                            let text = session
                                .messages
                                .iter()
                                .filter(|message| {
                                    matches!(message.sender, dsh_core::MessageSender::Assistant)
                                })
                                .map(|message| message.content.as_str())
                                .collect::<Vec<_>>()
                                .join("\n\n");
                            (session, text)
                        })
                };

                let Some((session, text)) = snapshot else {
                    let config_key =
                        format!("config:{}:{}:{}", permission_mode, model_name, preset_name);
                    if config_key != last_snapshot {
                        last_snapshot = config_key;
                        this.update(cx, |view, cx| {
                            view.permission_mode = permission_mode.clone();
                            view.model_name = model_name.clone();
                            view.text_input.update(cx, |input, cx| {
                                input.set_enter_behavior(&config.ui.enter_behavior, cx)
                            });
                            view.preset_selector.update(cx, |selector, cx| {
                                selector.set_preset(&preset_name, cx);
                            });
                            cx.notify();
                        })?;
                    }
                    continue;
                };

                let diff_key = session
                    .diffs
                    .values()
                    .map(|diff| format!("{}:{:?}:{}", diff.id, diff.accepted, diff.diff_content))
                    .collect::<Vec<_>>()
                    .join("|");
                let tool_key = session
                    .messages
                    .iter()
                    .flat_map(|message| message.tool_calls.iter())
                    .map(|tool| {
                        format!(
                            "{}:{:?}:{}:{:?}",
                            tool.id, tool.status, tool.duration_ms, tool.output
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("|");
                let log_key = session.terminal_logs.join("\n");
                let snapshot_key = format!(
                    "{}:{}:{}:{}:{}:{}:{}:{}:{}",
                    session.id,
                    session.title,
                    text,
                    permission_mode,
                    model_name,
                    preset_name,
                    diff_key,
                    tool_key,
                    log_key
                );
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

                let mut trace_entries = Vec::new();
                let mut turn = 0;
                for message in &session.messages {
                    match message.sender {
                        dsh_core::MessageSender::User => {
                            turn += 1;
                            trace_entries.push(TraceEntry {
                                turn,
                                kind: "USER",
                                title: "Message".into(),
                                detail: message.content.clone(),
                                args: String::new(),
                                output: String::new(),
                                duration_ms: 0,
                                tool_status: None,
                            });
                        }
                        dsh_core::MessageSender::Assistant => {
                            trace_entries.push(TraceEntry {
                                turn: turn.max(1),
                                kind: "ASSISTANT",
                                title: format!("Step {}", message.tool_calls.len().max(1)),
                                detail: message.content.clone(),
                                args: String::new(),
                                output: message.content.clone(),
                                duration_ms: message
                                    .tool_calls
                                    .iter()
                                    .map(|tool| tool.duration_ms)
                                    .sum(),
                                tool_status: None,
                            });
                            for tool in &message.tool_calls {
                                trace_entries.push(TraceEntry {
                                    turn: turn.max(1),
                                    kind: "TOOL",
                                    title: tool.tool_name.clone(),
                                    detail: tool.tool_name.clone(),
                                    args: tool.input.to_string(),
                                    output: tool
                                        .output
                                        .as_ref()
                                        .map(ToString::to_string)
                                        .unwrap_or_else(|| "等待工具返回".into()),
                                    duration_ms: tool.duration_ms,
                                    tool_status: Some(tool.status),
                                });
                            }
                        }
                        dsh_core::MessageSender::System => trace_entries.push(TraceEntry {
                            turn: turn.max(1),
                            kind: "SYSTEM",
                            title: "Context".into(),
                            detail: message.content.clone(),
                            args: String::new(),
                            output: message.content.clone(),
                            duration_ms: 0,
                            tool_status: None,
                        }),
                    }
                }

                this.update(cx, |view, cx| {
                    view.has_messages = !session.messages.is_empty();
                    view.has_active_session = view.has_messages || session.title != "新会话";
                    view.active_session_title = session.title.clone();
                    view.active_turns = turns;
                    view.active_steps = assistant_steps + tool_steps;
                    view.active_tool_ms = tool_ms;
                    view.permission_mode = permission_mode.clone();
                    view.model_name = model_name.clone();
                    view.text_input.update(cx, |input, cx| {
                        input.set_enter_behavior(&config.ui.enter_behavior, cx)
                    });
                    view.preset_selector.update(cx, |selector, cx| {
                        selector.set_preset(&preset_name, cx);
                    });
                    view.active_prompt = session
                        .messages
                        .iter()
                        .find(|message| matches!(message.sender, dsh_core::MessageSender::User))
                        .map(|message| message.content.clone())
                        .unwrap_or_default();
                    view.streaming_text = text;
                    view.trace_entries = trace_entries;
                    view.goal = session.goal.clone();
                    view.jobs = session.jobs.clone();
                    view.plan_active = session
                        .plan_state
                        .as_ref()
                        .map(|p| p.active)
                        .unwrap_or(false);
                    view.plan_markdown = session
                        .plan_state
                        .as_ref()
                        .map(|p| p.plan_markdown.clone())
                        .unwrap_or_default();
                    view.pending_question = session.pending_question.clone();
                    view.session_log_lines = session.terminal_logs.clone();
                    view.session_log_scroll_handle.scroll_to_bottom();
                    view.conversation_messages = session.messages.clone();
                    view.active_session_id = Some(session.id.clone());
                    let mut pending_diffs = session
                        .diffs
                        .values()
                        .filter(|diff| diff.accepted.is_none())
                        .cloned()
                        .collect::<Vec<_>>();
                    pending_diffs.sort_by(|left, right| left.file_path.cmp(&right.file_path));
                    view.pending_diffs = pending_diffs;
                    cx.notify();
                })?;
            }
            #[allow(unreachable_code)]
            Ok::<(), anyhow::Error>(())
        })
        .detach();

        view
    }

    pub fn start_edit_goal(&mut self, cx: &mut Context<Self>) {
        if let Some(goal) = &self.goal {
            self.goal_draft = goal.objective.clone();
            self.goal_editing = true;
            cx.notify();
        }
    }

    pub fn save_edit_goal(&mut self, cx: &mut Context<Self>) {
        let trimmed = self.goal_draft.trim().to_string();
        if trimmed.is_empty() {
            return;
        }
        if let Some(session_id) = self.active_session_id.clone() {
            let state = self.state.read(cx).clone();
            let phase = self
                .goal
                .as_ref()
                .map(|g| g.phase)
                .unwrap_or(dsh_protocol::GoalPhase::Active);
            self.goal_editing = false;
            tokio::spawn(async move {
                let _ = state.update_goal(&session_id, &trimmed, phase).await;
            });
            cx.notify();
        }
    }

    pub fn cancel_edit_goal(&mut self, cx: &mut Context<Self>) {
        self.goal_editing = false;
        cx.notify();
    }

    pub fn toggle_goal_pause(&mut self, cx: &mut Context<Self>) {
        if let Some(goal) = &self.goal {
            let next_phase = match goal.phase {
                dsh_protocol::GoalPhase::Active => dsh_protocol::GoalPhase::Paused,
                dsh_protocol::GoalPhase::Paused => dsh_protocol::GoalPhase::Active,
                dsh_protocol::GoalPhase::Blocked => dsh_protocol::GoalPhase::Active,
                dsh_protocol::GoalPhase::Complete => dsh_protocol::GoalPhase::Complete,
            };
            if let Some(session_id) = self.active_session_id.clone() {
                let state = self.state.read(cx).clone();
                let objective = goal.objective.clone();
                tokio::spawn(async move {
                    let _ = state.update_goal(&session_id, &objective, next_phase).await;
                });
                cx.notify();
            }
        }
    }

    pub fn clear_goal(&mut self, cx: &mut Context<Self>) {
        if let Some(session_id) = self.active_session_id.clone() {
            let state = self.state.read(cx).clone();
            self.goal = None;
            tokio::spawn(async move {
                let _ = state.clear_goal(&session_id).await;
            });
            cx.notify();
        }
    }

    pub fn toggle_jobs_menu(&mut self, cx: &mut Context<Self>) {
        self.jobs_menu_open = !self.jobs_menu_open;
        if self.jobs_menu_open {
            self.permission_open = false;
            self.model_open = false;
            self.plus_menu_open = false;
        }
        cx.notify();
    }

    pub fn kill_job(&mut self, job_id: &str, cx: &mut Context<Self>) {
        if let Some(session_id) = self.active_session_id.clone() {
            let state = self.state.read(cx).clone();
            let j_id = job_id.to_string();
            tokio::spawn(async move {
                let _ = state.kill_job(&session_id, &j_id).await;
            });
            cx.notify();
        }
    }

    pub fn add_attachment_dialog(&mut self, cx: &mut Context<Self>) {
        let paths_receiver = cx.prompt_for_paths(gpui::PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some("选择附加文件或图片".into()),
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = paths_receiver.await else {
                return;
            };
            let _ = this.update(cx, |view, cx| {
                for path in paths {
                    let path_str = path.to_string_lossy().to_string();
                    if !view.attachments.contains(&path_str) {
                        view.attachments.push(path_str);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn remove_attachment(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.attachments.len() {
            self.attachments.remove(index);
            cx.notify();
        }
    }

    pub fn export_session_markdown_action(&mut self, cx: &mut Context<Self>) {
        if let Some(session_id) = &self.active_session_id {
            let state = self.state.read(cx).clone();
            let sid = session_id.clone();
            let state_arc = state.clone();
            cx.spawn(async move |this, cx| {
                let sessions = state_arc.sessions.read().await;
                if let Some(session) = sessions.get(&sid) {
                    let md = dsh_core::AppState::export_session_markdown(session);
                    let filename = format!("session-{}.md", session.id);
                    let target_path =
                        std::path::PathBuf::from(&session.workspace_path).join(&filename);
                    let _ = std::fs::write(&target_path, &md);
                    drop(sessions);
                    let _ = this.update(cx, |view, cx| {
                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(md));
                        view.session_log_lines
                            .push(format!("已导出会话记录至: {}", target_path.display()));
                        cx.notify();
                    });
                }
            })
            .detach();
        }
    }

    pub fn export_session_json_action(&mut self, cx: &mut Context<Self>) {
        if let Some(session_id) = &self.active_session_id {
            let state = self.state.read(cx).clone();
            let sid = session_id.clone();
            let state_arc = state.clone();
            cx.spawn(async move |this, cx| {
                let sessions = state_arc.sessions.read().await;
                if let Some(session) = sessions.get(&sid) {
                    if let Ok(json) = dsh_core::AppState::export_session_json(session) {
                        let filename = format!("session-{}.json", session.id);
                        let target_path =
                            std::path::PathBuf::from(&session.workspace_path).join(&filename);
                        let _ = std::fs::write(&target_path, &json);
                        drop(sessions);
                        let _ = this.update(cx, |view, cx| {
                            cx.write_to_clipboard(gpui::ClipboardItem::new_string(json));
                            view.session_log_lines
                                .push(format!("已导出会话记录至: {}", target_path.display()));
                            cx.notify();
                        });
                    }
                }
            })
            .detach();
        }
    }

    pub fn exit_plan_mode(&mut self, cx: &mut Context<Self>) {
        if let Some(session_id) = self.active_session_id.clone() {
            let state = self.state.read(cx).clone();
            self.plan_active = false;
            tokio::spawn(async move {
                let _ = state.toggle_plan_mode(&session_id, false).await;
            });
            cx.notify();
        }
    }

    pub fn approve_plan(&mut self, cx: &mut Context<Self>) {
        if let Some(session_id) = self.active_session_id.clone() {
            let state = self.state.read(cx).clone();
            self.plan_markdown.clear();
            tokio::spawn(async move {
                let _ = state.toggle_plan_mode(&session_id, false).await;
            });
            cx.notify();
        }
    }

    pub fn toggle_question_option(&mut self, option: &str, multi: bool, cx: &mut Context<Self>) {
        if multi {
            if let Some(pos) = self
                .selected_question_options
                .iter()
                .position(|o| o == option)
            {
                self.selected_question_options.remove(pos);
            } else {
                self.selected_question_options.push(option.to_string());
            }
        } else {
            self.selected_question_options = vec![option.to_string()];
        }
        cx.notify();
    }

    pub fn submit_question_answer(&mut self, question_id: &str, cx: &mut Context<Self>) {
        let selected = self.selected_question_options.clone();
        if let Some(session_id) = self.active_session_id.clone() {
            let state = self.state.read(cx).clone();
            let q_id = question_id.to_string();
            tokio::spawn(async move {
                let _ = state
                    .answer_question(&session_id, &q_id, selected, None)
                    .await;
            });
            self.selected_question_options.clear();
            self.pending_question = None;
            cx.notify();
        }
    }

    pub fn reset(&mut self, cx: &mut Context<Self>) {
        self.has_messages = false;
        self.has_active_session = false;
        self.active_prompt.clear();
        self.active_session_title.clear();
        self.active_turns = 0;
        self.active_steps = 0;
        self.active_tool_ms = 0;
        self.active_llm_ms = 0;
        self.active_ttft_ms = 0;
        self.active_input_tokens = 0;
        self.active_output_tokens = 0;
        self.active_cache_hit = None;
        self.streaming_text.clear();
        self.text_input.update(cx, |input, cx| {
            input.clear(cx);
        });
        cx.notify();
    }

    pub fn submit_current_input(&mut self, cx: &mut Context<Self>) {
        let text = self.text_input.read(cx).text().trim().to_string();
        if text.is_empty() {
            return;
        }
        let prompt = text;

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

    fn toggle_permission(&mut self, cx: &mut Context<Self>) {
        self.permission_open = !self.permission_open;
        self.model_open = false;
        cx.notify();
    }

    fn toggle_model(&mut self, cx: &mut Context<Self>) {
        self.model_open = !self.model_open;
        self.permission_open = false;
        cx.notify();
    }

    fn toggle_plus_menu(&mut self, cx: &mut Context<Self>) {
        self.plus_menu_open = !self.plus_menu_open;
        self.permission_open = false;
        self.model_open = false;
        cx.notify();
    }

    fn insert_command(&mut self, command: &str, cx: &mut Context<Self>) {
        self.text_input.update(cx, |input, cx| {
            input.set_text(command, cx);
        });
        self.plus_menu_open = false;
        cx.notify();
    }

    fn set_permission(&mut self, value: &str, cx: &mut Context<Self>) {
        self.permission_mode = value.to_string();
        self.permission_open = false;
        let config_value = match value {
            "Read Only" => "read-only",
            "Workspace Write" => "workspace-write",
            _ => "full-access",
        };
        let state = self.state.read(cx).clone();
        let config_value = config_value.to_string();
        tokio::spawn(async move {
            let mut config = state.config.write().await;
            config.ui.permission_mode = config_value;
            let _ = config.save(&AppPaths::data_dir());
        });
        cx.notify();
    }

    fn set_session_view(&mut self, view: SessionView, cx: &mut Context<Self>) {
        self.active_view = view;
        self.session_log_open = false;
        self.permission_open = false;
        self.model_open = false;
        cx.notify();
    }

    fn toggle_session_log(&mut self, cx: &mut Context<Self>) {
        self.session_log_open = !self.session_log_open;
        cx.notify();
    }

    fn toggle_trace_duration(&mut self, cx: &mut Context<Self>) {
        self.trace_actual_duration = !self.trace_actual_duration;
        cx.notify();
    }

    fn toggle_trace_all(&mut self, cx: &mut Context<Self>) {
        self.trace_all_collapsed = !self.trace_all_collapsed;
        self.trace_collapsed_turns.clear();
        cx.notify();
    }

    fn toggle_trace_turn(&mut self, turn: usize, cx: &mut Context<Self>) {
        if !self.trace_collapsed_turns.insert(turn) {
            self.trace_collapsed_turns.remove(&turn);
        }
        cx.notify();
    }

    fn set_model(&mut self, value: &str, cx: &mut Context<Self>) {
        self.model_name = value.to_string();
        self.model_open = false;
        let state = self.state.read(cx).clone();
        let model_name = value.to_string();
        tokio::spawn(async move {
            let mut config = state.config.write().await;
            config.model.model_name = model_name;
            let _ = config.save(&AppPaths::data_dir());
        });
        cx.notify();
    }

    fn render_markdown_block(
        &self,
        block: MarkdownBlock,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
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
                let code_content = code.clone();
                let handle_copy_code = cx.listener(move |_this, _, _, cx| {
                    cx.write_to_clipboard(code_content.clone().into());
                });

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
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded(px(4.0))
                                    .hover(|s| s.bg(rgb(0xe5e7eb)))
                                    .cursor_pointer()
                                    .on_mouse_down(MouseButton::Left, handle_copy_code)
                                    .child(icons::copy(12.0, rgb(0x61666b)))
                                    .child(div().text_xs().text_color(rgb(0x61666b)).child("复制")),
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
                    .child(self.render_goal_bar(cx))
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
                            .child(self.render_attachments_bar(cx))
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
                                            .child(self.render_plus_button(cx))
                                            .child(
                                                div()
                                                    .size(px(28.0))
                                                    .rounded_full()
                                                    .bg(rgb(0xf1f3f5))
                                                    .hover(|s| s.bg(rgb(0xe9edf2)))
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .cursor_pointer()
                                                    .on_mouse_down(
                                                        gpui::MouseButton::Left,
                                                        cx.listener(|this, _, _, cx| {
                                                            this.add_attachment_dialog(cx)
                                                        }),
                                                    )
                                                    .child(icons::paperclip(14.0, rgb(0x61666b))),
                                            )
                                            .child(self.render_access_selector(cx))
                                            .child(self.render_jobs_action(cx))
                                            .when(self.plan_active, |this| {
                                                this.child(self.render_plan_chip(cx))
                                            }),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_3()
                                            .child(self.render_model_selector(cx))
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

    fn render_session_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let title = if self.active_session_title.is_empty() {
            "新会话"
        } else {
            &self.active_session_title
        };
        let active_view = self.active_view;
        let handle_chat =
            cx.listener(|this, _, _, cx| this.set_session_view(SessionView::Chat, cx));
        let handle_trace =
            cx.listener(|this, _, _, cx| this.set_session_view(SessionView::Trace, cx));
        let handle_log = cx.listener(|this, _, _, cx| this.toggle_session_log(cx));

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
                            .child(self.preset_selector.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .px_2p5()
                                    .py_1()
                                    .rounded(px(8.0))
                                    .border_1()
                                    .border_color(rgb(0xe1e5eb))
                                    .text_xs()
                                    .text_color(rgb(0x61666b))
                                    .hover(|s| s.bg(rgb(0xf1f3f5)))
                                    .cursor_pointer()
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.export_session_markdown_action(cx)
                                        }),
                                    )
                                    .child(icons::download(12.0, rgb(0x61666b)))
                                    .child("导出 Markdown"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .px_2p5()
                                    .py_1()
                                    .rounded(px(8.0))
                                    .border_1()
                                    .border_color(rgb(0xe1e5eb))
                                    .text_xs()
                                    .text_color(rgb(0x61666b))
                                    .hover(|s| s.bg(rgb(0xf1f3f5)))
                                    .cursor_pointer()
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.export_session_json_action(cx)
                                        }),
                                    )
                                    .child(icons::download(12.0, rgb(0x61666b)))
                                    .child("导出 JSON"),
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
                                    .hover(|s| s.bg(rgb(0xf1f3f5)))
                                    .cursor_pointer()
                                    .on_mouse_down(gpui::MouseButton::Left, handle_log)
                                    .child(if self.session_log_open {
                                        "Session log ↑"
                                    } else {
                                        "Session log ↓"
                                    }),
                            ),
                    ),
            )
            .when(self.session_log_open, |this| {
                this.child(
                    div()
                        .px_4()
                        .py_2()
                        .bg(rgb(0xf9fafb))
                        .border_b_1()
                        .border_color(rgb(0xe5e7eb))
                        .flex()
                        .flex_col()
                        .gap_1()
                        .text_xs()
                        .text_color(rgb(0x61666b))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(format!("Session log · {} 行", self.session_log_lines.len()))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_1()
                                                .cursor_pointer()
                                                .hover(|s| s.text_color(rgb(0x3964fe)))
                                                .on_mouse_down(
                                                    gpui::MouseButton::Left,
                                                    cx.listener(|this, _, _, cx| {
                                                        this.export_session_markdown_action(cx)
                                                    }),
                                                )
                                                .child(icons::download(11.0, rgb(0x61666b)))
                                                .child("保存 Markdown"),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_1()
                                                .cursor_pointer()
                                                .hover(|s| s.text_color(rgb(0x3964fe)))
                                                .on_mouse_down(
                                                    gpui::MouseButton::Left,
                                                    cx.listener(|this, _, _, cx| {
                                                        this.export_session_json_action(cx)
                                                    }),
                                                )
                                                .child(icons::download(11.0, rgb(0x61666b)))
                                                .child("保存 JSON"),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .id("session-log-content")
                                .max_h(px(180.0))
                                .overflow_y_scroll()
                                .track_scroll(&self.session_log_scroll_handle)
                                .children(if self.session_log_lines.is_empty() {
                                    vec![div()
                                        .text_xs()
                                        .text_color(rgb(0x81858c))
                                        .child("当前会话尚无终端日志")
                                        .into_any_element()]
                                } else {
                                    session_log_display_lines(&self.session_log_lines)
                                        .into_iter()
                                        .map(|line| {
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x3f454d))
                                                .child(line)
                                                .into_any_element()
                                        })
                                        .collect()
                                }),
                        ),
                )
            })
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
                            .border_color(if active_view == SessionView::Chat {
                                rgb(0x3964fe)
                            } else {
                                rgb(0xffffff)
                            })
                            .text_xs()
                            .text_color(if active_view == SessionView::Chat {
                                rgb(0x3964fe)
                            } else {
                                rgb(0x81858c)
                            })
                            .cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, handle_chat)
                            .child("对话"),
                    )
                    .child(
                        div()
                            .h_full()
                            .flex()
                            .items_center()
                            .border_b_2()
                            .border_color(if active_view == SessionView::Trace {
                                rgb(0x3964fe)
                            } else {
                                rgb(0xffffff)
                            })
                            .text_xs()
                            .text_color(if active_view == SessionView::Trace {
                                rgb(0x3964fe)
                            } else {
                                rgb(0x81858c)
                            })
                            .cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, handle_trace)
                            .child("轨迹"),
                    ),
            )
    }

    fn render_goal_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let goal = match &self.goal {
            Some(g) if g.phase != dsh_protocol::GoalPhase::Complete => g.clone(),
            _ => return div().into_any_element(),
        };

        let is_editing = self.goal_editing;
        let handle_edit_start = cx.listener(|this, _, _, cx| this.start_edit_goal(cx));
        let handle_edit_save = cx.listener(|this, _, _, cx| this.save_edit_goal(cx));
        let handle_edit_cancel = cx.listener(|this, _, _, cx| this.cancel_edit_goal(cx));
        let handle_pause_toggle = cx.listener(|this, _, _, cx| this.toggle_goal_pause(cx));
        let handle_clear = cx.listener(|this, _, _, cx| this.clear_goal(cx));

        let (bg, border, text_color) = goal_phase_color(goal.phase);
        let phase_text = goal_phase_label(goal.phase);

        div()
            .w_full()
            .max_w(px(776.0))
            .mb_2()
            .px_3()
            .py_1p5()
            .rounded_xl()
            .bg(bg)
            .border_1()
            .border_color(border)
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .flex_1()
                    .overflow_hidden()
                    .child(icons::target(15.0, text_color))
                    .child(
                        div()
                            .px_1p5()
                            .py_0p5()
                            .rounded_md()
                            .bg(rgba(0xffffff88))
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(text_color)
                            .child(format!("目标 ({})", phase_text)),
                    )
                    .child(if is_editing {
                        div()
                            .flex_1()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x1f2937))
                                    .child(format!("编辑中: {}", self.goal_draft)),
                            )
                            .into_any_element()
                    } else {
                        div()
                            .text_xs()
                            .text_color(rgb(0x1f2937))
                            .overflow_hidden()
                            .child(goal.objective.clone())
                            .into_any_element()
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .when(is_editing, |this| {
                        this.child(
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded_md()
                                .bg(rgb(0x3964fe))
                                .text_xs()
                                .text_color(rgb(0xffffff))
                                .cursor_pointer()
                                .on_mouse_down(gpui::MouseButton::Left, handle_edit_save)
                                .child("保存"),
                        )
                        .child(
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded_md()
                                .bg(rgb(0xe5e7eb))
                                .text_xs()
                                .text_color(rgb(0x374151))
                                .cursor_pointer()
                                .on_mouse_down(gpui::MouseButton::Left, handle_edit_cancel)
                                .child("取消"),
                        )
                    })
                    .when(!is_editing, |this| {
                        this.child(
                            div()
                                .p_1()
                                .rounded_md()
                                .hover(|s| s.bg(rgba(0x00000010)))
                                .cursor_pointer()
                                .on_mouse_down(gpui::MouseButton::Left, handle_pause_toggle)
                                .child(if goal.phase == dsh_protocol::GoalPhase::Paused {
                                    icons::play(13.0, rgb(0x4b5563)).into_any_element()
                                } else {
                                    icons::pause(13.0, rgb(0x4b5563)).into_any_element()
                                }),
                        )
                        .child(
                            div()
                                .p_1()
                                .rounded_md()
                                .hover(|s| s.bg(rgba(0x00000010)))
                                .cursor_pointer()
                                .on_mouse_down(gpui::MouseButton::Left, handle_edit_start)
                                .child(icons::wrench(13.0, rgb(0x4b5563))),
                        )
                        .child(
                            div()
                                .p_1()
                                .rounded_md()
                                .hover(|s| s.bg(rgba(0x00000010)))
                                .cursor_pointer()
                                .on_mouse_down(gpui::MouseButton::Left, handle_clear)
                                .child(icons::close(13.0, rgb(0x6b7280))),
                        )
                    }),
            )
            .into_any_element()
    }

    fn render_jobs_action(&self, cx: &mut Context<Self>) -> impl IntoElement {
        if self.jobs.is_empty() {
            return div().into_any_element();
        }

        let handle_toggle = cx.listener(|this, _, _, cx| this.toggle_jobs_menu(cx));
        let is_open = self.jobs_menu_open;

        let has_running = self
            .jobs
            .iter()
            .any(|j| j.status == dsh_protocol::JobStatus::Running);
        let has_warning = self.jobs.iter().any(|j| {
            matches!(
                j.status,
                dsh_protocol::JobStatus::Stopping | dsh_protocol::JobStatus::Killed
            )
        });
        let has_failed = self
            .jobs
            .iter()
            .any(|j| j.status == dsh_protocol::JobStatus::Failed);

        let dot_color = if has_running {
            rgba(0x16803cFF)
        } else if has_failed {
            rgba(0xb42318FF)
        } else if has_warning {
            rgba(0xb7791fFF)
        } else {
            rgba(0x81858cFF)
        };

        div()
            .relative()
            .child(
                div()
                    .h(px(28.0))
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .px_2()
                    .rounded_md()
                    .hover(|s| s.bg(rgb(0xf1f3f5)))
                    .cursor_pointer()
                    .on_mouse_down(gpui::MouseButton::Left, handle_toggle)
                    .child(div().size(px(6.0)).rounded_full().bg(dot_color))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x3f454d))
                            .child(format!("{} 项任务", self.jobs.len())),
                    )
                    .child(icons::chevron_down(12.0, rgb(0x81858c))),
            )
            .when(is_open, |this| {
                this.child(
                    div()
                        .absolute()
                        .bottom(px(34.0))
                        .left(px(0.0))
                        .w(px(336.0))
                        .p_2()
                        .rounded_xl()
                        .bg(rgb(0xffffff))
                        .border_1()
                        .border_color(rgb(0xe1e5eb))
                        .shadow_lg()
                        .flex()
                        .flex_col()
                        .gap_1p5()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .pb_1()
                                .border_b_1()
                                .border_color(rgb(0xf1f3f5))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(rgb(0x374151))
                                        .child("后台任务列表"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x9ca3af))
                                        .child(format!("共 {} 项", self.jobs.len())),
                                ),
                        )
                        .children(self.jobs.iter().map(|job| {
                            let status_dot = job_status_dot_color(job.status);
                            let status_text = job_status_label(job.status);
                            let dur_str = format_duration_metric(job.duration_ms.unwrap_or(0));
                            let job_id = job.id.clone();
                            let is_running = job.status == dsh_protocol::JobStatus::Running;
                            let handle_kill = cx.listener(move |this, _, _, cx| {
                                this.kill_job(&job_id, cx);
                            });

                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .p_1p5()
                                .rounded_lg()
                                .hover(|s| s.bg(rgb(0xf9fafb)))
                                .gap_2()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_1p5()
                                        .flex_1()
                                        .overflow_hidden()
                                        .child(div().size(px(6.0)).rounded_full().bg(status_dot))
                                        .child(
                                            div()
                                                .px_1()
                                                .rounded(px(3.0))
                                                .bg(rgb(0xf3f4f6))
                                                .text_xs()
                                                .text_color(rgb(0x6b7280))
                                                .child(job.kind.clone()),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgb(0x1f2937))
                                                .overflow_hidden()
                                                .child(job.label.clone()),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_1p5()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x9ca3af))
                                                .child(status_text),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x6b7280))
                                                .child(dur_str),
                                        )
                                        .when(is_running, |this| {
                                            this.child(
                                                div()
                                                    .size(px(18.0))
                                                    .rounded_full()
                                                    .hover(|s| s.bg(rgb(0xfee2e2)))
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .cursor_pointer()
                                                    .on_mouse_down(
                                                        gpui::MouseButton::Left,
                                                        handle_kill,
                                                    )
                                                    .child(icons::close(10.0, rgb(0xef4444))),
                                            )
                                        }),
                                )
                        })),
                )
            })
            .into_any_element()
    }

    fn render_attachments_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        if self.attachments.is_empty() {
            return div().into_any_element();
        }

        div()
            .w_full()
            .flex()
            .flex_wrap()
            .gap_1p5()
            .pb_2()
            .children(self.attachments.iter().enumerate().map(|(idx, path)| {
                let filename = std::path::Path::new(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(path.as_str())
                    .to_string();
                let handle_remove = cx.listener(move |this, _, _, cx| {
                    this.remove_attachment(idx, cx);
                });

                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .px_2()
                    .py_1()
                    .rounded_lg()
                    .bg(rgb(0xf1f3f5))
                    .border_1()
                    .border_color(rgb(0xe1e5eb))
                    .child(icons::paperclip(12.0, rgb(0x61666b)))
                    .child(div().text_xs().text_color(rgb(0x374151)).child(filename))
                    .child(
                        div()
                            .size(px(14.0))
                            .rounded_full()
                            .hover(|s| s.bg(rgb(0xe5e7eb)))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, handle_remove)
                            .child(icons::close(8.0, rgb(0x6b7280))),
                    )
            }))
            .into_any_element()
    }

    fn render_plan_chip(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let handle_exit = cx.listener(|this, _, _, cx| this.exit_plan_mode(cx));
        div()
            .flex()
            .items_center()
            .gap_1()
            .px_2()
            .py_0p5()
            .rounded_full()
            .bg(rgb(0xfef3c7))
            .border_1()
            .border_color(rgb(0xfde68a))
            .cursor_pointer()
            .on_mouse_down(gpui::MouseButton::Left, handle_exit)
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(0xb45309))
                    .child("Plan"),
            )
            .child(icons::close(10.0, rgb(0xb45309)))
    }
    fn render_access_selector(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let current = self.permission_mode.clone();
        let is_open = self.permission_open;
        let handle_toggle = cx.listener(|this, _, _, cx| this.toggle_permission(cx));
        let handle_full = cx.listener(|this, _, _, cx| this.set_permission("Full access", cx));
        let handle_workspace =
            cx.listener(|this, _, _, cx| this.set_permission("Workspace Write", cx));
        let handle_read = cx.listener(|this, _, _, cx| this.set_permission("Read Only", cx));

        div()
            .relative()
            .child(
                div()
                    .h(px(28.0))
                    .flex()
                    .items_center()
                    .gap_1()
                    .px_1p5()
                    .rounded_md()
                    .hover(|s| s.bg(rgb(0xf1f3f5)))
                    .cursor_pointer()
                    .on_mouse_down(gpui::MouseButton::Left, handle_toggle)
                    .child(icons::check(14.0, rgb(0x61666b)))
                    .child(div().text_xs().text_color(rgb(0x3f454d)).child(current))
                    .child(icons::chevron_down(12.0, rgb(0x81858c))),
            )
            .when(is_open, |this| {
                this.child(
                    div()
                        .absolute()
                        .bottom(px(34.0))
                        .left(px(0.0))
                        .w(px(190.0))
                        .p_1()
                        .rounded_lg()
                        .bg(rgb(0xffffff))
                        .border_1()
                        .border_color(rgb(0xe1e5eb))
                        .shadow_lg()
                        .flex()
                        .flex_col()
                        .children([
                            menu_choice(
                                "Full access",
                                self.permission_mode == "Full access",
                                handle_full,
                            )
                            .into_any_element(),
                            menu_choice(
                                "Workspace Write",
                                self.permission_mode == "Workspace Write",
                                handle_workspace,
                            )
                            .into_any_element(),
                            menu_choice(
                                "Read Only",
                                self.permission_mode == "Read Only",
                                handle_read,
                            )
                            .into_any_element(),
                        ]),
                )
            })
    }

    fn render_plus_menu(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let handle_help = cx.listener(|this, _, _, cx| this.insert_command("/help", cx));
        let handle_model = cx.listener(|this, _, _, cx| this.insert_command("/model", cx));
        let handle_clear = cx.listener(|this, _, _, cx| this.insert_command("/clear", cx));
        div()
            .absolute()
            .bottom(px(36.0))
            .left(px(0.0))
            .w(px(180.0))
            .p_1()
            .rounded_lg()
            .bg(rgb(0xffffff))
            .border_1()
            .border_color(rgb(0xe1e5eb))
            .shadow_lg()
            .flex()
            .flex_col()
            .child(menu_choice("/help", false, handle_help))
            .child(menu_choice("/model", false, handle_model))
            .child(menu_choice("/clear", false, handle_clear))
    }

    fn render_plus_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let handle_plus = cx.listener(|this, _, _, cx| this.toggle_plus_menu(cx));
        div()
            .relative()
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
                    .on_mouse_down(gpui::MouseButton::Left, handle_plus)
                    .child(icons::plus(14.0, rgb(0x61666b))),
            )
            .when(self.plus_menu_open, |this| {
                this.child(deferred(self.render_plus_menu(cx)))
            })
    }

    fn render_model_selector(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let current = self.model_name.clone();
        let is_open = self.model_open;
        let handle_toggle = cx.listener(|this, _, _, cx| this.toggle_model(cx));
        let groups = provider_groups(&current);

        div()
            .relative()
            .child(
                div()
                    .h(px(28.0))
                    .flex()
                    .items_center()
                    .gap_1()
                    .px_1p5()
                    .rounded_md()
                    .hover(|s| s.bg(rgb(0xf1f3f5)))
                    .cursor_pointer()
                    .on_mouse_down(gpui::MouseButton::Left, handle_toggle)
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x3f454d))
                            .child(current.clone()),
                    )
                    .child(icons::chevron_down(12.0, rgb(0x81858c))),
            )
            .when(is_open, |this| {
                this.child(
                    div()
                        .absolute()
                        .bottom(px(34.0))
                        .right(px(0.0))
                        .w(px(240.0))
                        .max_h(px(320.0))
                        .id("model-selector-menu")
                        .overflow_y_scroll()
                        .p_1()
                        .rounded_lg()
                        .bg(rgb(0xffffff))
                        .border_1()
                        .border_color(rgb(0xe1e5eb))
                        .shadow_lg()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .children(groups.into_iter().map(|group| {
                            div()
                                .flex()
                                .flex_col()
                                .gap_0p5()
                                .child(
                                    div()
                                        .px_2()
                                        .pt_1()
                                        .pb_0p5()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0x81858c))
                                        .child(group.provider),
                                )
                                .children(group.models.into_iter().map(|model| {
                                    let selected = current == model;
                                    let model_for_handler = model.clone();
                                    let handle = cx.listener(move |this, _, _, cx| {
                                        this.set_model(&model_for_handler, cx);
                                    });
                                    menu_choice(&model, selected, handle).into_any_element()
                                }))
                                .into_any_element()
                        })),
                )
            })
    }

    fn render_stats_line(&self) -> impl IntoElement {
        let line = format_stats_line(
            self.active_turns,
            self.active_steps,
            self.active_llm_ms,
            self.active_tool_ms,
            self.active_ttft_ms,
            self.active_output_tokens,
            self.active_llm_ms,
            self.active_cache_hit,
            self.active_input_tokens,
            self.active_output_tokens,
        );

        if line.is_empty() {
            return div().into_any_element();
        }

        div()
            .max_w(px(776.0))
            .w_full()
            .pt_1()
            .px_2()
            .text_xs()
            .text_color(rgb(0x81858c))
            .child(line)
            .into_any_element()
    }

    fn render_trace(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let query = self.trace_search_input.read(cx).text().to_lowercase();
        let mut turns = Vec::new();
        for entry in &self.trace_entries {
            if (!query.is_empty()
                && !format!("{} {} {}", entry.kind, entry.title, entry.detail)
                    .to_lowercase()
                    .contains(&query))
                || turns.contains(&entry.turn)
            {
                continue;
            }
            turns.push(entry.turn);
        }

        let handle_duration = cx.listener(|this, _, _, cx| this.toggle_trace_duration(cx));
        let handle_collapse = cx.listener(|this, _, _, cx| this.toggle_trace_all(cx));
        let mut rows = Vec::new();
        for turn in turns {
            let entries = self
                .trace_entries
                .iter()
                .filter(|entry| {
                    entry.turn == turn
                        && (query.is_empty()
                            || format!("{} {} {}", entry.kind, entry.title, entry.detail)
                                .to_lowercase()
                                .contains(&query))
                })
                .cloned()
                .collect::<Vec<_>>();
            let collapsed = self.trace_all_collapsed || self.trace_collapsed_turns.contains(&turn);
            let handle_turn = cx.listener(move |this, _, _, cx| this.toggle_trace_turn(turn, cx));
            rows.push(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .py_2()
                    .bg(rgb(0xf5f6f8))
                    .border_b_1()
                    .border_color(rgb(0xe5e7eb))
                    .hover(|s| s.bg(rgb(0xf1f3f5)))
                    .cursor_pointer()
                    .on_mouse_down(gpui::MouseButton::Left, handle_turn)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(if collapsed { "▸" } else { "▾" })
                            .child(format!("回合 {}", turn)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x81858c))
                            .child(format!("{} 步", entries.len())),
                    )
                    .into_any_element(),
            );

            if !collapsed {
                for entry in entries {
                    let drawer_entity = self.details_drawer.clone();
                    let title = entry.title.clone();
                    let args = entry.args.clone();
                    let output = entry.output.clone();
                    let duration = entry.duration_ms;
                    let handle_entry = cx.listener(move |_this, _, _, cx| {
                        drawer_entity.update(cx, |drawer, cx| {
                            drawer.open_tool(&title, duration, &args, &output, cx);
                        });
                    });
                    let duration_label = if self.trace_actual_duration {
                        format!("{}ms", entry.duration_ms)
                    } else {
                        "--".to_string()
                    };
                    rows.push(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .px_3()
                            .py_2()
                            .border_b_1()
                            .border_color(rgb(0xf0f1f3))
                            .hover(|s| s.bg(rgb(0xf9fafb)))
                            .cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, handle_entry)
                            .child(
                                div()
                                    .w(px(64.0))
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(if entry.kind == "TOOL" {
                                        rgb(0x3964fe)
                                    } else {
                                        rgb(0x81858c)
                                    })
                                    .child(entry.kind),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(0.0))
                                    .overflow_hidden()
                                    .text_xs()
                                    .text_color(rgb(0x3f454d))
                                    .text_ellipsis()
                                    .child(entry.title),
                            )
                            .when_some(entry.tool_status, |this, status| {
                                this.child(
                                    div()
                                        .w(px(42.0))
                                        .text_xs()
                                        .text_color(tool_status_color(status))
                                        .child(tool_status_label(status)),
                                )
                            })
                            .child(
                                div()
                                    .w(px(50.0))
                                    .text_xs()
                                    .text_color(rgb(0x81858c))
                                    .child(duration_label),
                            )
                            .into_any_element(),
                    );
                }
            }
        }

        div()
            .flex_1()
            .overflow_hidden()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .px_4()
                    .py_2()
                    .border_b_1()
                    .border_color(rgb(0xe5e7eb))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded(px(6.0))
                                    .bg(if self.trace_actual_duration {
                                        rgb(0xe8f0ff)
                                    } else {
                                        rgb(0xf1f3f5)
                                    })
                                    .text_xs()
                                    .text_color(rgb(0x3f454d))
                                    .cursor_pointer()
                                    .on_mouse_down(gpui::MouseButton::Left, handle_duration)
                                    .child(if self.trace_actual_duration {
                                        "实际时长"
                                    } else {
                                        "等宽时长"
                                    }),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded(px(6.0))
                                    .bg(rgb(0xf1f3f5))
                                    .text_xs()
                                    .text_color(rgb(0x3f454d))
                                    .cursor_pointer()
                                    .on_mouse_down(gpui::MouseButton::Left, handle_collapse)
                                    .child(if self.trace_all_collapsed {
                                        "展开回合"
                                    } else {
                                        "收起回合"
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .w(px(180.0))
                            .h(px(30.0))
                            .px_2()
                            .border_1()
                            .border_color(rgb(0xe1e5eb))
                            .rounded(px(6.0))
                            .child(self.trace_search_input.clone()),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .children(rows),
            )
    }

    fn render_plan_review_card(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let handle_approve = cx.listener(|this, _, _, cx| this.approve_plan(cx));
        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_3p5()
            .rounded_xl()
            .bg(rgb(0xfffbeb))
            .border_1()
            .border_color(rgb(0xfde68a))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().size(px(8.0)).rounded_full().bg(rgb(0xd97706)))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xb45309))
                            .child("Plan 审核"),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x78350f))
                    .child(self.plan_markdown.clone()),
            )
            .child(
                div().flex().items_center().justify_end().gap_2().child(
                    div()
                        .px_2p5()
                        .py_1()
                        .rounded_md()
                        .bg(rgb(0xd97706))
                        .hover(|s| s.bg(rgb(0xb45309)))
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0xffffff))
                        .cursor_pointer()
                        .on_mouse_down(gpui::MouseButton::Left, handle_approve)
                        .child("批准计划"),
                ),
            )
    }

    fn render_question_card(
        &self,
        q: &dsh_protocol::QuestionItem,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let q_id = q.id.clone();
        let multi = q.multi_select;
        let handle_submit = cx.listener({
            let q_id = q_id.clone();
            move |this, _, _, cx| this.submit_question_answer(&q_id, cx)
        });

        div()
            .flex()
            .flex_col()
            .gap_2p5()
            .p_3p5()
            .rounded_xl()
            .bg(rgb(0xf0f9ff))
            .border_1()
            .border_color(rgb(0xbae6fd))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().size(px(8.0)).rounded_full().bg(rgb(0x0284c7)))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x0369a1))
                            .child("请选择方案"),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(0x0f172a))
                    .child(q.prompt.clone()),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .children(q.options.iter().map(|opt| {
                        let opt_str = opt.clone();
                        let is_selected = self.selected_question_options.contains(&opt_str);
                        let handle_toggle = cx.listener({
                            let opt_str = opt_str.clone();
                            move |this, _, _, cx| this.toggle_question_option(&opt_str, multi, cx)
                        });
                        div()
                            .px_2p5()
                            .py_1()
                            .rounded_lg()
                            .border_1()
                            .border_color(if is_selected {
                                rgb(0x0284c7)
                            } else {
                                rgb(0xe2e8f0)
                            })
                            .bg(if is_selected {
                                rgb(0xe0f2fe)
                            } else {
                                rgb(0xffffff)
                            })
                            .text_xs()
                            .text_color(if is_selected {
                                rgb(0x0369a1)
                            } else {
                                rgb(0x334155)
                            })
                            .cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, handle_toggle)
                            .child(opt.clone())
                    })),
            )
            .child(
                div().flex().justify_end().pt_1().child(
                    div()
                        .px_3()
                        .py_1()
                        .rounded_md()
                        .bg(rgb(0x0284c7))
                        .hover(|s| s.bg(rgb(0x0369a1)))
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0xffffff))
                        .cursor_pointer()
                        .on_mouse_down(gpui::MouseButton::Left, handle_submit)
                        .child("确认选择"),
                ),
            )
    }
    fn render_active_messages(&self, cx: &mut Context<Self>) -> impl IntoElement {
        if self.active_view == SessionView::Trace {
            return div()
                .flex_1()
                .overflow_hidden()
                .flex()
                .flex_col()
                .child(self.render_session_header(cx))
                .child(self.render_trace(cx));
        }
        let user_prompt = if self.active_prompt.is_empty() {
            "暂无消息"
        } else {
            &self.active_prompt
        };

        let message_rows = if self.conversation_messages.is_empty() {
            vec![div()
                .text_sm()
                .text_color(rgb(0x81858c))
                .child(user_prompt.to_string())
                .into_any_element()]
        } else {
            self.conversation_messages
                .iter()
                .map(|message| match message.sender {
                    dsh_core::MessageSender::User => div()
                        .flex()
                        .justify_end()
                        .child(
                            div()
                                .max_w_3_4()
                                .px_4()
                                .py_2p5()
                                .rounded_2xl()
                                .bg(rgb(0xe8f0ff))
                                .text_sm()
                                .text_color(rgb(0x0f1115))
                                .child(message.content.clone()),
                        )
                        .into_any_element(),
                    dsh_core::MessageSender::Assistant => div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .max_w_full()
                        .p_4()
                        .children(message.tool_calls.iter().map(|tool| {
                            let drawer_entity = self.details_drawer.clone();
                            let title = tool.tool_name.clone();
                            let args = tool.input.to_string();
                            let output = tool
                                .output
                                .as_ref()
                                .map(ToString::to_string)
                                .unwrap_or_else(|| "等待工具返回".into());
                            let duration = tool.duration_ms;
                            let status = tool.status;
                            let status_label = tool_status_label(status);
                            let status_color = tool_status_color(status);
                            let handle_tool_click = cx.listener(move |_this, _, _, cx| {
                                drawer_entity.update(cx, |drawer, cx| {
                                    drawer.open_tool(&title, duration, &args, &output, cx);
                                });
                            });
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .px_3()
                                .py_1p5()
                                .rounded_md()
                                .bg(rgb(0xf5f6f8))
                                .border_1()
                                .border_color(status_color)
                                .hover(|s| s.bg(rgb(0xf1f3f5)).border_color(rgb(0x3964fe)))
                                .cursor_pointer()
                                .on_mouse_down(gpui::MouseButton::Left, handle_tool_click)
                                .child(icons::agent_preset(14.0, status_color))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x61666b))
                                        .child(format!("{} ({})", tool.tool_name, status_label)),
                                )
                                .child(div().text_xs().text_color(status_color).child(
                                    if status == ToolStatus::Running {
                                        "--".to_string()
                                    } else {
                                        format!("{}ms", tool.duration_ms)
                                    },
                                ))
                                .into_any_element()
                        }))
                        .children(
                            StreamingMarkdownParser::parse_markdown(&message.content)
                                .into_iter()
                                .map(|block| self.render_markdown_block(block, cx)),
                        )
                        .child({
                            let message_content = message.content.clone();
                            let handle_copy_msg = cx.listener(move |_this, _, _, cx| {
                                cx.write_to_clipboard(message_content.clone().into());
                            });
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .pt_2()
                                .child(
                                    div()
                                        .size(px(24.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .rounded(px(6.0))
                                        .hover(|s| s.bg(rgb(0xf1f3f5)))
                                        .cursor_pointer()
                                        .on_mouse_down(gpui::MouseButton::Left, handle_copy_msg)
                                        .child(icons::copy(13.0, rgb(0x81858c))),
                                )
                                .child(
                                    div()
                                        .size(px(24.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .rounded(px(6.0))
                                        .hover(|s| s.bg(rgb(0xf1f3f5)))
                                        .cursor_pointer()
                                        .child(icons::thumbs_up(13.0, rgb(0x81858c))),
                                )
                                .child(
                                    div()
                                        .size(px(24.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .rounded(px(6.0))
                                        .hover(|s| s.bg(rgb(0xf1f3f5)))
                                        .cursor_pointer()
                                        .child(icons::thumbs_down(13.0, rgb(0x81858c))),
                                )
                                .child(
                                    div()
                                        .size(px(24.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .rounded(px(6.0))
                                        .hover(|s| s.bg(rgb(0xf1f3f5)))
                                        .cursor_pointer()
                                        .child(icons::retry(13.0, rgb(0x81858c))),
                                )
                        })
                        .into_any_element(),
                    dsh_core::MessageSender::System => div()
                        .p_3()
                        .rounded_md()
                        .bg(rgb(0xf5f6f8))
                        .text_xs()
                        .text_color(rgb(0x61666b))
                        .child(message.content.clone())
                        .into_any_element(),
                })
                .collect()
        };
        let diff_rows = self
            .pending_diffs
            .iter()
            .map(|diff| {
                let state = self.state.clone();
                let session_id = self.active_session_id.clone();
                let diff_id = diff.id.clone();
                let diff_content = diff.diff_content.clone();
                let file_path = diff.file_path.clone();
                let handle_apply = cx.listener(move |_this, _, _, cx| {
                    let state = state.read(cx).clone();
                    let session_id = session_id.clone();
                    let diff_id = diff_id.clone();
                    let diff_content = diff_content.clone();
                    let file_path = file_path.clone();
                    cx.spawn(async move |this, cx| {
                        if let Some(session_id) = session_id {
                            let result =
                                state.apply_diff(&session_id, &diff_id, &diff_content).await;
                            this.update(cx, |view, cx| {
                                record_diff_result(
                                    &mut view.diff_notice,
                                    result.err().map(|error| error.to_string()),
                                    &format!("无法应用 {}", file_path),
                                );
                                cx.notify();
                            })?;
                        }
                        Ok::<(), anyhow::Error>(())
                    })
                    .detach();
                });
                let state = self.state.clone();
                let session_id = self.active_session_id.clone();
                let diff_id = diff.id.clone();
                let handle_reject = cx.listener(move |_this, _, _, cx| {
                    let state = state.read(cx).clone();
                    let session_id = session_id.clone();
                    let diff_id = diff_id.clone();
                    cx.spawn(async move |this, cx| {
                        if let Some(session_id) = session_id {
                            let result = state.reject_diff(&session_id, &diff_id).await;
                            this.update(cx, |view, cx| {
                                record_diff_result(
                                    &mut view.diff_notice,
                                    result.err().map(|error| error.to_string()),
                                    "无法拒绝变更",
                                );
                                cx.notify();
                            })?;
                        }
                        Ok::<(), anyhow::Error>(())
                    })
                    .detach();
                });
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_3()
                    .rounded_md()
                    .border_1()
                    .border_color(rgb(0x9bb8ff))
                    .bg(rgb(0xf5f8ff))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0x0f1115))
                            .child(diff.file_path.clone()),
                    )
                    .child(
                        div()
                            .max_h(px(96.0))
                            .overflow_hidden()
                            .text_xs()
                            .text_color(rgb(0x3f454d))
                            .child(diff.diff_content.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(menu_choice("应用", false, handle_apply))
                            .child(menu_choice("拒绝", false, handle_reject)),
                    )
                    .into_any_element()
            })
            .collect::<Vec<_>>();

        div()
            .flex_1()
            .overflow_hidden()
            .flex()
            .flex_col()
            .child(self.render_session_header(cx))
            .child(
                div()
                    .flex_1()
                    .p_6()
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .max_w_full()
                            .p_4()
                            .children(message_rows)
                            .when(!self.plan_markdown.is_empty(), |this| {
                                this.child(self.render_plan_review_card(cx))
                            })
                            .when_some(self.pending_question.as_ref(), |this, q| {
                                if q.answered.is_none() {
                                    this.child(self.render_question_card(q, cx))
                                } else {
                                    this
                                }
                            })
                            .when_some(self.diff_notice.as_ref(), |this, notice| {
                                this.child(
                                    div()
                                        .p_2()
                                        .rounded_md()
                                        .bg(rgb(0xffecec))
                                        .text_xs()
                                        .text_color(rgb(0xb42318))
                                        .child(notice.clone()),
                                )
                            })
                            .children(diff_rows),
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
                        .child(self.render_goal_bar(cx))
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
                                .child(self.render_attachments_bar(cx))
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
                                                .child(self.render_plus_button(cx))
                                                .child(
                                                    div()
                                                        .size(px(28.0))
                                                        .rounded_full()
                                                        .bg(rgb(0xf1f3f5))
                                                        .hover(|s| s.bg(rgb(0xe9edf2)))
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .cursor_pointer()
                                                        .on_mouse_down(
                                                            gpui::MouseButton::Left,
                                                            cx.listener(|this, _, _, cx| {
                                                                this.add_attachment_dialog(cx)
                                                            }),
                                                        )
                                                        .child(icons::paperclip(
                                                            14.0,
                                                            rgb(0x61666b),
                                                        )),
                                                )
                                                .child(self.render_access_selector(cx))
                                                .child(self.render_jobs_action(cx))
                                                .when(self.plan_active, |this| {
                                                    this.child(self.render_plan_chip(cx))
                                                }),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_3()
                                                .child(self.render_model_selector(cx))
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

fn menu_choice(
    label: &str,
    selected: bool,
    handler: impl Fn(&gpui::MouseDownEvent, &mut Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap_2()
        .px_2()
        .py_1p5()
        .rounded_md()
        .hover(|s| s.bg(rgb(0xf1f3f5)))
        .cursor_pointer()
        .on_mouse_down(gpui::MouseButton::Left, handler)
        .text_xs()
        .text_color(rgb(0x3f454d))
        .child(label.to_string())
        .child(if selected {
            icons::check(14.0, rgb(0x3964fe)).into_any_element()
        } else {
            div().size(px(14.0)).into_any_element()
        })
}

pub fn format_duration_metric(ms: u64) -> String {
    if ms >= 1000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{ms}ms")
    }
}

pub fn format_token_count(tokens: usize) -> String {
    if tokens >= 1000 {
        format!("{:.1}k", tokens as f64 / 1000.0)
    } else {
        format!("{tokens}")
    }
}

pub fn format_stats_line(
    turns: usize,
    steps: usize,
    llm_ms: u64,
    tool_ms: u64,
    ttft_ms: u64,
    decode_tokens: usize,
    decode_ms: u64,
    cache_hit_percent: Option<u8>,
    input_tokens: usize,
    output_tokens: usize,
) -> String {
    if steps == 0 {
        return String::new();
    }

    let mut groups = Vec::new();

    // 1. 轮/步
    groups.push(format!("{} 轮 · {} 步", turns.max(1), steps.max(1)));

    // 2. 耗时
    let mut durations = Vec::new();
    if llm_ms > 0 {
        durations.push(format!("LLM {}", format_duration_metric(llm_ms)));
    }
    if tool_ms > 0 {
        durations.push(format!("工具调用 {}", format_duration_metric(tool_ms)));
    }
    if !durations.is_empty() {
        groups.push(durations.join(" · "));
    }

    // 3. 速率
    let mut speeds = Vec::new();
    if ttft_ms > 0 {
        speeds.push(format!("首 token 平均 {}", format_duration_metric(ttft_ms)));
    }
    if decode_tokens > 0 && decode_ms > 0 {
        let tps = (decode_tokens as f64) / (decode_ms as f64 / 1000.0);
        speeds.push(format!("{:.1} token/s", tps));
    }
    if !speeds.is_empty() {
        groups.push(speeds.join(" · "));
    }

    // 4. 缓存命中 & Token用量
    if let Some(percent) = cache_hit_percent {
        groups.push(format!("缓存命中 {}%", percent));
    }
    if input_tokens > 0 || output_tokens > 0 {
        groups.push(format!(
            "输入 {} · 输出 {}",
            format_token_count(input_tokens),
            format_token_count(output_tokens)
        ));
    }

    groups.join(" | ")
}

#[cfg(test)]
pub mod stats_tests {
    use super::*;

    #[test]
    fn test_format_duration_metric() {
        assert_eq!(format_duration_metric(450), "450ms");
        assert_eq!(format_duration_metric(1200), "1.2s");
        assert_eq!(format_duration_metric(3000), "3.0s");
    }

    #[test]
    fn test_format_token_count() {
        assert_eq!(format_token_count(350), "350");
        assert_eq!(format_token_count(1500), "1.5k");
        assert_eq!(format_token_count(10000), "10.0k");
    }

    #[test]
    fn test_format_stats_line_empty_when_zero_steps() {
        assert_eq!(format_stats_line(0, 0, 0, 0, 0, 0, 0, None, 0, 0), "");
    }

    #[test]
    fn test_format_stats_line_full_metrics() {
        let line = format_stats_line(1, 3, 1500, 450, 320, 100, 2000, Some(95), 1200, 450);
        assert_eq!(
            line,
            "1 轮 · 3 步 | LLM 1.5s · 工具调用 450ms | 首 token 平均 320ms · 50.0 token/s | 缓存命中 95% | 输入 1.2k · 输出 450"
        );
    }
}

#[cfg(test)]
mod plan_and_question_tests {
    #[test]
    fn test_question_option_single_select_toggle() {
        let mut selected = Vec::new();
        let toggle = |sel: &mut Vec<String>, opt: &str, multi: bool| {
            if multi {
                if let Some(pos) = sel.iter().position(|o| o == opt) {
                    sel.remove(pos);
                } else {
                    sel.push(opt.to_string());
                }
            } else {
                *sel = vec![opt.to_string()];
            }
        };

        toggle(&mut selected, "Option 1", false);
        assert_eq!(selected, vec!["Option 1".to_string()]);
        toggle(&mut selected, "Option 2", false);
        assert_eq!(selected, vec!["Option 2".to_string()]);
    }

    #[test]
    fn test_question_option_multi_select_toggle() {
        let mut selected = Vec::new();
        let toggle = |sel: &mut Vec<String>, opt: &str, multi: bool| {
            if multi {
                if let Some(pos) = sel.iter().position(|o| o == opt) {
                    sel.remove(pos);
                } else {
                    sel.push(opt.to_string());
                }
            } else {
                *sel = vec![opt.to_string()];
            }
        };

        toggle(&mut selected, "Option A", true);
        toggle(&mut selected, "Option B", true);
        assert_eq!(
            selected,
            vec!["Option A".to_string(), "Option B".to_string()]
        );
        toggle(&mut selected, "Option A", true);
        assert_eq!(selected, vec!["Option B".to_string()]);
    }
}

#[cfg(test)]
mod goal_and_jobs_ui_tests {
    use super::*;

    #[test]
    fn test_goal_phase_labels() {
        assert_eq!(goal_phase_label(dsh_protocol::GoalPhase::Active), "进行中");
        assert_eq!(goal_phase_label(dsh_protocol::GoalPhase::Paused), "已暂停");
        assert_eq!(goal_phase_label(dsh_protocol::GoalPhase::Blocked), "阻塞");
        assert_eq!(
            goal_phase_label(dsh_protocol::GoalPhase::Complete),
            "已完成"
        );
    }

    #[test]
    fn test_job_status_labels() {
        assert_eq!(job_status_label(dsh_protocol::JobStatus::Running), "运行中");
        assert_eq!(
            job_status_label(dsh_protocol::JobStatus::Stopping),
            "正在停止"
        );
        assert_eq!(
            job_status_label(dsh_protocol::JobStatus::Completed),
            "已完成"
        );
        assert_eq!(job_status_label(dsh_protocol::JobStatus::Killed), "已终止");
        assert_eq!(job_status_label(dsh_protocol::JobStatus::Failed), "失败");
    }
}
