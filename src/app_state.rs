use std::sync::Arc;

use tokio::sync::Mutex;

pub enum Mode {
    Terminal,
    Chat,
}

pub struct AppState {
    pub running: bool,
    pub current_mode: Mode,
    pub tick: i64,
    pub ai_response: String,
    pub terminal_context: Arc<Mutex<String>>,
    pub user_chat_to_send_to_gpt: String,
    pub terminal_scroll: usize, // Added to track the scroll position
}

impl AppState {
    pub fn new(terminal_context: Arc<Mutex<String>>) -> Self {
        Self {
            running: true,
            current_mode: Mode::Terminal,
            terminal_context,
            ai_response: "".to_string(),
            tick: 0,
            user_chat_to_send_to_gpt: "".to_string(),
            terminal_scroll: 0, // Initialized to 0
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
