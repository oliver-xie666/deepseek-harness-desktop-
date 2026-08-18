use std::ops::Range;

use gpui::{
    div, fill, hsla, point, prelude::*, px, relative, size, App, Bounds, Context,
    ElementInputHandler, Entity, EntityInputHandler, FocusHandle, IntoElement, KeyDownEvent,
    LayoutId, MouseButton, MouseDownEvent, PaintQuad, Pixels, ShapedLine, SharedString, TextRun,
    UTF16Selection, Window,
};

pub struct TextInput {
    pub focus_handle: FocusHandle,
    pub content: String,
    pub placeholder: String,
    cursor: usize,
    cursor_visible: bool,
}

impl TextInput {
    pub fn new(placeholder: &str, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            content: String::new(),
            placeholder: placeholder.to_string(),
            cursor: 0,
            cursor_visible: false,
        }
    }

    pub fn text(&self) -> &str {
        &self.content
    }

    pub fn set_text(&mut self, text: &str, cx: &mut Context<Self>) {
        self.content = text.to_string();
        self.cursor = self.content.len();
        self.cursor_visible = true;
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.content.clear();
        self.cursor = 0;
        self.cursor_visible = true;
        cx.notify();
    }

    fn move_left(&mut self, cx: &mut Context<Self>) {
        if self.cursor > 0 {
            self.cursor = previous_boundary(&self.content, self.cursor);
        }
        self.reset_cursor(cx);
    }

    fn move_right(&mut self, cx: &mut Context<Self>) {
        if self.cursor < self.content.len() {
            self.cursor = next_boundary(&self.content, self.cursor);
        }
        self.reset_cursor(cx);
    }

    fn backspace(&mut self, cx: &mut Context<Self>) {
        if self.cursor > 0 {
            let start = previous_boundary(&self.content, self.cursor);
            self.content.drain(start..self.cursor);
            self.cursor = start;
        }
        self.reset_cursor(cx);
    }

    fn insert(&mut self, text: &str, cx: &mut Context<Self>) {
        self.content.insert_str(self.cursor, text);
        self.cursor += text.len();
        self.reset_cursor(cx);
    }

    fn reset_cursor(&mut self, cx: &mut Context<Self>) {
        self.cursor_visible = true;
        cx.notify();
    }

    fn on_mouse_down(
        &mut self,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_handle.focus(window, cx);
        self.cursor = self.content.len();
        self.reset_cursor(cx);
    }

    pub fn on_key_down(&mut self, ev: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        match ev.keystroke.key.as_str() {
            "backspace" => self.backspace(cx),
            "left" => self.move_left(cx),
            "right" => self.move_right(cx),
            "home" => {
                self.cursor = 0;
                self.reset_cursor(cx);
            }
            "end" => {
                self.cursor = self.content.len();
                self.reset_cursor(cx);
            }
            "enter" => self.insert("\n", cx),
            "space" => self.insert(" ", cx),
            _ => {
                if let Some(text) = ev.keystroke.key_char.as_deref() {
                    if !text.chars().any(char::is_control) {
                        self.insert(text, cx);
                    }
                }
            }
        }
    }
}

fn previous_boundary(text: &str, offset: usize) -> usize {
    text[..offset]
        .char_indices()
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0)
}

fn next_boundary(text: &str, offset: usize) -> usize {
    text[offset..]
        .chars()
        .next()
        .map(|ch| offset + ch.len_utf8())
        .unwrap_or(text.len())
}

fn utf16_to_utf8(text: &str, offset: usize) -> usize {
    let mut utf16 = 0;
    for (index, ch) in text.char_indices() {
        if utf16 >= offset {
            return index;
        }
        utf16 += ch.len_utf16();
    }
    text.len()
}

fn utf8_to_utf16(text: &str, offset: usize) -> usize {
    text[..offset].encode_utf16().count()
}

