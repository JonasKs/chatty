use std::sync::Arc;

use ratatui::prelude::Stylize;
use ratatui::text::{Line, Span};
use tokio::sync::Mutex;

pub enum Mode {
    Terminal,
    Chat,
}
#[derive(PartialEq)]
pub enum MessageSender {
    Assistant,
    User,
}

pub struct Message {
    pub sender: MessageSender,
    pub message: String,
}

impl Message {
    pub fn style(&self, width: usize, role: String) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        match self.sender {
            MessageSender::User => {
                lines.push(
                    Line::raw(format!(
                        "{}💻 You 💻┐",
                        "─".repeat(width.saturating_sub(12)),
                    ))
                    .right_aligned()
                    .bold()
                    .yellow(),
                );
                lines.extend(
                    self.message
                        .lines()
                        .map(|m| Line::from(m).right_aligned().yellow())
                        .collect::<Vec<Line>>(),
                )
            }
            MessageSender::Assistant => {
                let mut header_spans = vec![];
                header_spans.push(Span::raw("┌🤖 GPT"));
                if role.is_empty() {
                    header_spans.push(Span::raw(" 🤖"))
                } else {
                    header_spans.push(Span::raw(" - "));
                    header_spans.push(Span::raw(format!("{}", role)).on_dark_gray());
                    header_spans.push(Span::raw(" 🤖"));
                }
                header_spans.push(Span::raw(format!(
                    "{}",
                    "─"
                        .to_string()
                        .repeat(width.saturating_sub(15 + role.len()))
                )));

                lines.push(Line::from(header_spans).left_aligned().bold().light_green());
                lines.extend(
                    self.message
                        .lines()
                        .map(|m| Line::from(m).left_aligned())
                        .collect::<Vec<Line>>(),
                )
            }
        }
        lines.push(Line::from("").centered());
        lines
    }
}

pub struct AppState {
    pub running: bool,
    pub current_mode: Mode,
    pub tick: i64,
    pub terminal_context: Arc<Mutex<String>>,
    pub user_chat_to_send_to_gpt: String,
    pub chat_history: Vec<Message>,
    pub disable_chat: bool,
    pub scroll: u16,
    pub terminal_has_been_active: bool,
}

impl AppState {
    pub fn new(terminal_context: Arc<Mutex<String>>) -> Self {
        Self {
            running: true,
            current_mode: Mode::Chat,
            terminal_context,
            tick: 0,
            user_chat_to_send_to_gpt: String::new(),
            chat_history: Vec::new(),
            disable_chat: false,
            scroll: 0,
            terminal_has_been_active: false,
        }
    }

    pub fn change_mode(&mut self) {
        match self.current_mode {
            Mode::Chat => self.current_mode = Mode::Terminal,
            Mode::Terminal => {
                self.terminal_has_been_active = true;
                self.current_mode = Mode::Chat
            }
        }
    }

    pub fn tick(&mut self) {
        self.tick += 1;
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
