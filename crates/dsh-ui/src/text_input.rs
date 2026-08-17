use gpui::{
    div, prelude::*, rgb, Context, CursorStyle, FocusHandle, IntoElement, KeyDownEvent,
    MouseButton, MouseDownEvent, Window,
};

pub struct TextInput {
    pub focus_handle: FocusHandle,
    pub content: String,
    pub placeholder: String,
}

impl TextInput {
    pub fn new(placeholder: &str, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            content: String::new(),
            placeholder: placeholder.to_string(),
        }
    }

    pub fn text(&self) -> &str {
        &self.content
    }

    pub fn set_text(&mut self, text: &str, cx: &mut Context<Self>) {
        self.content = text.to_string();
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.content.clear();
        cx.notify();
    }

    pub fn on_mouse_down(&mut self, _ev: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_handle.focus(window, cx);
        cx.notify();
    }

    pub fn on_key_down(&mut self, ev: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let key = ev.keystroke.key.as_str();

        match key {
            "backspace" => {
                self.content.pop();
                cx.notify();
            }
            "space" => {
                self.content.push(' ');
                cx.notify();
            }
            "enter" => {
                // Enter handled by parent/form if needed
            }
            _ => {
                if let Some(ref key_char) = ev.keystroke.key_char {
                    self.content.push_str(key_char);
                    cx.notify();
                } else if key.chars().count() == 1 {
                    let ch = key.chars().next().unwrap();
                    if !ch.is_control() {
                        self.content.push(ch);
                        cx.notify();
                    }
                }
            }
        }
    }
}

impl Render for TextInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_focused = self.focus_handle.is_focused(window);
        let has_text = !self.content.is_empty();

        let display_text = if has_text {
            self.content.clone()
        } else {
            self.placeholder.clone()
        };

        let text_color = if has_text {
            rgb(0xffffff)
        } else {
            rgb(0x61666b)
        };

        let handle_mouse = cx.listener(|this, ev: &MouseDownEvent, window, cx| {
            this.on_mouse_down(ev, window, cx);
        });

        let handle_key = cx.listener(|this, ev: &KeyDownEvent, window, cx| {
            this.on_key_down(ev, window, cx);
        });

        div()
            .track_focus(&self.focus_handle)
            .w_full()
            .min_h(gpui::px(32.0))
            .cursor(CursorStyle::IBeam)
            .on_mouse_down(MouseButton::Left, handle_mouse)
            .on_key_down(handle_key)
            .flex()
            .items_center()
            .text_sm()
            .text_color(text_color)
            .child(display_text)
            .when(is_focused, |this| {
                this.child(
                    div()
                        .w(gpui::px(2.0))
                        .h(gpui::px(16.0))
                        .bg(rgb(0x4176e6))
                        .ml_0p5(),
                )
            })
    }
}
