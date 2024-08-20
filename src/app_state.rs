use std::sync::Arc;

use ratatui::{
    style::{Color, Stylize},
    text::{Line, Text},
};
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
    pub fn style(&self) -> Vec<Line> {
        let mut lines = match self.sender {
            MessageSender::User => self
                .message
                .lines()
                .map(|m| Line::from(m).right_aligned())
                .collect(),
            MessageSender::Assistant => self
                .message
                .lines()
                .map(|m| Line::from(m).yellow().left_aligned())
                .collect::<Vec<Line>>(),
        };
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
        }
    }

    pub fn change_mode(&mut self) {
        match self.current_mode {
            Mode::Chat => self.current_mode = Mode::Terminal,
            Mode::Terminal => self.current_mode = Mode::Chat,
        }
    }

    pub fn tick(&mut self) {
        self.tick += 1;
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