impl EntityInputHandler for TextInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let start = utf16_to_utf8(&self.content, range_utf16.start);
        let end = utf16_to_utf8(&self.content, range_utf16.end);
        actual_range
            .replace(utf8_to_utf16(&self.content, start)..utf8_to_utf16(&self.content, end));
        Some(self.content[start..end].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let cursor = utf8_to_utf16(&self.content, self.cursor);
        Some(UTF16Selection {
            range: cursor..cursor,
            reversed: false,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        None
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {}

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .map(|r| utf16_to_utf8(&self.content, r.start)..utf16_to_utf8(&self.content, r.end))
            .unwrap_or(self.cursor..self.cursor);
        self.content.replace_range(range.clone(), new_text);
        self.cursor = range.start + new_text.len();
        self.reset_cursor(cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _new_selected_range_utf16: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace_text_in_range(range_utf16, new_text, window, cx);
    }

    fn bounds_for_range(
        &mut self,
        _range: Range<usize>,
        _bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        None
    }
    fn character_index_for_point(
        &mut self,
        _point: gpui::Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        None
    }
}

impl Render for TextInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let handle_mouse = cx.listener(|this, event: &MouseDownEvent, window, cx| {
            this.on_mouse_down(event, window, cx);
        });
        let handle_key = cx.listener(|this, event: &KeyDownEvent, window, cx| {
            this.on_key_down(event, window, cx);
        });
        div()
            .track_focus(&self.focus_handle)
            .on_mouse_down(MouseButton::Left, handle_mouse)
            .on_key_down(handle_key)
            .w_full()
            .min_h(px(32.0))
            .text_color(hsla(0.0, 0.0, 0.06, 1.0))
            .child(TextInputElement { input: cx.entity() })
    }
}

struct TextInputElement {
    input: Entity<TextInput>,
}

struct PrepaintState {
    lines: Vec<ShapedLine>,
    cursor: Option<PaintQuad>,
}

impl IntoElement for TextInputElement {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for TextInputElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let input = self.input.read(cx);
        let line_count = input.content.split('\n').count().max(1);
        let mut style = gpui::Style::default();
        style.size.width = relative(1.).into();
        style.size.height = (window.line_height() * line_count as f32).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _state: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> PrepaintState {
        let input = self.input.read(cx);
        let content = input.content.clone();
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let color = style.color;
        let lines = if content.is_empty() {
            let text: SharedString = input.placeholder.clone().into();
            let run = TextRun {
                len: text.len(),
                font: style.font(),
                color: hsla(0., 0., 0.5, 0.5),
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            vec![window
                .text_system()
                .shape_line(text, font_size, &[run], None)]
        } else {
            content
                .split('\n')
                .map(|line| {
                    let text: SharedString = line.to_string().into();
                    let run = TextRun {
                        len: text.len(),
                        font: style.font(),
                        color,
                        background_color: None,
                        underline: None,
                        strikethrough: None,
                    };
                    window
                        .text_system()
                        .shape_line(text, font_size, &[run], None)
                })
                .collect()
        };
        let (line, offset) = cursor_line_offset(&content, input.cursor);
        let line = line.min(lines.len() - 1);
        let cursor_x = if content.is_empty() {
            px(0.)
        } else {
            lines[line].x_for_index(offset)
        };
        let cursor = if input.focus_handle.is_focused(window) && input.cursor_visible {
            Some(fill(
                Bounds::new(
                    point(
                        bounds.left() + cursor_x,
                        bounds.top() + window.line_height() * line as f32,
                    ),
                    size(px(1.5), window.line_height()),
                ),
                color,
            ))
        } else {
            None
        };
        PrepaintState { lines, cursor }
    }

    fn paint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _state: &mut (),
        prepaint: &mut PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let input = self.input.read(cx);
        window.handle_input(
            &input.focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        for (index, line) in prepaint.lines.iter().enumerate() {
            line.paint(
                point(
                    bounds.left(),
                    bounds.top() + window.line_height() * index as f32,
                ),
                window.line_height(),
                gpui::TextAlign::Left,
                None,
                window,
                cx,
            )
            .unwrap();
        }
        if let Some(cursor) = prepaint.cursor.take() {
            window.paint_quad(cursor);
        }
    }
}

fn cursor_line_offset(text: &str, cursor: usize) -> (usize, usize) {
    let line_start = text[..cursor].rfind('\n').map(|i| i + 1).unwrap_or(0);
    (
        text[..cursor].bytes().filter(|&b| b == b'\n').count(),
        cursor - line_start,
    )
}
