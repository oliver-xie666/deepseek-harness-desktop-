use std::ops::Range;

use gpui::{
    div, fill, hsla, point, prelude::*, px, relative, size, App, Bounds, Context,
    ElementInputHandler, Entity, EntityInputHandler, FocusHandle, IntoElement, KeyDownEvent,
    LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels,
    ShapedLine, SharedString, TextRun, UTF16Selection, Window,
};

pub struct TextInput {
    pub focus_handle: FocusHandle,
    pub content: String,
    pub placeholder: String,
    selection: Range<usize>,
    selection_reversed: bool,
    cursor_visible: bool,
    last_layouts: Vec<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    is_selecting: bool,
    enter_behavior: String,
    submit_requested: bool,
}

impl TextInput {
    pub fn new(placeholder: &str, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            content: String::new(),
            placeholder: placeholder.to_string(),
            selection: 0..0,
            selection_reversed: false,
            cursor_visible: false,
            last_layouts: Vec::new(),
            last_bounds: None,
            is_selecting: false,
            enter_behavior: "queue".into(),
            submit_requested: false,
        }
    }

    pub fn text(&self) -> &str {
        &self.content
    }

    pub fn set_text(&mut self, text: &str, cx: &mut Context<Self>) {
        self.content = text.to_string();
        self.selection = self.content.len()..self.content.len();
        self.selection_reversed = false;
        self.cursor_visible = true;
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.content.clear();
        self.selection = 0..0;
        self.selection_reversed = false;
        self.cursor_visible = true;
        cx.notify();
    }

    pub fn set_enter_behavior(&mut self, behavior: &str, cx: &mut Context<Self>) {
        self.enter_behavior = behavior.to_string();
        cx.notify();
    }

    pub fn take_submit_requested(&mut self, cx: &mut Context<Self>) -> bool {
        let requested = self.submit_requested;
        self.submit_requested = false;
        if requested {
            cx.notify();
        }
        requested
    }

    fn move_left(&mut self, cx: &mut Context<Self>) {
        if !self.selection.is_empty() {
            self.move_to(self.selection.start, cx);
        } else {
            let cursor = self.cursor_offset();
            self.move_to(previous_boundary(&self.content, cursor), cx);
        }
        self.reset_cursor(cx);
    }

    fn move_right(&mut self, cx: &mut Context<Self>) {
        if !self.selection.is_empty() {
            self.move_to(self.selection.end, cx);
        } else {
            let cursor = self.cursor_offset();
            self.move_to(next_boundary(&self.content, cursor), cx);
        }
        self.reset_cursor(cx);
    }

    fn backspace(&mut self, cx: &mut Context<Self>) {
        if self.selection.is_empty() {
            let cursor = self.cursor_offset();
            if cursor > 0 {
                self.selection = previous_boundary(&self.content, cursor)..cursor;
            }
        }
        self.replace_selection("");
        self.reset_cursor(cx);
    }

    fn insert(&mut self, text: &str, cx: &mut Context<Self>) {
        self.replace_selection(text);
        self.reset_cursor(cx);
    }

    fn delete(&mut self, cx: &mut Context<Self>) {
        if self.selection.is_empty() {
            let cursor = self.cursor_offset();
            if cursor < self.content.len() {
                self.selection = cursor..next_boundary(&self.content, cursor);
            }
        }
        self.replace_selection("");
        self.reset_cursor(cx);
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selection.start
        } else {
            self.selection.end
        }
    }

    fn move_to(&mut self, offset: usize, _cx: &mut Context<Self>) {
        self.selection = offset..offset;
        self.selection_reversed = false;
    }

    fn select_to(&mut self, offset: usize, _cx: &mut Context<Self>) {
        let anchor = self.cursor_offset();
        if offset >= anchor {
            self.selection = anchor..offset;
            self.selection_reversed = false;
        } else {
            self.selection = offset..anchor;
            self.selection_reversed = true;
        }
    }

    fn replace_selection(&mut self, text: &str) {
        let range = self.selection.clone();
        self.content.replace_range(range.clone(), text);
        let cursor = range.start + text.len();
        self.selection = cursor..cursor;
        self.selection_reversed = false;
    }

    fn index_for_mouse_position(&self, position: gpui::Point<Pixels>, window: &Window) -> usize {
        if self.content.is_empty() {
            return 0;
        }
        let Some(bounds) = self.last_bounds else {
            return self.content.len();
        };
        let local = bounds
            .localize(&position)
            .unwrap_or(point(px(0.0), px(0.0)));
        let line_height = window.line_height();
        let line_index = (local.y / line_height).floor().max(0.0) as usize;
        let line_index = line_index.min(self.last_layouts.len().saturating_sub(1));
        let mut start = 0;
        for line in self.last_layouts.iter().take(line_index) {
            start += line.text.len() + 1;
        }
        let line = match self.last_layouts.get(line_index) {
            Some(line) => line,
            None => return self.content.len(),
        };
        (start + line.closest_index_for_x(local.x.max(px(0.0)))).min(self.content.len())
    }

    fn reset_cursor(&mut self, cx: &mut Context<Self>) {
        self.cursor_visible = true;
        cx.notify();
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_handle.focus(window, cx);
        self.is_selecting = true;
        let offset = self.index_for_mouse_position(event.position, window);
        if event.modifiers.shift {
            self.select_to(offset, cx);
        } else {
            self.move_to(offset, cx);
        }
        self.reset_cursor(cx);
    }

    fn on_mouse_up(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        self.is_selecting = false;
    }

    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_selecting {
            self.select_to(self.index_for_mouse_position(event.position, window), cx);
            self.reset_cursor(cx);
        }
    }

    pub fn on_key_down(&mut self, ev: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        match ev.keystroke.key.as_str() {
            "backspace" => self.backspace(cx),
            "delete" => self.delete(cx),
            "left" => self.move_left(cx),
            "right" => self.move_right(cx),
            "home" => {
                self.move_to(0, cx);
                self.reset_cursor(cx);
            }
            "end" => {
                self.move_to(self.content.len(), cx);
                self.reset_cursor(cx);
            }
            "enter" if self.enter_behavior == "queue" => {
                self.submit_requested = true;
                cx.notify();
            }
            "enter" => self.insert("\n", cx),
            "a" if ev.keystroke.modifiers.control || ev.keystroke.modifiers.platform => {
                self.selection = 0..self.content.len();
                self.selection_reversed = false;
                self.reset_cursor(cx);
            }
            _ => {}
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
        Some(UTF16Selection {
            range: utf8_to_utf16(&self.content, self.selection.start)
                ..utf8_to_utf16(&self.content, self.selection.end),
            reversed: self.selection_reversed,
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
            .unwrap_or_else(|| self.selection.clone());
        self.content.replace_range(range.clone(), new_text);
        let cursor = range.start + new_text.len();
        self.selection = cursor..cursor;
        self.selection_reversed = false;
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
        point: gpui::Point<Pixels>,
        window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        Some(utf8_to_utf16(
            &self.content,
            self.index_for_mouse_position(point, window),
        ))
    }
}

impl Render for TextInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let handle_mouse = cx.listener(|this, event: &MouseDownEvent, window, cx| {
            this.on_mouse_down(event, window, cx);
        });
        let handle_mouse_up = cx.listener(|this, event: &MouseUpEvent, window, cx| {
            this.on_mouse_up(event, window, cx);
        });
        let handle_mouse_move = cx.listener(|this, event: &MouseMoveEvent, window, cx| {
            this.on_mouse_move(event, window, cx);
        });
        let handle_key = cx.listener(|this, event: &KeyDownEvent, window, cx| {
            this.on_key_down(event, window, cx);
        });
        div()
            .track_focus(&self.focus_handle)
            .on_mouse_down(MouseButton::Left, handle_mouse)
            .on_mouse_up(MouseButton::Left, handle_mouse_up)
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, event: &MouseUpEvent, window, cx| {
                    this.on_mouse_up(event, window, cx);
                }),
            )
            .on_mouse_move(handle_mouse_move)
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
    selection: Vec<PaintQuad>,
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
        let cursor = input.cursor_offset();
        let selected_range = input.selection.clone();
        let (line, offset) = cursor_line_offset(&content, cursor);
        let line = line.min(lines.len() - 1);
        let cursor_x = if content.is_empty() {
            px(0.)
        } else {
            lines[line].x_for_index(offset)
        };
        let mut selection = Vec::new();
        if !content.is_empty() && !selected_range.is_empty() {
            let mut line_start = 0;
            for (line_index, shaped_line) in lines.iter().enumerate() {
                let line_end = line_start + shaped_line.text.len();
                let start = selected_range.start.max(line_start);
                let end = selected_range.end.min(line_end);
                if start < end {
                    selection.push(fill(
                        Bounds::from_corners(
                            point(
                                bounds.left() + shaped_line.x_for_index(start - line_start),
                                bounds.top() + window.line_height() * line_index as f32,
                            ),
                            point(
                                bounds.left() + shaped_line.x_for_index(end - line_start),
                                bounds.top() + window.line_height() * (line_index + 1) as f32,
                            ),
                        ),
                        gpui::rgba(0x3964fe55),
                    ));
                }
                line_start = line_end + 1;
            }
        }
        let cursor = if selected_range.is_empty()
            && input.focus_handle.is_focused(window)
            && input.cursor_visible
        {
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
        PrepaintState {
            lines,
            selection,
            cursor,
        }
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
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        for selection in prepaint.selection.drain(..) {
            window.paint_quad(selection);
        }
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
        self.input.update(cx, |input, _cx| {
            input.last_layouts = prepaint.lines.clone();
            input.last_bounds = Some(bounds);
        });
    }
}

fn cursor_line_offset(text: &str, cursor: usize) -> (usize, usize) {
    let line_start = text[..cursor].rfind('\n').map(|i| i + 1).unwrap_or(0);
    (
        text[..cursor].bytes().filter(|&b| b == b'\n').count(),
        cursor - line_start,
    )
}
